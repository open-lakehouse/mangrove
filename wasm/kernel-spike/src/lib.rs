//! B0 spike (#198): drive the published, synchronous `buoyant_kernel` 0.28 from async code
//! without ever blocking on IO.
//!
//! [`ReplayEngine`] is a kernel [`Engine`] whose handlers read only from an in-memory cache. A
//! cache miss is *recorded* and fails fast with a pending error — it never blocks. [`replay`] runs
//! a kernel call, fetches the recorded misses concurrently through `object_store`, and re-runs the
//! call until a pass completes with **zero** misses. Only that final pass's result is returned, so
//! any fallback the kernel took on a pending error in an earlier pass can never leak out: the
//! accepted pass saw exactly what a blocking engine would have seen.
//!
//! Kernel calls are deterministic given the same inputs, so each pass gets at least one
//! dependency level further (list `_delta_log` → read `_last_checkpoint` / commits / checkpoint →
//! …). The re-runs only repeat kernel CPU work, never IO.
//!
//! Scope: read-only. Writes are unsupported, and whole objects are cached (a parquet footer read
//! fetches the full file) — fine for `_delta_log` sized inputs, an optimization point for large
//! checkpoints.

use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::sync::{Arc, Mutex};

use buoyant_kernel::arrow::json::ReaderBuilder;
use buoyant_kernel::engine::arrow_conversion::TryFromArrow as _;
use buoyant_kernel::engine::arrow_data::ArrowEngineData;
use buoyant_kernel::engine::arrow_expression::ArrowEvaluationHandler;
use buoyant_kernel::engine::arrow_utils::{
    RowIndexBuilder, build_json_reorder_indices, fixup_json_read, fixup_parquet_read,
    json_arrow_schema, ordering_needs_row_indexes, parquet_read_plan, parse_json,
};
use buoyant_kernel::parquet::arrow::arrow_reader::{
    ArrowReaderMetadata, ArrowReaderOptions, ParquetRecordBatchReaderBuilder,
};
use buoyant_kernel::plans::ir::nodes::Operator;
use buoyant_kernel::plans::{IoOperation, Operation, PlanExecutor, PlanResult};
use buoyant_kernel::schema::{SchemaRef, StructType};
use buoyant_kernel::{
    DeltaResult, DeltaResultIterator, DeltaResultIteratorStatic, Engine, EngineData, Error,
    EvaluationHandler, FileDataReadResultIterator, FileMeta, FileSize, FileSlice,
    FilteredEngineData, JsonHandler, ParquetFooter, ParquetHandler, PredicateRef, StorageHandler,
};
use bytes::Bytes;
use futures::TryStreamExt as _;
use object_store::path::Path;
use object_store::{ObjectStore, ObjectStoreExt as _};
use url::Url;

/// Upper bound on replay passes. Each pass resolves at least one dependency level of IO, and a
/// snapshot + scan-metadata build needs only a handful; hitting this means the kernel call is not
/// converging (e.g. it requests a different object on every pass).
pub const MAX_PASSES: usize = 16;

const PENDING: &str = "replay: IO pending (not yet fetched)";

/// A kernel IO request the cache could not serve yet.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Miss {
    /// `StorageHandler::list_from(url)`.
    List(Url),
    /// Whole-object read (`read_files`, JSON/parquet reads, footers).
    Get(Url),
    /// `StorageHandler::head(url)`.
    Head(Url),
}

#[derive(Debug, Clone)]
enum Entry<T> {
    Present(T),
    NotFound,
}

#[derive(Debug, Default)]
struct State {
    listings: HashMap<Url, Vec<FileMeta>>,
    objects: HashMap<Url, Entry<Bytes>>,
    heads: HashMap<Url, Entry<FileMeta>>,
    misses: HashSet<Miss>,
}

/// Counters describing one [`replay`] run, for the spike report.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ReplayStats {
    /// Kernel passes executed, including the final zero-miss pass.
    pub passes: usize,
    /// Distinct IO requests fetched across all passes.
    pub fetches: usize,
}

struct Inner {
    state: Arc<Mutex<State>>,
    evaluation: Arc<ArrowEvaluationHandler>,
    /// Expose an IO-only [`PlanExecutor`]. Required by `Scan::declarative_metadata_scan_plan`
    /// (checkpoint-shape footer probes); kept off for snapshot construction so the kernel's P&M
    /// replay takes the handler path instead of issuing `QueryPlan`s we do not interpret.
    plan_io: bool,
}

