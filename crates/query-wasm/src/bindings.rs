//! The wasm-bindgen surface: [`UcQueryEngine`], the browser-facing entry point
//! implementing the `open_lakehouse.query.v1` runner contract.
//!
//! `runQuery(sql, opts, on_batch)` performs the whole preview pipeline —
//! extract the table reference from the SQL, resolve it through the UC REST
//! API, map the vended credential to a fetch-backed store, discover and prime
//! the `_delta_log`, open the table, and stream self-contained Arrow IPC
//! chunks to `on_batch(Uint8Array, numRows)`.
//!
//! Thrown errors are `js_sys::Error`s with a machine-readable `code` property:
//! `"UNSUPPORTED"` (fall back to another runner), `"NETWORK"` (direct storage
//! fetch blocked — CORS or connectivity; also fallback-worthy), or `"FAILED"`.
//!
//! Host this in a Web Worker: the kernel's inline-executor bursts run
//! synchronously against primed data and would jank the main thread.

use futures::TryStreamExt;
use js_sys::{Function, Uint8Array};
use url::Url;
use wasm_bindgen::prelude::*;

use unitycatalog_object_store::UnityObjectStoreFactory;

use crate::catalog::UcRestResolver;
use crate::engine::{ACTIONS_LOG_UDTF, LogKind, RECONCILED_LOG_UDTF, run_unified};
use crate::error::Error;
use crate::files::path::VolumePath;

// Named uniquely: `deltalake-wasm` (also a cdylib-capable dependency) exports
// its own `#[wasm_bindgen(start)] fn init`, and identically-named start
// symbols collide at link time.
#[wasm_bindgen(start)]
fn query_wasm_init() {
    console_error_panic_hook::set_once();
}

/// Machine-readable failure classes, exposed as the `code` property on thrown
/// errors so the TS side can decide whether to fall back.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ErrorCode {
    Unsupported,
    Network,
    Failed,
}

impl ErrorCode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Unsupported => "UNSUPPORTED",
            Self::Network => "NETWORK",
            Self::Failed => "FAILED",
        }
    }
}

/// Classify an engine error for the JS boundary.
///
/// Deletion-vector and zstd/brotli failures surface from deep inside the scan
/// as message text rather than typed variants — match them here (the loud-fail
/// guards delta-rs put in place) so they trigger fallback instead of an error
/// panel.
fn classify(err: &Error) -> ErrorCode {
    if err.is_unsupported() {
        return ErrorCode::Unsupported;
    }
    let message = err.to_string().to_ascii_lowercase();
    if message.contains("deletion vector") || message.contains("zstd") || message.contains("brotli")
    {
        return ErrorCode::Unsupported;
    }
    if message.contains("network/cors") {
        return ErrorCode::Network;
    }
    ErrorCode::Failed
}

fn js_error(err: Error) -> JsValue {
    let js = js_sys::Error::new(&err.to_string());
    // Reflect::set on a js_sys::Error cannot fail; ignore the Result.
    let _ = js_sys::Reflect::set(
        &js,
        &JsValue::from_str("code"),
        &JsValue::from_str(classify(&err).as_str()),
    );
    js.into()
}

/// Options accepted by the [`UcQueryEngine`] constructor.
#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EngineOptions {
    /// Bearer token for the UC REST API. Same-origin cookies flow regardless.
    auth_token: Option<String>,
}

/// Options accepted by `runQuery`, mirroring `RunQueryRequest`.
#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RunOptions {
    /// Row cap; the runner's default applies when omitted.
    limit: Option<u32>,
    /// Session default namespace for bare table names in the SQL.
    catalog: Option<String>,
    schema: Option<String>,
}

/// Which reconciled-log surface `runLogQuery` scans (wire form of [`LogKind`]).
#[derive(serde::Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum LogKindWire {
    /// Surviving scan-file rows after replay.
    #[default]
    Reconciled,
    /// The reconciled full action stream.
    Actions,
}

impl From<LogKindWire> for LogKind {
    fn from(wire: LogKindWire) -> Self {
        match wire {
            LogKindWire::Reconciled => LogKind::Reconciled,
            LogKindWire::Actions => LogKind::Actions,
        }
    }
}

/// Options accepted by `runLogQuery`.
///
/// Unlike [`RunOptions`], the physical table (`target`) is carried out-of-band
/// rather than parsed from the SQL — the log-query SQL references a fixed
/// logical table name (`reconciled_log` / `action_log`), so the UC address must
/// come from `target`.
#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LogRunOptions {
    /// Row cap; the runner's default applies when omitted.
    limit: Option<u32>,
    /// Session defaults completing a partial `target`.
    catalog: Option<String>,
    schema: Option<String>,
    /// The physical table whose log to scan (`catalog.schema.table`).
    target: String,
    /// Which log surface to project.
    #[serde(default)]
    kind: LogKindWire,
}

