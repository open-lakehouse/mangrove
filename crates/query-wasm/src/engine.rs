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
use datafusion::sql::parser::{DFParserBuilder, Statement as DfStatement};
use datafusion::sql::sqlparser::ast::{
    Expr as SqlExpr, FunctionArg, FunctionArgExpr, ObjectName as SqlObjectName,
    ObjectNamePart as SqlObjectNamePart, Query as SqlQuery, SetExpr as SqlSetExpr,
    Statement as SqlStatement, TableFactor, TableFunctionArgs, Value as SqlValue,
};
use datafusion::sql::{ResolvedTableReference, TableReference};
use futures::StreamExt;
use futures::stream::BoxStream;
use object_store::ObjectStore;
use url::Url;

use olai_delta_df::{
    ActionsLogProvider, DeltaEngineSessionOptions, DeltaSsaScanConfig, DeltaSsaTableProvider,
    FileMeta, ReconciledLogProvider, SnapshotRef, build_snapshot_from_manifest,
    delta_engine_session, delta_engine_session_config,
};

use crate::catalog::{ResolvedTables, SessionRouters, UcTableResolver};
use crate::error::{Error, Result};
use crate::log_udtf::{DeltaLogUdtf, build_log_provider};
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

/// The table-function name addressing the reconciled scan-file-row log surface
/// (`ReconciledLogProvider`). `delta_reconciled_log('catalog.schema.table')`.
pub const RECONCILED_LOG_UDTF: &str = "delta_reconciled_log";
/// The table-function name addressing the reconciled full-action log surface
/// (`ActionsLogProvider`). `delta_log_actions('catalog.schema.table')`.
pub const ACTIONS_LOG_UDTF: &str = "delta_log_actions";

/// A `delta_reconciled_log(...)` / `delta_log_actions(...)` call found in a query:
/// which log surface, and the Unity Catalog table its single string-literal
/// argument addresses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogUdtfCall {
    /// Which reconciled-log surface the function projects.
    pub kind: LogKind,
    /// The Unity Catalog table the argument denotes (defaults filled in).
    pub address: TableAddress,
}

/// Extract every `delta_reconciled_log(...)` / `delta_log_actions(...)` call from
/// a parsed statement, resolving each one's single string-literal argument to a
/// [`TableAddress`] (missing qualifiers filled from the session defaults).
///
/// This is a manual sqlparser AST walk because
/// [`resolve_table_references`](datafusion::sql::resolve::resolve_table_references)
/// records only a table function's *name* as a relation — its `RelationVisitor`
/// visits the `ObjectName`, never `TableFactor::Table.args` — so the `'c.s.t'`
/// argument would otherwise be invisible to the resolver. The two log functions
/// are addressed by argument (collision-free) rather than by a reserved logical
/// table name, so the argument is where the physical table lives.
///
/// Rejects — as [`Error::Unsupported`] — a matching call whose argument is not
/// exactly one single-quoted string literal.
pub fn extract_log_udtf_calls(
    statement: &DfStatement,
    default_catalog: Option<&str>,
    default_schema: Option<&str>,
) -> Result<Vec<LogUdtfCall>> {
    let mut calls = Vec::new();
    if let DfStatement::Statement(inner) = statement
        && let SqlStatement::Query(query) = inner.as_ref()
    {
        walk_query_for_udtfs(query, default_catalog, default_schema, &mut calls)?;
    }
    Ok(calls)
}

/// Recognize a `TableFactor::Table` whose name is one of the log UDTFs (matched
/// case-insensitively on the final name part) *and* carries function arguments,
/// returning the [`LogKind`] it maps to. Non-UDTF tables and same-named plain
/// tables (no `args`) return `None`.
fn log_udtf_kind(name: &SqlObjectName, args: Option<&TableFunctionArgs>) -> Option<LogKind> {
    // A plain table reference (`FROM delta_log_actions`) parses with `args:
    // None`; only a call (`delta_log_actions(...)`) carries `Some`.
    args?;
    let last = name.0.last()?;
    let ident = match last {
        SqlObjectNamePart::Identifier(ident) => ident.value.as_str(),
        // A function-form object-name part (`schema.fn()`) is not how these
        // UDTFs are written; ignore it.
        _ => return None,
    };
    if ident.eq_ignore_ascii_case(RECONCILED_LOG_UDTF) {
        Some(LogKind::Reconciled)
    } else if ident.eq_ignore_ascii_case(ACTIONS_LOG_UDTF) {
        Some(LogKind::Actions)
    } else {
        None
    }
}

