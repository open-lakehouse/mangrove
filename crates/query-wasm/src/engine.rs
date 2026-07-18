//! Query execution over an opened Delta table: SQL table-reference extraction,
//! qualified-name registration, and the `open_lakehouse.query.v1` chunk framing.
//!
//! The runner contract hands us raw SQL (built by `@open-lakehouse/query` as
//! ``SELECT … FROM `catalog`.`schema`.`table` LIMIT n``). The engine extracts
//! the single table reference, resolves it against the request's optional
//! default catalog/schema (the Unity Catalog address), opens the table through
//! [`deltalake_wasm`], registers an async-native scan
//! ([`DeltaSsaTableProvider`]) under exactly the name the SQL will resolve to,
//! and executes — emitting one **self-contained** Arrow IPC stream per record
//! batch (schema + batch + EOS), per the proto contract. This differs from
//! `deltalake_wasm::query_ipc`, whose chunks form one incremental stream and
//! only parse when concatenated.
//!
//! Both snapshot construction *and* the scan are now async-native and engine-free.
//! Construction drives the kernel `sm_plans` P&M state machine over the discovered
//! `_delta_log` manifest ([`build_snapshot_from_manifest`]) — list-free (no
//! directory listing) and with **no `PrimedStore` prefetch** and **no
//! `InlineExecutor`**. The scan is the `sm_plans`-driven [`DeltaSsaTableProvider`],
//! which streams data files lazily through DataFusion's own async object-store
//! stack. The `deltalake_wasm` facade is no longer on this path.

use std::sync::Arc;

use arrow_array::RecordBatch;
use arrow_ipc::writer::StreamWriter;
use arrow_schema::SchemaRef;
use datafusion::catalog::memory::{MemoryCatalogProvider, MemorySchemaProvider};
use datafusion::catalog::{SchemaProvider, TableProvider};
use datafusion::execution::context::SessionContext;
use datafusion::sql::parser::DFParserBuilder;
use datafusion::sql::{ResolvedTableReference, TableReference};
use futures::StreamExt;
use futures::stream::BoxStream;
use object_store::ObjectStore;
use url::Url;

use olai_delta_df::{
    ActionsLogProvider, DeltaEngineSessionOptions, DeltaSsaScanConfig, DeltaSsaTableProvider,
    FileMeta, ReconciledLogProvider, SnapshotRef, build_snapshot_from_manifest,
    delta_engine_session,
};

use crate::error::{Error, Result};
use crate::resolve::DiscoveredLog;

/// An opened Delta table ready to register and query: the query [`SessionContext`] (with the
/// vended-credential object store registered and view types forced off) plus the async-native
/// kernel [`SnapshotRef`] the scan provider reads.
pub struct OpenedTable {
    /// The query session — `register_table` and `execute_chunks` run against this.
    pub ctx: SessionContext,
    /// The kernel snapshot, built list-free and engine-free from the discovered manifest.
    pub snapshot: SnapshotRef,
}

/// The Unity Catalog address a query's table reference resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableAddress {
    pub catalog: String,
    pub schema: String,
    pub table: String,
}

impl TableAddress {
    /// `catalog.schema.table`, for messages.
    pub fn full_name(&self) -> String {
        format!("{}.{}.{}", self.catalog, self.schema, self.table)
    }
}