impl Inner {
    fn record(&self, miss: Miss) {
        self.state.lock().unwrap().misses.insert(miss);
    }

    /// Serve a whole object, recording a miss when absent.
    fn object(&self, url: &Url) -> DeltaResult<Bytes> {
        let state = self.state.lock().unwrap();
        match state.objects.get(url) {
            Some(Entry::Present(bytes)) => Ok(bytes.clone()),
            Some(Entry::NotFound) => Err(Error::file_not_found(url.as_str())),
            None => {
                drop(state);
                self.record(Miss::Get(url.clone()));
                Err(Error::generic(PENDING))
            }
        }
    }

    /// Serve every object in `urls`, recording *all* misses before failing so one round fetches
    /// the whole batch rather than one file per pass.
    fn objects<'a>(&self, urls: impl IntoIterator<Item = &'a Url>) -> DeltaResult<Vec<Bytes>> {
        let results: Vec<_> = urls.into_iter().map(|url| self.object(url)).collect();
        results.into_iter().collect()
    }
}

/// A read-only kernel [`Engine`] over a fetch-on-replay cache. See the crate docs.
#[derive(Clone)]
pub struct ReplayEngine {
    inner: Arc<Inner>,
}

impl ReplayEngine {
    /// An engine for snapshot construction / imperative scan metadata (no plan executor).
    pub fn new() -> Self {
        Self::with_plan_io(false)
    }

    /// An engine that also exposes an IO-only [`PlanExecutor`] (needed by the declarative scan
    /// plan API). `QueryPlan`s are rejected as unsupported.
    pub fn with_plan_io(plan_io: bool) -> Self {
        Self {
            inner: Arc::new(Inner {
                state: Arc::default(),
                evaluation: Arc::new(ArrowEvaluationHandler),
                plan_io,
            }),
        }
    }

    /// Share this engine's cache with a sibling engine of a different `plan_io` mode, so a scan
    /// built after the snapshot reuses everything the snapshot pass already fetched.
    pub fn sharing_cache(&self, plan_io: bool) -> Self {
        Self {
            inner: Arc::new(Inner {
                state: Arc::clone(&self.inner.state),
                evaluation: Arc::clone(&self.inner.evaluation),
                plan_io,
            }),
        }
    }

    fn take_misses(&self) -> Vec<Miss> {
        let mut state = self.inner.state.lock().unwrap();
        state.misses.drain().collect()
    }
}

impl Default for ReplayEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine for ReplayEngine {
    fn evaluation_handler(&self) -> Arc<dyn EvaluationHandler> {
        self.inner.evaluation.clone()
    }

    fn storage_handler(&self) -> Arc<dyn StorageHandler> {
        Arc::new(Storage(self.inner.clone()))
    }

    fn json_handler(&self) -> Arc<dyn JsonHandler> {
        Arc::new(Json(self.inner.clone()))
    }

    fn parquet_handler(&self) -> Arc<dyn ParquetHandler> {
        Arc::new(Parquet(self.inner.clone()))
    }

    fn plan_executor(&self) -> Option<Arc<dyn PlanExecutor>> {
        self.inner
            .plan_io
            .then(|| Arc::new(IoExecutor(self.inner.clone())) as Arc<dyn PlanExecutor>)
    }
}

struct Storage(Arc<Inner>);

impl StorageHandler for Storage {
    fn list_from(
        &self,
        path: &Url,
    ) -> DeltaResult<Box<dyn Iterator<Item = DeltaResult<FileMeta>>>> {
        let listed = self.0.state.lock().unwrap().listings.get(path).cloned();
        match listed {
            Some(metas) => Ok(Box::new(metas.into_iter().map(Ok))),
            None => {
                self.0.record(Miss::List(path.clone()));
                Err(Error::generic(PENDING))
            }
        }
    }

