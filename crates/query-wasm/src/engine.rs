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
use datafusion::execution::context::SessionContext;
use datafusion::execution::runtime_env::RuntimeEnv;
use datafusion::prelude::SessionConfig;
use datafusion::sql::TableReference;
use datafusion::sql::parser::DFParserBuilder;
use futures::StreamExt;
use futures::stream::BoxStream;
use object_store::ObjectStore;
use url::Url;

use delta_df_provider::{
    DeltaSsaScanConfig, DeltaSsaTableProvider, FileMeta, SnapshotRef, build_snapshot_from_manifest,
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

    let qualifier = |explicit: Option<&str>, default: Option<&str>, kind: &str| {
        explicit.or(default).map(str::to_owned).ok_or_else(|| {
            Error::unsupported(format!(
                "table reference `{reference}` has no {kind}; qualify it or set a \
                     session default"
            ))
        })
    };
    let address = TableAddress {
        catalog: qualifier(reference.catalog(), default_catalog, "catalog")?,
        schema: qualifier(reference.schema(), default_schema, "schema")?,
        table: reference.table().to_owned(),
    };
    Ok((reference, address))
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
/// `store` registered under `table_url`'s origin, and `parquet.schema_force_view_types = false` so
/// the physical reader emits plain `Utf8`/`Binary` (the browser arrow-js IPC reader can't decode
/// `Utf8View`/`BinaryView`; mangrove #28). Mirrors the former `deltalake_wasm::session` shape minus
/// the delta-rs-specific planner (the async-native provider needs none).
fn build_query_session(store: Arc<dyn ObjectStore>, table_url: &Url) -> SessionContext {
    let mut config = SessionConfig::new()
        .with_target_partitions(1)
        .with_round_robin_repartition(false)
        .with_repartition_joins(false)
        .with_repartition_aggregations(false)
        .with_repartition_windows(false)
        .with_repartition_sorts(false)
        .with_repartition_file_scans(false);
    config
        .options_mut()
        .execution
        .parquet
        .schema_force_view_types = false;
    // The compiled Delta scan `LogicalPlan` is optimized against THIS session (the provider plans
    // it via `session.create_physical_plan`), so this session must carry the same load-bearing
    // override the scan executor's own session sets (see `DataFusionExecutor::replay_session_config`
    // / `from_session`): DataFusion's leaf-expression pushdown inlines the FSR replay's
    // `named_struct` build into every Filter leaf and produces an ambiguous `scan.add`/`add` schema,
    // failing `push_down_leaf_projections`. Commit-only previews don't hit the ambiguous shape, but
    // a classic-checkpointed table's scan plan does — without this the checkpointed preview fails
    // (apache/datafusion#20432).
    config
        .options_mut()
        .optimizer
        .enable_leaf_expression_pushdown = false;
    let ctx = SessionContext::new_with_config_rt(config, Arc::new(RuntimeEnv::default()));

    // Register the store under the table URL's origin (scheme://authority/), matching how the
    // kernel/DataFusion resolve object stores by `ObjectStoreUrl` authority.
    let mut base = table_url.clone();
    base.set_path("/");
    base.set_query(None);
    base.set_fragment(None);
    ctx.runtime_env().register_object_store(&base, store);
    ctx
}

/// Register the opened table's scan under exactly the name `reference` resolves
/// to on `ctx`, creating the catalog/schema providers as needed.
///
/// Bare and partial references land in the session's default catalog/schema —
/// the same resolution `ctx.sql` applies — so the query's `FROM` clause finds
/// the scan wherever it looks.
pub fn register_table(
    ctx: &SessionContext,
    opened: &OpenedTable,
    reference: &TableReference,
) -> Result<()> {
    // Resolve bare/partial references the same way `ctx.sql` will: against the
    // session's configured default catalog and schema.
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

    // Arrow "view" types (Utf8View/BinaryView) are forced off on the query session at build time
    // (see `build_query_session`): arrow-rs 58 / DataFusion materialize string & binary columns as
    // view types by default, but the browser-side apache-arrow IPC reader can't decode them in any
    // published release — its `Type` enum stops at `LargeUtf8 = 20`, so a Utf8View field (id 24)
    // hits no case and throws "Unrecognized type: undefined (24)". Reading them as plain
    // Utf8/Binary keeps the emitted IPC within the JS reader's vocabulary. This is an *unreleased*
    // upstream gap (apache/arrow-js PR #320 adds view support on `main`; latest release 21.1.0
    // lacks it) — drop the override once a release ships the reader (mangrove #28). The provider
    // config records the same intent and asserts it against the session at scan time.
    let scan_config = DeltaSsaScanConfig {
        schema_force_view_types: false,
    };
    // Hand the async-native kernel `SnapshotRef` (built list-free + engine-free from the discovered
    // manifest, no `PrimedStore` prefetch) to the sm_plans-driven provider — the full construction
    // + scan path is now async-native.
    let scan = DeltaSsaTableProvider::new(opened.snapshot.clone(), scan_config)?;
    schema.register_table(resolved.table.to_string(), Arc::new(scan))?;
    Ok(())
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
    let schema = batches.schema();

    struct State {
        batches: datafusion::execution::SendableRecordBatchStream,
        schema: SchemaRef,
        sent_any: bool,
    }
    let stream = futures::stream::unfold(
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
    );
    Ok(stream.boxed())
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