/// Extract the single table reference from `sql`.
///
/// Returns the reference exactly as written (for registration) plus the Unity
/// Catalog [`TableAddress`] it denotes, filling missing qualifiers from the
/// request's optional session defaults. Rejects — as [`Error::Unsupported`] —
/// non-SELECT statements, multi-table queries, and references that stay
/// under-qualified after applying the defaults.
pub fn extract_table(
    sql: &str,
    default_catalog: Option<&str>,
    default_schema: Option<&str>,
) -> Result<(TableReference, TableAddress)> {
    use datafusion::sql::parser::Statement as DfStatement;
    use datafusion::sql::sqlparser::ast::Statement as SqlStatement;
    use datafusion::sql::sqlparser::dialect::GenericDialect;

    let mut statements = DFParserBuilder::new(sql)
        .with_dialect(&GenericDialect {})
        .build()
        .map_err(Error::DataFusion)?
        .parse_statements()
        .map_err(Error::DataFusion)?;
    if statements.len() != 1 {
        return Err(Error::unsupported(format!(
            "expected exactly one SQL statement, got {}",
            statements.len()
        )));
    }
    let statement = statements.pop_front().expect("length checked above");
    match &statement {
        DfStatement::Statement(inner) if matches!(**inner, SqlStatement::Query(_)) => {}
        _ => {
            return Err(Error::unsupported(
                "only SELECT queries run in the browser".to_string(),
            ));
        }
    }

    // `true` = normalize unquoted identifiers to lowercase, matching the
    // session default the query will execute under.
    let (mut references, _) = datafusion::sql::resolve::resolve_table_references(&statement, true)?;
    references.dedup();
    let reference = match references.as_slice() {
        [single] => single.clone(),
        [] => {
            return Err(Error::unsupported("query references no table".to_string()));
        }
        many => {
            return Err(Error::unsupported(format!(
                "query references {} tables; the in-browser engine supports one",
                many.len()
            )));
        }
    };

    let address = table_address_from_reference(&reference, default_catalog, default_schema)?;
    Ok((reference, address))
}

/// Fill a [`TableReference`]'s missing catalog/schema qualifiers from the
/// request's session defaults, yielding the fully-qualified Unity Catalog
/// [`TableAddress`]. Errors — as [`Error::Unsupported`] — when a qualifier is
/// still missing after applying the defaults.
fn table_address_from_reference(
    reference: &TableReference,
    default_catalog: Option<&str>,
    default_schema: Option<&str>,
) -> Result<TableAddress> {
    let qualifier = |explicit: Option<&str>, default: Option<&str>, kind: &str| {
        explicit.or(default).map(str::to_owned).ok_or_else(|| {
            Error::unsupported(format!(
                "table reference `{reference}` has no {kind}; qualify it or set a \
                     session default"
            ))
        })
    };
    Ok(TableAddress {
        catalog: qualifier(reference.catalog(), default_catalog, "catalog")?,
        schema: qualifier(reference.schema(), default_schema, "schema")?,
        table: reference.table().to_owned(),
    })
}

/// Parse a physical table target (`catalog.schema.table`, or a partial name
/// completed by the request's session defaults) into a Unity Catalog
/// [`TableAddress`].
///
/// The log-query seam carries the physical table out-of-band (not in the SQL,
/// which references a fixed logical name), so — unlike [`extract_table`] — the
/// address comes from this string rather than the SQL's `FROM` clause.
pub fn parse_table_address(
    target: &str,
    default_catalog: Option<&str>,
    default_schema: Option<&str>,
) -> Result<TableAddress> {
    let reference = TableReference::parse_str(target);
    table_address_from_reference(&reference, default_catalog, default_schema)
}

/// Open the Delta table at `table_url` from `store` by constructing its snapshot from the
/// pre-discovered `_delta_log` manifest — async-native, list-free, and engine-free.
///
/// The snapshot is pinned to the discovered log's newest version so the query sees exactly the
/// manifest that was discovered. Construction `.await`s the kernel P&M drive (it reads commit /
/// checkpoint files over `store`); it must not be blocked, or a browser worker's event loop would
/// starve and the open would hang (see [`build_snapshot_from_manifest`]).
///
/// `max_catalog_version` must be set for catalog-managed tables (the kernel refuses to build
/// their snapshot without it) and must be `None` for filesystem/external tables (the kernel
/// rejects a catalog version on a non-catalog-managed table). Pass the catalog's latest
/// ratified version, which the server reports only for managed tables.
pub async fn open_table(
    store: Arc<dyn ObjectStore>,
    table_url: &Url,
    log: DiscoveredLog,
    _max_catalog_version: Option<u64>,
) -> Result<OpenedTable> {
    // Build the query session with the vended-credential store registered under the table's
    // authority and Arrow view types forced off (the browser IPC reader can't decode them;
    // mangrove #28). Single-partition / no-repartition mirrors the wasm execution model.
    let ctx = build_query_session(Arc::clone(&store), table_url);

    // Absolute-URL the discovered manifest into kernel `FileMeta`s. `DiscoveredLog.manifest`
    // carries store-relative `Path`s (from HEAD probes); resolve each against the table URL's
    // origin so the kernel log paths match what the session store serves.
    let manifest: Vec<FileMeta> = log
        .manifest
        .iter()
        .map(|meta| {
            let mut url = table_url.clone();
            url.set_path(&format!("/{}", meta.location));
            FileMeta {
                location: url,
                last_modified: 0,
                size: meta.size,
            }
        })
        .collect();

    // Async-native, list-free, engine-free snapshot construction — no `PrimedStore` prefetch and
    // no `InlineExecutor`. `max_catalog_version` is encoded by the pinned `log.version` (the
    // catalog's ratified version drives the manifest's newest commit in `resolve.rs`), so the P&M
    // replay resolves exactly that version.
    let snapshot = build_snapshot_from_manifest(&ctx, table_url, manifest, log.version).await?;

    Ok(OpenedTable { ctx, snapshot })
}