    fn read_files(
        &self,
        files: Vec<FileSlice>,
    ) -> DeltaResult<Box<dyn Iterator<Item = DeltaResult<Bytes>>>> {
        // Record every miss up front, then hand back per-file results (a `FileNotFound` on one
        // file must surface on that item, as the default engine does).
        let results: Vec<DeltaResult<Bytes>> = files
            .into_iter()
            .map(|(url, range)| {
                let bytes = self.0.object(&url)?;
                Ok(match range {
                    Some(range) => bytes.slice(range.start as usize..range.end as usize),
                    None => bytes,
                })
            })
            .collect();
        if results
            .iter()
            .any(|r| matches!(r, Err(Error::Generic(m)) if m == PENDING))
        {
            return Err(Error::generic(PENDING));
        }
        Ok(Box::new(results.into_iter()))
    }

    fn copy_atomic(&self, _src: &Url, _dest: &Url) -> DeltaResult<()> {
        Err(Error::unsupported("ReplayEngine is read-only"))
    }

    fn put(&self, _path: &Url, _data: Bytes, _overwrite: bool) -> DeltaResult<()> {
        Err(Error::unsupported("ReplayEngine is read-only"))
    }

    fn head(&self, path: &Url) -> DeltaResult<FileMeta> {
        let head = self.0.state.lock().unwrap().heads.get(path).cloned();
        match head {
            Some(Entry::Present(meta)) => Ok(meta),
            Some(Entry::NotFound) => Err(Error::file_not_found(path.as_str())),
            None => {
                self.0.record(Miss::Head(path.clone()));
                Err(Error::generic(PENDING))
            }
        }
    }

    fn delete(&self, _path: &Url) -> DeltaResult<()> {
        Err(Error::unsupported("ReplayEngine is read-only"))
    }
}

struct Json(Arc<Inner>);

impl JsonHandler for Json {
    fn parse_json(
        &self,
        json_strings: Box<dyn EngineData>,
        output_schema: SchemaRef,
    ) -> DeltaResult<Box<dyn EngineData>> {
        parse_json(json_strings, output_schema)
    }

    fn read_json_files(
        &self,
        files: &[FileMeta],
        physical_schema: SchemaRef,
        _predicate: Option<PredicateRef>,
    ) -> DeltaResult<FileDataReadResultIterator> {
        let contents = self.0.objects(files.iter().map(|f| &f.location))?;
        let arrow_schema = Arc::new(json_arrow_schema(&physical_schema)?);
        let reorder = build_json_reorder_indices(&physical_schema)?;
        // Mirrors the default engine's per-file JSON decode (`open_json_file` + `fixup_json_read`)
        // over in-memory bytes.
        let mut out: Vec<DeltaResult<Box<dyn EngineData>>> = Vec::new();
        for (file, bytes) in files.iter().zip(contents) {
            let reader = ReaderBuilder::new(arrow_schema.clone())
                .with_coerce_primitive(true)
                .build(Cursor::new(bytes))?;
            for batch in reader {
                out.push(
                    batch
                        .map_err(Error::from)
                        .and_then(|b| fixup_json_read(b, &reorder, file.location.as_str()))
                        .map(|data| Box::new(data) as Box<dyn EngineData>),
                );
            }
        }
        Ok(Box::new(out.into_iter()))
    }

    fn write_json_file(
        &self,
        _path: &Url,
        _data: DeltaResultIterator<'_, FilteredEngineData>,
        _overwrite: bool,
    ) -> DeltaResult<FileSize> {
        Err(Error::unsupported("ReplayEngine is read-only"))
    }
}

struct Parquet(Arc<Inner>);

fn reader_options() -> ArrowReaderOptions {
    ArrowReaderOptions::new().with_skip_arrow_metadata(true)
}

impl ParquetHandler for Parquet {
    fn read_parquet_files(
        &self,
        files: &[FileMeta],
        physical_schema: SchemaRef,
        // A row-group skipping hint; ignoring it is always correct.
        _predicate: Option<PredicateRef>,
    ) -> DeltaResult<FileDataReadResultIterator> {
        let contents = self.0.objects(files.iter().map(|f| &f.location))?;
        // Mirrors the default engine's sync parquet decode: plan the projection against the
        // footer, read, then let the kernel fix up ordering / row indexes / nested nulls.
        let mut out: Vec<DeltaResult<Box<dyn EngineData>>> = Vec::new();
        for (file, bytes) in files.iter().zip(contents) {
            let metadata = ArrowReaderMetadata::load(&bytes, reader_options())?;
            let (ordering, mask) = parquet_read_plan(&physical_schema, &metadata)?;
            let mut row_indexes = ordering_needs_row_indexes(&ordering)
                .then(|| RowIndexBuilder::new(metadata.metadata().row_groups()).build())
                .transpose()?;
            let mut builder = ParquetRecordBatchReaderBuilder::new_with_metadata(bytes, metadata);
            if let Some(mask) = mask {
                builder = builder.with_projection(mask);
            }
            for batch in builder.build()? {
                out.push(
                    batch
                        .map_err(Error::from)
                        .and_then(|b| {
                            fixup_parquet_read(
                                b,
                                &ordering,
                                row_indexes.as_mut(),
                                Some(file.location.as_str()),
                                Some(&physical_schema),
                            )
                        })
                        .map(|data: ArrowEngineData| Box::new(data) as Box<dyn EngineData>),
                );
            }
        }
        Ok(Box::new(out.into_iter()))
    }