/// Pull the single string-literal argument out of a log-UDTF call, erroring on
/// wrong arity or a non-literal argument.
fn single_string_literal_arg(name: &str, args: &TableFunctionArgs) -> Result<String> {
    if args.args.len() != 1 {
        return Err(Error::unsupported(format!(
            "{name}(...) takes exactly one argument (a 'catalog.schema.table' string), got {}",
            args.args.len()
        )));
    }
    match &args.args[0] {
        FunctionArg::Unnamed(FunctionArgExpr::Expr(SqlExpr::Value(value))) => match &value.value {
            SqlValue::SingleQuotedString(s) => Ok(s.clone()),
            other => Err(Error::unsupported(format!(
                "{name}(...) argument must be a single-quoted string literal, got {other:?}"
            ))),
        },
        other => Err(Error::unsupported(format!(
            "{name}(...) argument must be a single-quoted string literal, got {other}"
        ))),
    }
}

/// Recurse a query body (CTEs, set operations, subqueries, joins) collecting log
/// UDTF calls from every `FROM` clause.
fn walk_query_for_udtfs(
    query: &SqlQuery,
    default_catalog: Option<&str>,
    default_schema: Option<&str>,
    out: &mut Vec<LogUdtfCall>,
) -> Result<()> {
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            walk_query_for_udtfs(&cte.query, default_catalog, default_schema, out)?;
        }
    }
    walk_set_expr(&query.body, default_catalog, default_schema, out)
}

fn walk_set_expr(
    set_expr: &SqlSetExpr,
    default_catalog: Option<&str>,
    default_schema: Option<&str>,
    out: &mut Vec<LogUdtfCall>,
) -> Result<()> {
    match set_expr {
        SqlSetExpr::Select(select) => {
            for twj in &select.from {
                walk_table_factor(&twj.relation, default_catalog, default_schema, out)?;
                for join in &twj.joins {
                    walk_table_factor(&join.relation, default_catalog, default_schema, out)?;
                }
            }
            Ok(())
        }
        SqlSetExpr::Query(query) => {
            walk_query_for_udtfs(query, default_catalog, default_schema, out)
        }
        SqlSetExpr::SetOperation { left, right, .. } => {
            walk_set_expr(left, default_catalog, default_schema, out)?;
            walk_set_expr(right, default_catalog, default_schema, out)
        }
        // Values/Insert/Update/Delete/Merge/Table bodies carry no FROM-clause
        // table functions the preview cares about.
        _ => Ok(()),
    }
}

