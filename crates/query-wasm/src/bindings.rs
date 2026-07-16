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

use std::sync::Arc;

use futures::TryStreamExt;
use js_sys::{Function, Uint8Array};
use object_store::ObjectStore;
use object_store::path::Path;
use url::Url;
use wasm_bindgen::prelude::*;

use crate::engine::{execute_chunks, extract_table, open_table, register_table};
use crate::error::Error;
use crate::fetch_store::UcFetchStore;
use crate::resolve::{discover_log, plan_table};
use crate::uc::UcClient;

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
}

impl UcQueryEngine {
    async fn run_query_inner(
        &self,
        sql: &str,
        opts: RunOptions,
        on_batch: &Function,
    ) -> Result<JsValue, Error> {
        // 1. Which table does the SQL read?
        let (reference, address) =
            extract_table(sql, opts.catalog.as_deref(), opts.schema.as_deref())?;

        // 2. Resolve it through Unity Catalog and gate on the v1 envelope.
        let uc = UcClient::new(self.base_url.clone(), self.auth_token.clone());
        let loaded = uc
            .load_table(&address.catalog, &address.schema, &address.table)
            .await?;
        let plan = plan_table(&loaded)?;
        let credential = uc.read_table_credentials(&plan.table_uuid).await?;

        // 3. Vended credential → browser-fetchable store.
        let storage = crate::creds::resolve_storage(&plan.location, &credential)?;
        let table_path = Path::from_url_path(storage.table_url.path())
            .map_err(|e| Error::InvalidUrl(format!("table path: {e}")))?;
        let store: Arc<dyn ObjectStore> = Arc::new(UcFetchStore::try_new(
            storage.table_url.clone(),
            &storage.headers,
        )?);

        // 4. Discover the log, build the snapshot async-native (no prime), and register
        //    under the SQL's name. `latest_version` is `Some` only for catalog-managed
        //    tables, which is exactly when the kernel needs it as `max_catalog_version`.
        let log = discover_log(&store, &table_path, plan.latest_version).await?;
        let opened = open_table(store, &storage.table_url, log, plan.latest_version).await?;
        let table_version = opened.snapshot.version();
        register_table(&opened.ctx, &opened, &reference)?;

        // 5. Stream contract-framed chunks to the callback.
        let mut chunks = execute_chunks(&opened.ctx, sql, opts.limit.map(|l| l as usize)).await?;
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
        serde_wasm_bindgen::to_value(&stats)
            .map_err(|e| Error::InvalidResponse(format!("stats: {e}")))
    }
}
