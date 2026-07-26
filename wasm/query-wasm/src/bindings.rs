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
// Files: the volume file-browser surface (`UcFilesEngine`).
//
// Mirrors hydrofoil's Tauri backend split exactly:
//  - the unary METADATA RPCs (GetFileMetadata / ListDirectoryContents /
//    DeleteFile / CreateDirectory / DeleteDirectory / GetDirectoryMetadata) go
//    through ONE generic dispatch export, `connectUnary`, as binary proto
//    (crate::files::service dispatches them through a connectrpc Router); and
//  - file BYTES bypass proto entirely — `readFileBytes` / `writeFileBytes` are
//    dedicated native exports with a raw binary body, since streaming file bytes
//    through the connect envelope is what hydrofoil deliberately avoids.
// =====================================================================

/// Options accepted by `readFileBytes`, mirroring `DownloadFileRequest`.
#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReadFileOptions {
    /// Byte offset to start from (defaults to 0).
    offset: Option<u64>,
    /// Number of bytes to read (defaults to the rest of the file).
    length: Option<u64>,
}

/// Summary returned by `readFileBytes`.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadStats {
    bytes_read: u64,
}

/// Options accepted by `writeFileBytes`.
#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WriteFileOptions {
    /// MIME type recorded as the object's `Content-Type` attribute.
    content_type: Option<String>,
    /// Optional if-match etag: when set, the put is a conditional overwrite that
    /// only succeeds if the object still carries this etag (a lost-update guard);
    /// a failed precondition surfaces as the `CONFLICT`-classed error. Absent =
    /// unconditional overwrite. (A local extension: the proto has no such field.)
    if_match_etag: Option<String>,
}

/// The post-write metadata `writeFileBytes` resolves to (the `UploadFileResponse`
/// shape: path + size + etag).
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct WriteStats {
    path: String,
    file_size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    etag: Option<String>,
}

/// The in-browser Unity Catalog **volume files** engine.
///
/// Metadata operations flow through the [`connectUnary`](UcFilesEngine::connect_unary)
/// dispatch export as binary proto (`portal.files.v1.FilesService`); file bytes
/// flow through the dedicated [`readFileBytes`](UcFilesEngine::read_file_bytes) /
/// [`writeFileBytes`](UcFilesEngine::write_file_bytes) exports.
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

    /// Dispatch one unary `portal.files.v1.FilesService` RPC through the in-wasm
    /// connect Router, as binary proto.
    ///
    /// `path` is the full RPC path (e.g.
    /// `portal.files.v1.FilesService/GetFileMetadata`); `request_bytes` is the
    /// binary-proto request body; the returned `Uint8Array` is the binary-proto
    /// response body. The TS `createWasmFilesTransport` calls this for every
    /// metadata RPC (`toBinary(input)` → here → `fromBinary(output)`). Byte RPCs
    /// (Upload/Download) never reach here — they use the dedicated byte exports.
    ///
    /// Mirrors hydrofoil's `connect_unary_proto` Tauri command.
    #[wasm_bindgen(js_name = connectUnary)]
    pub async fn connect_unary(
        &self,
        path: String,
        request_bytes: Uint8Array,
    ) -> Result<Uint8Array, JsValue> {
        self.connect_unary_inner(&path, request_bytes)
            .await
            .map_err(js_error)
    }

    /// Read a file (or byte range), calling `on_chunk(Uint8Array)` per body chunk
    /// in file order.
    ///
    /// `path`: a canonical `/Volumes/…` file path. `opts`: `{ offset?, length? }`.
    /// Returns `{ bytesRead }`. Bytes bypass the connect dispatcher (native path).
    #[wasm_bindgen(js_name = readFileBytes)]
    pub async fn read_file_bytes(
        &self,
        path: String,
        opts: JsValue,
        on_chunk: Function,
    ) -> Result<JsValue, JsValue> {
        let opts: ReadFileOptions = if opts.is_undefined() || opts.is_null() {
            ReadFileOptions::default()
        } else {
            serde_wasm_bindgen::from_value(opts).map_err(|e| {
                js_error(Error::InvalidResponse(format!(
                    "readFileBytes options: {e}"
                )))
            })?
        };
        self.read_file_inner(&path, opts, &on_chunk)
            .await
            .map_err(js_error)
    }

    /// Write (create or overwrite) a file from one buffered byte body.
    ///
    /// `path`: a canonical `/Volumes/…` file path. `bytes`: the whole file body
    /// (a client stream degrades to one buffered call, as hydrofoil does). `opts`:
    /// `{ contentType?, ifMatchEtag? }`. Returns `{ path, fileSize, etag? }`.
    /// Bytes bypass the connect dispatcher (native path).
    #[wasm_bindgen(js_name = writeFileBytes)]
    pub async fn write_file_bytes(
        &self,
        path: String,
        bytes: Uint8Array,
        opts: JsValue,
    ) -> Result<JsValue, JsValue> {
        let opts: WriteFileOptions = if opts.is_undefined() || opts.is_null() {
            WriteFileOptions::default()
        } else {
            serde_wasm_bindgen::from_value(opts).map_err(|e| {
                js_error(Error::InvalidResponse(format!(
                    "writeFileBytes options: {e}"
                )))
            })?
        };
        self.write_file_inner(&path, bytes, opts)
            .await
            .map_err(js_error)
    }
}