fn walk_table_factor(
    factor: &TableFactor,
    default_catalog: Option<&str>,
    default_schema: Option<&str>,
    out: &mut Vec<LogUdtfCall>,
) -> Result<()> {
    match factor {
        TableFactor::Table { name, args, .. } => {
            if let Some(kind) = log_udtf_kind(name, args.as_ref()) {
                let fn_name = name.to_string();
                let target = single_string_literal_arg(&fn_name, args.as_ref().expect("Some"))?;
                let address = parse_table_address(&target, default_catalog, default_schema)?;
                out.push(LogUdtfCall { kind, address });
            }
            Ok(())
        }
        TableFactor::Derived { subquery, .. } => {
            walk_query_for_udtfs(subquery, default_catalog, default_schema, out)
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => {
            walk_table_factor(
                &table_with_joins.relation,
                default_catalog,
                default_schema,
                out,
            )?;
            for join in &table_with_joins.joins {
                walk_table_factor(&join.relation, default_catalog, default_schema, out)?;
            }
            Ok(())
        }
        // Function/TableFunction/Pivot/Unpivot/JsonTable etc. are not how the log
        // UDTFs are addressed here; ignore.
        _ => Ok(()),
    }
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
    max_catalog_version: Option<u64>,
) -> Result<OpenedTable> {
    // Build the query session with the vended-credential store registered under the table's
    // authority and Arrow view types forced off (the browser IPC reader can't decode them;
    // mangrove #28). Single-partition / no-repartition mirrors the wasm execution model.
    let ctx = build_query_session(Arc::clone(&store), table_url);
    let snapshot = open_snapshot(&ctx, table_url, log, max_catalog_version).await?;
    Ok(OpenedTable { ctx, snapshot })
}

/// Build the kernel [`SnapshotRef`] for the table at `table_url` from a
/// pre-discovered `_delta_log` manifest, driven against an **existing** session.
///
/// The session-decoupled core of [`open_table`]: it does NOT create or register a
/// store — the caller's `ctx` must already have the table's object store
/// registered under `table_url`'s `scheme://host` (the unified resolve pass does
/// this via a [`RoutingObjectStore`](olai_delta_df::RoutingObjectStore); the
/// single-table [`open_table`] path does it via [`build_query_session`]). This is
/// what lets many tables share one session: each table's snapshot is built here
/// against the same `ctx`, and the routed store serves each table's reads by path
/// prefix.
///
/// `max_catalog_version` must be set for catalog-managed tables (the kernel
/// refuses to build their snapshot without it) and must be `None` for
/// filesystem/external tables. It is encoded by the pinned `log.version` (the
/// catalog's ratified version drives the manifest's newest commit in
/// `resolve.rs`), so the P&M replay resolves exactly that version.
///
/// Async-native: it `.await`s the kernel P&M drive (reading commit / checkpoint
/// files over the session's store); it must not be blocked, or a browser worker's
/// event loop would starve and the open would hang.
pub async fn open_snapshot(
    ctx: &SessionContext,
    table_url: &Url,
    log: DiscoveredLog,
    _max_catalog_version: Option<u64>,
) -> Result<SnapshotRef> {
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
    // no `InlineExecutor`.
    Ok(build_snapshot_from_manifest(ctx, table_url, manifest, log.version).await?)
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
pub fn wasm_scan_config() -> DeltaSsaScanConfig {
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
    scan_provider_chunks(ctx, provider, limit).await
}

/// Scan `provider` with a full projection and no filters, capping at `limit`
/// rows, and stream self-contained IPC chunks.
///
/// The provider-level generalization of [`scan_log_chunks`]: it drives
/// [`TableProvider::scan`] directly rather than `ctx.sql("SELECT * …")`, so the
/// log providers' camelCase columns (`deletionVector`, `metaData`, …) survive —
/// DataFusion's SQL wildcard expansion / analyzer lowercases them and then fails
/// to resolve them against the provider schema (a SQL-surface quirk orthogonal to
/// the provider, which scans correctly). The UI's log SQL is always a bare
/// `SELECT * … LIMIT n`, so a full projection with the row cap reproduces it
/// exactly while sidestepping the SQL surface. Used by the unified entrypoint,
/// where the provider comes from a pre-resolved log UDTF.
pub async fn scan_provider_chunks(
    ctx: &SessionContext,
    provider: Arc<dyn TableProvider>,
    limit: Option<usize>,
) -> Result<BoxStream<'static, Result<IpcChunk>>> {
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

/// Build the browser-shaped query [`SessionContext`] the unified pass resolves
/// tables onto: the Delta engine's `::wasm()` preset config (view types off,
/// repartition off) on a fresh [`RuntimeEnv`] with **no** object store yet — the
/// resolve pass registers a routed store per origin. Contrast
/// [`build_query_session`], which registers a single store for the one table it
/// opens.
fn build_unified_session() -> SessionContext {
    use datafusion::execution::runtime_env::RuntimeEnv;
    SessionContext::new_with_config_rt(
        delta_engine_session_config(&DeltaEngineSessionOptions::wasm()),
        Arc::new(RuntimeEnv::default()),
    )
}

/// Resolve a query's tables through `resolver`, register them (data table +
/// the two log UDTFs) on a fresh browser session, and stream the result as
/// contract-framed IPC chunks — the single pipeline behind both `runQuery` and
/// `runLogQuery`.
///
/// The unified replacement for the old inline `run_query_inner` /
/// `run_log_query_inner` pipelines. Steps (handover §6):
/// 1. Parse SQL once; reject non-SELECT / multi-statement.
/// 2. `resolve_table_references` → data-table refs; [`extract_log_udtf_calls`] →
///    log-UDTF addresses; subtract the UDTF *names* from the data refs.
/// 3. Enforce the **single-table** guard over the union of real tables addressed
///    (one data ref *or* one log-UDTF arg) — multi-table is a follow-up the
///    structure already supports.
/// 4. Async pre-resolution: `resolver.resolve` each addressed table, registering
///    its routed store and inserting into a [`ResolvedTables`] map.
/// 5. Register into the session: a data ref → [`DeltaSsaTableProvider`] under the
///    resolved name; the two log UDTFs sharing the map.
/// 6. Execute **without** depending on SQL wildcard resolution: a data query via
///    `ctx.sql` (normal-named columns), a log query via a direct provider scan
///    ([`scan_provider_chunks`]) — the log providers' camelCase columns don't
///    survive SQL wildcard expansion.
///
/// Returns the framed chunk stream and the pinned table version the query read.
/// The stream owns its execution state, so it outlives the dropped session.
pub async fn run_unified(
    resolver: &impl UcTableResolver,
    sql: &str,
    limit: Option<usize>,
    default_catalog: Option<&str>,
    default_schema: Option<&str>,
) -> Result<(BoxStream<'static, Result<IpcChunk>>, u64)> {
    use datafusion::sql::sqlparser::dialect::GenericDialect;

    // 1. Parse once; keep the single-statement + SELECT guards.
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

    // 2. Data-table refs (UDTF *names* appear here as relations; drop them) and
    //    log-UDTF calls (addressed by argument).
    let (mut references, _) = datafusion::sql::resolve::resolve_table_references(&statement, true)?;
    references.dedup();
    references.retain(|r| !is_log_udtf_name(r));
    let log_calls = extract_log_udtf_calls(&statement, default_catalog, default_schema)?;

    // 3. Single-table guard over the union of addressed tables.
    let data_ref = match references.as_slice() {
        [] => None,
        [single] => Some(single.clone()),
        many => {
            return Err(Error::unsupported(format!(
                "query references {} tables; the in-browser engine supports one",
                many.len()
            )));
        }
    };
    let total_tables = data_ref.iter().count() + log_calls.len();
    if total_tables == 0 {
        return Err(Error::unsupported("query references no table".to_string()));
    }
    if total_tables > 1 {
        return Err(Error::unsupported(format!(
            "query references {total_tables} tables (data + log); the in-browser engine supports one"
        )));
    }

    // 4. Build the session and pre-resolve every addressed table.
    let ctx = build_unified_session();
    let routers = SessionRouters::new();
    let resolved = ResolvedTables::new();

    // A data ref carries only a `TableReference`; derive its UC address the same
    // way `extract_table` does so the resolver and registration agree.
    let data_addr = match &data_ref {
        Some(reference) => Some((
            reference.clone(),
            table_address_from_reference(reference, default_catalog, default_schema)?,
        )),
        None => None,
    };

    let mut table_version = 0u64;
    if let Some((_, addr)) = &data_addr {
        let rt = resolver.resolve(&ctx, &routers, addr).await?;
        table_version = rt.table_version;
        resolved.insert(addr, rt);
    }
    for call in &log_calls {
        let rt = resolver.resolve(&ctx, &routers, &call.address).await?;
        table_version = rt.table_version;
        resolved.insert(&call.address, rt);
    }

    // 5. Register the data table and both log UDTFs (sharing the resolved map).
    if let Some((reference, addr)) = &data_addr {
        let rt = resolved
            .get(addr)
            .expect("data table was just inserted into the resolved map");
        register_data_table(&ctx, &rt.snapshot, reference)?;
    }
    ctx.register_udtf(
        RECONCILED_LOG_UDTF,
        Arc::new(DeltaLogUdtf::new(LogKind::Reconciled, resolved.clone())),
    );
    ctx.register_udtf(
        ACTIONS_LOG_UDTF,
        Arc::new(DeltaLogUdtf::new(LogKind::Actions, resolved.clone())),
    );

    // 6. Execute — data via SQL, log via a direct provider scan (bypassing the
    //    SQL wildcard analyzer that mangles the log providers' camelCase columns).
    let chunks = if let Some(call) = log_calls.first() {
        let rt = resolved
            .get(&call.address)
            .expect("log table was just inserted into the resolved map");
        let provider = build_log_provider(call.kind, rt.snapshot, wasm_scan_config())?;
        scan_provider_chunks(&ctx, provider, limit).await?
    } else {
        execute_chunks(&ctx, sql, limit).await?
    };
    Ok((chunks, table_version))
}

/// True when `reference`'s bare table name is one of the log UDTF names — so it
/// can be dropped from the data-table refs `resolve_table_references` surfaces
/// (a table function's name is recorded there as if it were a relation).
fn is_log_udtf_name(reference: &TableReference) -> bool {
    let name = reference.table();
    name.eq_ignore_ascii_case(RECONCILED_LOG_UDTF) || name.eq_ignore_ascii_case(ACTIONS_LOG_UDTF)
}

/// Register a [`DeltaSsaTableProvider`] over `snapshot` under exactly the name
/// `reference` resolves to on `ctx`. The snapshot-taking generalization of
/// [`register_table`] for the unified pass, which holds a bare [`SnapshotRef`]
/// rather than an [`OpenedTable`].
fn register_data_table(
    ctx: &SessionContext,
    snapshot: &SnapshotRef,
    reference: &TableReference,
) -> Result<()> {
    let (schema, resolved) = resolve_and_ensure_schema(ctx, reference)?;
    let scan = DeltaSsaTableProvider::new(snapshot.clone(), wasm_scan_config())?;
    schema.register_table(resolved.table.to_string(), Arc::new(scan))?;
    Ok(())
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

    /// Parse `sql` to a single DF statement for the UDTF-extraction tests.
    fn parse_one(sql: &str) -> DfStatement {
        use datafusion::sql::sqlparser::dialect::GenericDialect;
        let mut statements = DFParserBuilder::new(sql)
            .with_dialect(&GenericDialect {})
            .build()
            .unwrap()
            .parse_statements()
            .unwrap();
        assert_eq!(statements.len(), 1);
        statements.pop_front().unwrap()
    }

    #[test]
    fn extracts_full_and_partial_udtf_calls() {
        // Full three-part argument, both surfaces.
        let calls = extract_log_udtf_calls(
            &parse_one("SELECT * FROM delta_log_actions('a.b.c')"),
            None,
            None,
        )
        .unwrap();
        assert_eq!(
            calls,
            vec![LogUdtfCall {
                kind: LogKind::Actions,
                address: TableAddress {
                    catalog: "a".into(),
                    schema: "b".into(),
                    table: "c".into(),
                },
            }]
        );

        let calls = extract_log_udtf_calls(
            &parse_one("SELECT * FROM delta_reconciled_log('a.b.c') LIMIT 10"),
            None,
            None,
        )
        .unwrap();
        assert_eq!(calls[0].kind, LogKind::Reconciled);

        // Case-insensitive function name; bare/partial argument completed by
        // session defaults.
        let calls = extract_log_udtf_calls(
            &parse_one("SELECT * FROM Delta_Log_Actions('orders')"),
            Some("uc"),
            Some("sales"),
        )
        .unwrap();
        assert_eq!(calls[0].address.full_name(), "uc.sales.orders");
    }

    #[test]
    fn plain_table_named_like_a_udtf_is_not_a_call() {
        // No arguments → a regular table reference, not a UDTF call.
        let calls =
            extract_log_udtf_calls(&parse_one("SELECT * FROM delta_log_actions"), None, None)
                .unwrap();
        assert!(calls.is_empty(), "bare name is a table, not a UDTF call");
    }

    #[test]
    fn rejects_non_literal_and_wrong_arity_udtf_args() {
        // Wrong arity.
        let err = extract_log_udtf_calls(
            &parse_one("SELECT * FROM delta_log_actions('a.b.c', 'x')"),
            None,
            None,
        )
        .unwrap_err();
        assert!(err.is_unsupported(), "{err}");

        // Non-literal argument (a column reference).
        let err = extract_log_udtf_calls(
            &parse_one("SELECT * FROM delta_reconciled_log(some_col)"),
            None,
            None,
        )
        .unwrap_err();
        assert!(err.is_unsupported(), "{err}");

        // Zero arguments.
        let err =
            extract_log_udtf_calls(&parse_one("SELECT * FROM delta_log_actions()"), None, None)
                .unwrap_err();
        assert!(err.is_unsupported(), "{err}");
    }
}