    fn write_parquet_file(
        &self,
        _location: Url,
        _data: DeltaResultIteratorStatic<Box<dyn EngineData>>,
    ) -> DeltaResult<()> {
        Err(Error::unsupported("ReplayEngine is read-only"))
    }

    fn read_parquet_footer(&self, file: &FileMeta) -> DeltaResult<ParquetFooter> {
        let bytes = self.0.object(&file.location)?;
        let metadata = ArrowReaderMetadata::load(&bytes, reader_options())?;
        let schema = Arc::new(StructType::try_from_arrow(metadata.schema().as_ref())?);
        Ok(ParquetFooter { schema })
    }
}

/// IO-only plan executor: serves `IoOperation`s from the cache via the handlers above.
struct IoExecutor(Arc<Inner>);

impl PlanExecutor for IoExecutor {
    fn execute_op(&self, op: Operation) -> DeltaResult<PlanResult> {
        let storage = Storage(self.0.clone());
        match op {
            Operation::IoOperation(IoOperation::FileListing { url }) => Ok(PlanResult::FileMeta(
                Box::new(storage.list_from(&url)?.collect::<Vec<_>>().into_iter()),
            )),
            Operation::IoOperation(IoOperation::ReadBytes { files }) => Ok(PlanResult::Bytes(
                Box::new(storage.read_files(files)?.collect::<Vec<_>>().into_iter()),
            )),
            Operation::IoOperation(IoOperation::HeadFile { url }) => Ok(PlanResult::FileMeta(
                Box::new(std::iter::once(storage.head(&url))),
            )),
            Operation::IoOperation(IoOperation::ParquetFooter { file }) => Ok(
                PlanResult::ParquetFooter(Parquet(self.0.clone()).read_parquet_footer(&file)?),
            ),
            Operation::IoOperation(op) => Err(Error::unsupported(format!(
                "ReplayEngine plan executor is read-only: {op:?}"
            ))),
            // Leaf-only plans (a single scan with no file constants) are exactly a handler read —
            // e.g. the checkpoint-shape sidecar probe. Anything with real operators is the
            // connector-native path (compile into DataFusion), not ours to interpret here.
            Operation::QueryPlan(plan) => match plan.nodes.as_slice() {
                [node] if node.inputs.is_empty() => match &node.op {
                    Operator::ScanParquet(scan) if scan.file_constant_columns.is_empty() => {
                        let files: Vec<FileMeta> =
                            scan.files.iter().map(|f| f.meta.clone()).collect();
                        Ok(PlanResult::Data(Box::new(
                            Parquet(self.0.clone())
                                .read_parquet_files(&files, scan.schema.clone(), None)?
                                .collect::<Vec<_>>()
                                .into_iter(),
                        )))
                    }
                    Operator::ScanJson(scan) if scan.file_constant_columns.is_empty() => {
                        let files: Vec<FileMeta> =
                            scan.files.iter().map(|f| f.meta.clone()).collect();
                        Ok(PlanResult::Data(Box::new(
                            Json(self.0.clone())
                                .read_json_files(&files, scan.schema.clone(), None)?
                                .collect::<Vec<_>>()
                                .into_iter(),
                        )))
                    }
                    other => Err(Error::unsupported(format!(
                        "ReplayEngine plan executor only serves leaf scans, got {other}"
                    ))),
                },
                _ => Err(Error::unsupported(
                    "ReplayEngine plan executor only serves single-node leaf scan plans",
                )),
            },
        }
    }
}

