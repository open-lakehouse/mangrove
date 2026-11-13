//! An in-process `uc-server` for hermetic tests.
//!
//! [`TestServer`] runs the same service `uc-server serve` builds ([`embed::start`]),
//! with an ephemeral in-memory SQLite backend and a temporary `file://` managed
//! storage root, on a random loopback port. Rust tests drive it through
//! [`TestServer::context`]. External engines (DuckDB, Spark) get it from the
//! `uc-test-server` binary, which prints the address once it is ready.
//!
//! The storage root is canonicalized before it is configured. On macOS `/tmp`
//! is a symlink to `/private/tmp`, and the server, `object_store` and the engines
//! all have to agree on one spelling of every path.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use axum::extract::Request;
use axum::middleware::{self, Next};
use axum::response::Response;

use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use unitycatalog_server::config::{
    Backend, Config, ConfigValue, LocalStorageConfig, SqliteBackendConfig,
};
use unitycatalog_server::embed;
use url::Url;

use crate::{AcceptanceError, AcceptanceResult, JourneyContext};

/// A running in-process server. Stops when dropped.
pub struct TestServer {
    addr: SocketAddr,
    root: PathBuf,
    requests: RequestLog,
    // Keeps a temporary root alive (and removes it) for the server's lifetime.
    _tempdir: Option<tempfile::TempDir>,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<std::io::Result<()>>>,
}

impl TestServer {
    /// Start a server on a random loopback port over a fresh temporary storage root.
    pub async fn start() -> AcceptanceResult<Self> {
        let tempdir = tempfile::tempdir()?;
        let mut server = Self::start_at(tempdir.path(), 0, false).await?;
        server._tempdir = Some(tempdir);
        Ok(server)
    }

    /// Start a server on `127.0.0.1:port` (`0` picks a free port) whose managed
    /// storage root is the existing directory `root`. With `echo_requests`, every
    /// request is also printed to stderr as `<METHOD> <path> -> <status>`.
    pub async fn start_at(root: &Path, port: u16, echo_requests: bool) -> AcceptanceResult<Self> {
        let root = std::fs::canonicalize(root)?;
        let requests = RequestLog::default();
        let log = requests.clone();
        let router = embed::start(&config(&root)?)
            .await
            .map_err(|e| AcceptanceError::JourneyValidation(format!("start server: {e}")))?
            .router
            .layer(middleware::from_fn(move |req: Request, next: Next| {
                let log = log.clone();
                async move { log.record(req, next, echo_requests).await }
            }));

        let listener = TcpListener::bind(("127.0.0.1", port)).await?;
        let addr = listener.local_addr()?;
        let (shutdown, signal) = oneshot::channel::<()>();
        let task = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = signal.await;
                })
                .await
        });

        Ok(Self {
            addr,
            root,
            requests,
            _tempdir: None,
            shutdown: Some(shutdown),
            task: Some(task),
        })
    }

    /// The server origin, e.g. `http://127.0.0.1:54321`.
    pub fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// The canonical host path backing managed storage.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The managed storage root as a `file://` URL with a trailing slash.
    pub fn storage_root(&self) -> String {
        storage_root_url(&self.root)
    }

    /// Every request served so far, as `<METHOD> <path>` (no query string).
    pub fn requests(&self) -> Vec<String> {
        self.requests.0.lock().unwrap().clone()
    }

    /// A conformance context targeting this server, anonymously.
    pub fn context(&self) -> AcceptanceResult<JourneyContext> {
        JourneyContext::live(&self.url(), self.storage_root())
    }

    /// Stop accepting requests and wait for in-flight ones to finish.
    pub async fn shutdown(mut self) -> AcceptanceResult<()> {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(task) = self.task.take() {
            task.await
                .map_err(|e| AcceptanceError::JourneyExecution(format!("server task: {e}")))??;
        }
        Ok(())
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

/// The requests a [`TestServer`] has served, in arrival order.
#[derive(Clone, Default)]
struct RequestLog(Arc<Mutex<Vec<String>>>);

impl RequestLog {
    async fn record(&self, req: Request, next: Next, echo: bool) -> Response {
        let line = format!("{} {}", req.method(), req.uri().path());
        self.0.lock().unwrap().push(line.clone());
        let response = next.run(req).await;
        if echo {
            eprintln!("{line} -> {}", response.status().as_u16());
        }
        response
    }
}

/// The server config: in-memory SQLite, anonymous auth, `root` as the only
/// allowed local root and as the metastore's managed storage root.
fn config(root: &Path) -> AcceptanceResult<Config> {
    let root_str = root.to_str().ok_or_else(|| {
        AcceptanceError::JourneyValidation(format!("storage root {root:?} is not UTF-8"))
    })?;
    let mut config = Config {
        backend: Backend::Sqlite(SqliteBackendConfig {
            path: ConfigValue::Value(":memory:".to_string()),
        }),
        local_storage: LocalStorageConfig {
            allowed_roots: vec![root_str.to_string()],
        },
        managed_storage_root: Some(storage_root_url(root)),
        ..Default::default()
    };
    // The bundled UI is a separate build feature; don't ask for it.
    config.ui.serve = false;
    Ok(config)
}

fn storage_root_url(root: &Path) -> String {
    let url = Url::from_directory_path(root).expect("canonical storage root is absolute");
    url.to_string()
}