/// Result summary returned by `runQuery`.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RunStats {
    chunks: u32,
    rows: u64,
    /// The pinned table version the preview read.
    table_version: u64,
}

/// The in-browser Unity Catalog query engine.
#[wasm_bindgen]
pub struct UcQueryEngine {
    base_url: Url,
    auth_token: Option<String>,
}

#[wasm_bindgen]
impl UcQueryEngine {
    /// Create an engine talking to the UC REST API at `base_url`
    /// (e.g. `${origin}/api/2.1/unity-catalog`). `opts`: `{ authToken? }`.
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: String, opts: JsValue) -> Result<UcQueryEngine, JsValue> {
        let opts: EngineOptions = if opts.is_undefined() || opts.is_null() {
            EngineOptions::default()
        } else {
            serde_wasm_bindgen::from_value(opts)
                .map_err(|e| js_error(Error::InvalidResponse(format!("engine options: {e}"))))?
        };
        let base_url = Url::parse(&base_url)
            .map_err(|e| js_error(Error::InvalidUrl(format!("unity catalog base url: {e}"))))?;
        Ok(UcQueryEngine {
            base_url,
            auth_token: opts.auth_token,
        })
    }

    /// Execute `sql` (one SELECT over one UC table), calling
    /// `on_batch(Uint8Array, numRows)` per self-contained Arrow IPC chunk.
    ///
    /// `opts`: `{ limit?, catalog?, schema? }` per the runner contract.
    #[wasm_bindgen(js_name = runQuery)]
    pub async fn run_query(
        &self,
        sql: String,
        opts: JsValue,
        on_batch: Function,
    ) -> Result<JsValue, JsValue> {
        let opts: RunOptions = if opts.is_undefined() || opts.is_null() {
            RunOptions::default()
        } else {
            serde_wasm_bindgen::from_value(opts)
                .map_err(|e| js_error(Error::InvalidResponse(format!("run options: {e}"))))?
        };
        self.run_query_inner(&sql, opts, &on_batch)
            .await
            .map_err(js_error)
    }

    /// Execute a reconciled-Delta-log query against the table named by
    /// `opts.target`, calling `on_batch(Uint8Array, numRows)` per
    /// self-contained Arrow IPC chunk.
    ///
    /// `opts`: `{ limit?, catalog?, schema?, target, kind }`. `sql` references
    /// the fixed logical table name the provider is registered under
    /// (`reconciled_log` or `action_log`, per `kind`); the physical table comes
    /// from `target`, not the SQL.
    #[wasm_bindgen(js_name = runLogQuery)]
    pub async fn run_log_query(
        &self,
        sql: String,
        opts: JsValue,
        on_batch: Function,
    ) -> Result<JsValue, JsValue> {
        let opts: LogRunOptions = if opts.is_undefined() || opts.is_null() {
            return Err(js_error(Error::InvalidResponse(
                "runLogQuery requires options with a `target`".to_string(),
            )));
        } else {
            serde_wasm_bindgen::from_value(opts)
                .map_err(|e| js_error(Error::InvalidResponse(format!("log run options: {e}"))))?
        };
        self.run_log_query_inner(&sql, opts, &on_batch)
            .await
            .map_err(js_error)
    }
}

impl UcQueryEngine {
    async fn run_query_inner(
        &self,
        sql: &str,
        opts: RunOptions,
        on_batch: &Function,
    ) -> Result<JsValue, Error> {
        // The unified resolve pass parses the SQL, resolves each referenced table
        // (data or log UDTF) through Unity Catalog, registers the routed store +
        // providers, and returns the framed chunk stream + pinned version.
        let resolver = UcRestResolver::new(self.base_url.clone(), self.auth_token.clone());
        let (chunks, table_version) = run_unified(
            &resolver,
            sql,
            opts.limit.map(|l| l as usize),
            opts.catalog.as_deref(),
            opts.schema.as_deref(),
        )
        .await?;
        stream_to_callback(chunks, table_version, on_batch).await
    }