/// Build the single-partition query [`SessionContext`] the preview runs on: the vended-credential
/// `store` registered under `table_url`'s origin, configured for the Delta engine via
/// [`olai_delta_df::delta_engine_session`] with the browser (`::wasm()`) preset:
/// `schema_force_view_types = false` (the browser arrow-js IPC reader can't decode
/// `Utf8View`/`BinaryView`, mangrove #28) and every repartition pass disabled (the wasm runtime is
/// single-threaded). The load-bearing reconciliation config (leaf-pushdown off etc.) the
/// async-native provider's scan plan requires is applied by the same helper, so it stays in sync
/// with the engine automatically instead of being hand-copied here.
///
/// The preview always targets the browser regardless of build target — the native test binary
/// (`tests/native.rs`) exercises this exact browser-shaped session — so we pass `::wasm()`
/// explicitly rather than the build-target-dependent [`DeltaEngineSessionOptions::default`], which
/// would resolve to the native preset (view types on) off `wasm32` and disagree with the provider's
/// `schema_force_view_types = false` scan config.
fn build_query_session(store: Arc<dyn ObjectStore>, table_url: &Url) -> SessionContext {
    delta_engine_session(store, table_url, &DeltaEngineSessionOptions::wasm())
}

/// Which Delta-log surface a log-query registers over the opened snapshot.
///
/// Both variants are engine-free, async-native `TableProvider`s from
/// `olai-delta-df` reconciled from the same kernel snapshot — they differ only
/// in what the reconciled log projects to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogKind {
    /// Surviving scan-file rows after log replay (`ReconciledLogProvider`).
    Reconciled,
    /// The reconciled full action stream (`ActionsLogProvider`).
    Actions,
}

/// The browser scan config shared by every provider registered on the query
/// session.
///
/// Arrow "view" types (Utf8View/BinaryView) are forced off (`schema_force_view_types = false`):
/// arrow-rs 58 / DataFusion materialize string & binary columns as view types by default, but the
/// browser-side apache-arrow IPC reader can't decode them in any published release — its `Type`
/// enum stops at `LargeUtf8 = 20`, so a Utf8View field (id 24) hits no case and throws
/// "Unrecognized type: undefined (24)". Reading them as plain Utf8/Binary keeps the emitted IPC
/// within the JS reader's vocabulary. This is an *unreleased* upstream gap (apache/arrow-js PR #320
/// adds view support on `main`; latest release 21.1.0 lacks it) — drop the override once a release
/// ships the reader (mangrove #28). The provider config records the same intent and asserts it
/// against the session (built with `::wasm()`, view types off) at scan time.
fn wasm_scan_config() -> DeltaSsaScanConfig {
    DeltaSsaScanConfig {
        schema_force_view_types: false,
    }
}