/// Error from [`replay`].
#[derive(Debug)]
pub enum ReplayError {
    /// The accepted (zero-miss) pass failed: a genuine kernel error.
    Kernel(Error),
    /// Fetching a recorded miss failed.
    Store(object_store::Error),
    /// No zero-miss pass within [`MAX_PASSES`].
    NotConverged { passes: usize },
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kernel(e) => write!(f, "kernel: {e}"),
            Self::Store(e) => write!(f, "store: {e}"),
            Self::NotConverged { passes } => {
                write!(f, "replay did not converge in {passes} passes")
            }
        }
    }
}

impl std::error::Error for ReplayError {}

/// Run `call` against `engine`, fetching its IO through `store` between passes, until a pass
/// records no misses. Returns that pass's result plus run statistics.
///
/// `store` is addressed by URL path (as the kernel default engine does), so it must be rooted at
/// the URL authority of the table (e.g. an `InMemory` for `memory:///…`, or a container-rooted
/// cloud store).
pub async fn replay<T>(
    store: &dyn ObjectStore,
    engine: &ReplayEngine,
    mut call: impl FnMut(&ReplayEngine) -> DeltaResult<T>,
) -> Result<(T, ReplayStats), ReplayError> {
    let mut stats = ReplayStats::default();
    while stats.passes < MAX_PASSES {
        stats.passes += 1;
        let result = call(engine);
        let misses = engine.take_misses();
        if misses.is_empty() {
            return result.map(|v| (v, stats)).map_err(ReplayError::Kernel);
        }
        stats.fetches += misses.len();
        let fetched = futures::future::try_join_all(misses.into_iter().map(|m| fetch(store, m)))
            .await
            .map_err(ReplayError::Store)?;
        let mut state = engine.inner.state.lock().unwrap();
        for f in fetched {
            match f {
                Fetched::List(url, metas) => {
                    state.listings.insert(url, metas);
                }
                Fetched::Get(url, entry) => {
                    state.objects.insert(url, entry);
                }
                Fetched::Head(url, entry) => {
                    state.heads.insert(url, entry);
                }
            }
        }
    }
    Err(ReplayError::NotConverged {
        passes: stats.passes,
    })
}

enum Fetched {
    List(Url, Vec<FileMeta>),
    Get(Url, Entry<Bytes>),
    Head(Url, Entry<FileMeta>),
}

fn object_path(url: &Url) -> Result<Path, object_store::Error> {
    Path::from_url_path(url.path()).map_err(|e| object_store::Error::Generic {
        store: "replay",
        source: Box::new(e),
    })
}

fn meta_for(url: &Url, meta: &object_store::ObjectMeta) -> FileMeta {
    let mut location = url.clone();
    location.set_path(&format!("/{}", meta.location.as_ref()));
    FileMeta {
        location,
        last_modified: meta.last_modified.timestamp_millis(),
        size: meta.size,
    }
}

async fn fetch(store: &dyn ObjectStore, miss: Miss) -> Result<Fetched, object_store::Error> {
    match miss {
        // Same semantics as the default engine's `list_from`: list the parent "directory" of
        // `url` starting *after* `url`'s path (exclusive offset), sorted.
        Miss::List(url) => {
            let offset = object_path(&url)?;
            let prefix = if url.path().ends_with('/') {
                offset.clone()
            } else {
                let mut parts: Vec<_> = offset.parts().collect();
                parts.pop();
                Path::from_iter(parts)
            };
            let mut metas: Vec<FileMeta> = store
                .list_with_offset(Some(&prefix), &offset)
                .map_ok(|meta| meta_for(&url, &meta))
                .try_collect()
                .await?;
            metas.sort_unstable();
            Ok(Fetched::List(url, metas))
        }
        Miss::Get(url) => {
            let path = object_path(&url)?;
            let entry = match store.get(&path).await {
                Ok(result) => Entry::Present(result.bytes().await?),
                Err(object_store::Error::NotFound { .. }) => Entry::NotFound,
                Err(e) => return Err(e),
            };
            Ok(Fetched::Get(url, entry))
        }
        Miss::Head(url) => {
            let path = object_path(&url)?;
            let entry = match store.head(&path).await {
                Ok(meta) => Entry::Present(meta_for(&url, &meta)),
                Err(object_store::Error::NotFound { .. }) => Entry::NotFound,
                Err(e) => return Err(e),
            };
            Ok(Fetched::Head(url, entry))
        }
    }
}