    async fn run_log_query_inner(
        &self,
        _sql: &str,
        opts: LogRunOptions,
        on_batch: &Function,
    ) -> Result<JsValue, Error> {
        // The physical table rides on `target`, not the SQL (which references a
        // fixed logical name). Address the log surface collision-free via the
        // matching UDTF and run it through the same unified pass — which scans the
        // log provider directly (the log providers' camelCase columns don't
        // survive DataFusion's SQL wildcard expansion), so `_sql` is unused beyond
        // the row cap (already parsed into `opts.limit`).
        let kind: LogKind = opts.kind.into();
        let udtf = match kind {
            LogKind::Reconciled => RECONCILED_LOG_UDTF,
            LogKind::Actions => ACTIONS_LOG_UDTF,
        };
        // Single-quote the target as the UDTF's string literal; escape any quote
        // in the identifier so the synthesized SQL parses.
        let escaped = opts.target.replace('\'', "''");
        let log_sql = format!("SELECT * FROM {udtf}('{escaped}')");

        let resolver = UcRestResolver::new(self.base_url.clone(), self.auth_token.clone());
        let (chunks, table_version) = run_unified(
            &resolver,
            &log_sql,
            opts.limit.map(|l| l as usize),
            opts.catalog.as_deref(),
            opts.schema.as_deref(),
        )
        .await?;
        stream_to_callback(chunks, table_version, on_batch).await
    }
}

/// Drain a contract-framed IPC chunk stream to `on_batch`, returning the run
/// summary. Shared by the data and log paths.
async fn stream_to_callback(
    mut chunks: futures::stream::BoxStream<'static, Result<crate::engine::IpcChunk, Error>>,
    table_version: u64,
    on_batch: &Function,
) -> Result<JsValue, Error> {
    let mut stats = RunStats {
        chunks: 0,
        rows: 0,
        table_version,
    };
    while let Some(chunk) = chunks.try_next().await? {
        stats.chunks += 1;
        stats.rows += chunk.num_rows as u64;
        let bytes = Uint8Array::from(chunk.ipc.as_slice());
        on_batch
            .call2(
                &JsValue::NULL,
                &bytes.into(),
                &JsValue::from_f64(chunk.num_rows as f64),
            )
            .map_err(|e| Error::InvalidResponse(format!("on_batch callback threw: {e:?}")))?;
    }
    serde_wasm_bindgen::to_value(&stats).map_err(|e| Error::InvalidResponse(format!("stats: {e}")))
}

// =====================================================================
// Files: the read-only volume file-browser surface (`UcFilesEngine`).
// =====================================================================

/// Options accepted by `listDirectory`, mirroring `ListDirectoryRequest`.
#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ListDirectoryOptions {
    /// Page size; the provider applies its own cap when omitted.
    max_results: Option<u32>,
    /// Opaque continuation token from a previous page's `nextPageToken`.
    page_token: Option<String>,
}

/// Options accepted by `readFile`, mirroring `ReadFileRequest`.
#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReadFileOptions {
    /// Byte offset to start from (defaults to 0).
    offset: Option<u64>,
    /// Number of bytes to read (defaults to the rest of the file).
    length: Option<u64>,
}

/// Summary returned by `readFile`.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadStats {
    bytes_read: u64,
}

/// The in-browser Unity Catalog **volume files** engine: read-only directory
/// listing, ranged reads, and stat over UC-vended volume credentials.
///
/// Errors are `js_sys::Error`s with the same `code` contract as
/// [`UcQueryEngine`] (`UNSUPPORTED` / `NETWORK` / `FAILED`). Azure + GCP are
/// supported; AWS / R2 surface `UNSUPPORTED`.
#[wasm_bindgen]
pub struct UcFilesEngine {
    base_url: Url,
    auth_token: Option<String>,
}