/// Resolve `reference` the same way `ctx.sql` will — against the session's
/// configured default catalog and schema — and return the (lazily created)
/// schema provider it lands in, plus the fully-resolved reference.
///
/// Bare and partial references land in the session's default catalog/schema, so
/// a query's `FROM` clause finds whatever we register wherever it looks.
fn resolve_and_ensure_schema(
    ctx: &SessionContext,
    reference: &TableReference,
) -> Result<(Arc<dyn SchemaProvider>, ResolvedTableReference)> {
    let resolved = {
        let state = ctx.state();
        let options = &state.config().options().catalog;
        reference
            .clone()
            .resolve(&options.default_catalog, &options.default_schema)
    };

    let catalog = match ctx.catalog(resolved.catalog.as_ref()) {
        Some(existing) => existing,
        None => {
            let created = Arc::new(MemoryCatalogProvider::new());
            ctx.register_catalog(resolved.catalog.as_ref(), created.clone());
            created
        }
    };
    let schema = match catalog.schema(resolved.schema.as_ref()) {
        Some(existing) => existing,
        None => {
            let created = Arc::new(MemorySchemaProvider::new());
            catalog.register_schema(resolved.schema.as_ref(), created.clone())?;
            created
        }
    };
    Ok((schema, resolved))
}

/// Register the opened table's data scan under exactly the name `reference`
/// resolves to on `ctx`, creating the catalog/schema providers as needed.
pub fn register_table(
    ctx: &SessionContext,
    opened: &OpenedTable,
    reference: &TableReference,
) -> Result<()> {
    let (schema, resolved) = resolve_and_ensure_schema(ctx, reference)?;
    // Hand the async-native kernel `SnapshotRef` (built list-free + engine-free from the discovered
    // manifest, no `PrimedStore` prefetch) to the sm_plans-driven provider — the full construction
    // + scan path is now async-native.
    let scan = DeltaSsaTableProvider::new(opened.snapshot.clone(), wasm_scan_config())?;
    schema.register_table(resolved.table.to_string(), Arc::new(scan))?;
    Ok(())
}

/// Build the Delta-log `TableProvider` for `kind` over the opened snapshot.
///
/// Both log providers reconcile the same kernel snapshot and take the same
/// `(SnapshotRef, DeltaSsaScanConfig)` constructor — they differ only in what
/// the reconciled log projects to.
fn log_provider(opened: &OpenedTable, kind: LogKind) -> Result<Arc<dyn TableProvider>> {
    Ok(match kind {
        LogKind::Reconciled => Arc::new(ReconciledLogProvider::new(
            opened.snapshot.clone(),
            wasm_scan_config(),
        )?),
        LogKind::Actions => Arc::new(ActionsLogProvider::new(
            opened.snapshot.clone(),
            wasm_scan_config(),
        )?),
    })
}

/// Scan a Delta-log surface (per `kind`) over the opened snapshot, capping at
/// `limit` rows, and stream self-contained IPC chunks.
///
/// Unlike the data path, this drives [`TableProvider::scan`] directly rather
/// than going through `ctx.sql("SELECT * …")`. The log providers emit camelCase
/// columns (`deletionVector`, `metaData`, …); DataFusion's SQL wildcard
/// expansion / analyzer lowercases them and then fails to resolve them against
/// the provider schema (a SQL-surface quirk orthogonal to the provider, which
/// scans correctly). The UI's log SQL is always a bare `SELECT * … LIMIT n`, so
/// projecting all columns with the row cap reproduces it exactly while
/// sidestepping the SQL surface entirely.
pub async fn scan_log_chunks(
    ctx: &SessionContext,
    opened: &OpenedTable,
    kind: LogKind,
    limit: Option<usize>,
) -> Result<BoxStream<'static, Result<IpcChunk>>> {
    let provider = log_provider(opened, kind)?;
    let state = ctx.state();
    // Full projection, no filters — the UI issues `SELECT * … LIMIT n`.
    let plan = provider.scan(&state, None, &[], limit).await?;
    let batches = datafusion::physical_plan::execute_stream(plan, state.task_ctx())?;
    Ok(frame_ipc_chunks(batches))
}

/// One result chunk: a self-contained Arrow IPC stream plus its row count.
#[derive(Debug, Clone)]
pub struct IpcChunk {
    /// Arrow IPC stream bytes: schema message + one record batch + EOS.
    pub ipc: Vec<u8>,
    /// Rows in this chunk.
    pub num_rows: usize,
}

/// Encode `batches` (possibly none — a schema-only chunk) as one self-contained
/// Arrow IPC stream.
fn encode_chunk(schema: &SchemaRef, batches: &[&RecordBatch]) -> Result<Vec<u8>> {
    let mut writer = StreamWriter::try_new(Vec::new(), schema)?;
    for batch in batches {
        writer.write(batch)?;
    }
    Ok(writer.into_inner()?)
}