impl UcFilesEngine {
    /// Build the canonical Unity Catalog factory once per op. On wasm it drives a
    /// browser Fetch transport (bearer via `with_token` when a token is set,
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

    async fn connect_unary_inner(
        &self,
        path: &str,
        request_bytes: Uint8Array,
    ) -> Result<Uint8Array, Error> {
        let request = bytes::Bytes::from(request_bytes.to_vec());
        let response = crate::files::service::connect_unary(
            self.base_url.clone(),
            self.auth_token.clone(),
            path,
            request,
        )
        .await
        // The connect error carries the failure class in its code; re-tag it into
        // the engine `Error` so the JS `code` contract (`classify`) holds — a
        // conflict maps through the `conflict:` marker, transport through
        // `network/CORS:`, everything else to `FAILED`.
        .map_err(connect_error_to_engine)?;
        Ok(Uint8Array::from(response.as_ref()))
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

    async fn write_file_inner(
        &self,
        path: &str,
        bytes: Uint8Array,
        opts: WriteFileOptions,
    ) -> Result<JsValue, Error> {
        let parsed = VolumePath::parse(path)?;
        let factory = self.factory().await?;
        let meta = crate::files::engine::write_file(
            &factory,
            &parsed,
            bytes.to_vec(),
            opts.content_type,
            opts.if_match_etag,
        )
        .await?;
        let stats = WriteStats {
            path: meta.path,
            file_size: meta.file_size.max(0) as u64,
            etag: (!meta.etag.is_empty()).then_some(meta.etag),
        };
        serde_wasm_bindgen::to_value(&stats)
            .map_err(|e| Error::InvalidResponse(format!("write stats: {e}")))
    }
}

/// Re-tag a `connectrpc::ConnectError` back into an engine [`Error`] so the JS
/// boundary's `code` contract (`classify`) still holds after the round trip
/// through the dispatcher. The service maps a write conflict to `AlreadyExists`
/// and a transport failure to `Unavailable`; recover those markers here.
fn connect_error_to_engine(err: connectrpc::ConnectError) -> Error {
    use connectrpc::ErrorCode;
    let message = err.message.clone().unwrap_or_default();
    match err.code {
        ErrorCode::AlreadyExists => Error::UnityCatalog(format!("conflict: {message}")),
        ErrorCode::Unavailable => Error::UnityCatalog(format!("network/CORS: {message}")),
        _ => Error::InvalidResponse(message),
    }
}