#[wasm_bindgen]
impl UcFilesEngine {
    /// Create an engine talking to the UC REST API at `base_url`
    /// (e.g. `${origin}/api/2.1/unity-catalog`). `opts`: `{ authToken? }`.
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: String, opts: JsValue) -> Result<UcFilesEngine, JsValue> {
        let opts: EngineOptions = if opts.is_undefined() || opts.is_null() {
            EngineOptions::default()
        } else {
            serde_wasm_bindgen::from_value(opts)
                .map_err(|e| js_error(Error::InvalidResponse(format!("engine options: {e}"))))?
        };
        let base_url = Url::parse(&base_url)
            .map_err(|e| js_error(Error::InvalidUrl(format!("unity catalog base url: {e}"))))?;
        Ok(UcFilesEngine {
            base_url,
            auth_token: opts.auth_token,
        })
    }

    /// List one bounded page of a directory's immediate children.
    ///
    /// `path`: a canonical `/Volumes/<c>/<s>/<v>[/<rest>]` path. `opts`:
    /// `{ maxResults?, pageToken? }`. Returns
    /// `{ entries: [{ path, isDirectory, fileSize, lastModified }], nextPageToken? }`.
    #[wasm_bindgen(js_name = listDirectory)]
    pub async fn list_directory(&self, path: String, opts: JsValue) -> Result<JsValue, JsValue> {
        let opts: ListDirectoryOptions = if opts.is_undefined() || opts.is_null() {
            ListDirectoryOptions::default()
        } else {
            serde_wasm_bindgen::from_value(opts).map_err(|e| {
                js_error(Error::InvalidResponse(format!(
                    "listDirectory options: {e}"
                )))
            })?
        };
        self.list_directory_inner(&path, opts)
            .await
            .map_err(js_error)
    }

    /// Read a file (or byte range), calling `on_chunk(Uint8Array)` per body chunk
    /// in file order.
    ///
    /// `path`: a canonical `/Volumes/…` file path. `opts`: `{ offset?, length? }`.
    /// Returns `{ bytesRead }`.
    #[wasm_bindgen(js_name = readFile)]
    pub async fn read_file(
        &self,
        path: String,
        opts: JsValue,
        on_chunk: Function,
    ) -> Result<JsValue, JsValue> {
        let opts: ReadFileOptions = if opts.is_undefined() || opts.is_null() {
            ReadFileOptions::default()
        } else {
            serde_wasm_bindgen::from_value(opts)
                .map_err(|e| js_error(Error::InvalidResponse(format!("readFile options: {e}"))))?
        };
        self.read_file_inner(&path, opts, &on_chunk)
            .await
            .map_err(js_error)
    }

    /// Metadata for a single file (the analog of an HTTP HEAD).
    ///
    /// `path`: a canonical `/Volumes/…` file path. Returns
    /// `{ path, fileSize, lastModified, contentType?, etag? }`.
    #[wasm_bindgen(js_name = stat)]
    pub async fn stat(&self, path: String) -> Result<JsValue, JsValue> {
        self.stat_inner(&path).await.map_err(js_error)
    }
}

impl UcFilesEngine {
    /// Build the canonical Unity Catalog factory once per op. On wasm it drives a
    /// browser Fetch transport (bearer via `with_auth` when a token is set,
    /// otherwise the ambient browser session) — same construction as the table
    /// path's [`UcRestResolver`](crate::catalog::UcRestResolver).
    async fn factory(&self) -> Result<UnityObjectStoreFactory, Error> {
        Ok(UnityObjectStoreFactory::builder()
            .with_uri(self.base_url.as_str())
            .with_token(self.auth_token.clone())
            .with_allow_unauthenticated(self.auth_token.is_none())
            .build()
            .await?)
    }

    async fn list_directory_inner(
        &self,
        path: &str,
        opts: ListDirectoryOptions,
    ) -> Result<JsValue, Error> {
        let parsed = VolumePath::parse(path)?;
        let factory = self.factory().await?;
        let page = crate::files::engine::list_directory(
            &factory,
            &parsed,
            opts.max_results,
            opts.page_token,
        )
        .await?;
        serde_wasm_bindgen::to_value(&page)
            .map_err(|e| Error::InvalidResponse(format!("directory page: {e}")))
    }

    async fn read_file_inner(
        &self,
        path: &str,
        opts: ReadFileOptions,
        on_chunk: &Function,
    ) -> Result<JsValue, Error> {
        let parsed = VolumePath::parse(path)?;
        let factory = self.factory().await?;
        let bytes_read =
            crate::files::engine::read_file(&factory, &parsed, opts.offset, opts.length, |chunk| {
                let array = Uint8Array::from(chunk.as_ref());
                on_chunk.call1(&JsValue::NULL, &array.into()).map_err(|e| {
                    Error::InvalidResponse(format!("on_chunk callback threw: {e:?}"))
                })?;
                Ok(())
            })
            .await?;
        let stats = ReadStats { bytes_read };
        serde_wasm_bindgen::to_value(&stats)
            .map_err(|e| Error::InvalidResponse(format!("read stats: {e}")))
    }

    async fn stat_inner(&self, path: &str) -> Result<JsValue, Error> {
        let parsed = VolumePath::parse(path)?;
        let factory = self.factory().await?;
        let meta = crate::files::engine::stat(&factory, &parsed).await?;
        serde_wasm_bindgen::to_value(&meta)
            .map_err(|e| Error::InvalidResponse(format!("file metadata: {e}")))
    }
}