/// Execute `sql` on `ctx`, capping the result at `limit` rows when given, and
/// stream self-contained IPC chunks.
///
/// Framing per the `open_lakehouse.query.v1` contract: one chunk per record
/// batch, each independently decodable; a query with no rows yields exactly one
/// schema-only chunk.
pub async fn execute_chunks(
    ctx: &SessionContext,
    sql: &str,
    limit: Option<usize>,
) -> Result<BoxStream<'static, Result<IpcChunk>>> {
    let mut df = ctx.sql(sql).await?;
    if let Some(limit) = limit {
        df = df.limit(0, Some(limit))?;
    }
    let batches = df.execute_stream().await?;
    Ok(frame_ipc_chunks(batches))
}

/// Frame a record-batch stream as the self-contained IPC chunks the
/// `open_lakehouse.query.v1` contract requires: one chunk per non-empty batch,
/// each independently decodable; an empty stream yields exactly one schema-only
/// chunk so consumers can still render column headers.
fn frame_ipc_chunks(
    batches: datafusion::execution::SendableRecordBatchStream,
) -> BoxStream<'static, Result<IpcChunk>> {
    let schema = batches.schema();

    struct State {
        batches: datafusion::execution::SendableRecordBatchStream,
        schema: SchemaRef,
        sent_any: bool,
    }
    futures::stream::unfold(
        Some(State {
            batches,
            schema,
            sent_any: false,
        }),
        |state| async move {
            let mut state = state?;
            loop {
                return match state.batches.next().await {
                    // Skip empty batches: a chunk must carry rows (or be the
                    // one terminal schema-only chunk).
                    Some(Ok(batch)) if batch.num_rows() == 0 => continue,
                    Some(Ok(batch)) => {
                        let chunk = encode_chunk(&state.schema, &[&batch]).map(|ipc| IpcChunk {
                            ipc,
                            num_rows: batch.num_rows(),
                        });
                        state.sent_any = true;
                        Some((chunk, Some(state)))
                    }
                    Some(Err(err)) => Some((Err(err.into()), None)),
                    None if !state.sent_any => {
                        let chunk = encode_chunk(&state.schema, &[])
                            .map(|ipc| IpcChunk { ipc, num_rows: 0 });
                        Some((chunk, None))
                    }
                    None => None,
                };
            }
        },
    )
    .boxed()
}

// Native-only: unit tests never run on wasm32 (no test runner without
// wasm-bindgen-test), and the async ones need tokio.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn extracts_backtick_quoted_full_reference() {
        let (reference, address) =
            extract_table("SELECT * FROM `uc`.`sales`.`Orders` LIMIT 100", None, None).unwrap();
        assert_eq!(reference.table(), "Orders");
        assert_eq!(
            address,
            TableAddress {
                catalog: "uc".into(),
                schema: "sales".into(),
                table: "Orders".into(),
            }
        );
    }

    #[test]
    fn fills_defaults_for_bare_and_partial_references() {
        let (_, address) =
            extract_table("SELECT id FROM orders", Some("uc"), Some("sales")).unwrap();
        assert_eq!(address.full_name(), "uc.sales.orders");

        let (_, address) = extract_table("SELECT id FROM sales.orders", Some("uc"), None).unwrap();
        assert_eq!(address.catalog, "uc");
        assert_eq!(address.schema, "sales");

        let err = extract_table("SELECT id FROM orders", None, Some("s")).unwrap_err();
        assert!(err.is_unsupported(), "{err}");
    }

    #[test]
    fn rejects_multi_table_and_non_select() {
        let err =
            extract_table("SELECT * FROM a.b.c JOIN a.b.d ON c.id = d.id", None, None).unwrap_err();
        assert!(err.is_unsupported(), "{err}");

        let err = extract_table("DROP TABLE a.b.c", None, None).unwrap_err();
        assert!(err.is_unsupported(), "{err}");

        // Self-joins of ONE table are fine: a single distinct reference.
        extract_table(
            "SELECT * FROM a.b.c x JOIN a.b.c y ON x.id = y.id",
            None,
            None,
        )
        .unwrap();
    }
}
