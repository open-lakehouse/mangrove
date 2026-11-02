//! The server without a socket, for hosts that drive the router themselves.
//!
//! [`start`] builds the same service `uc-server serve` runs (backend, API
//! surfaces, authentication, operational endpoints), but returns the axum
//! [`Router`] instead of binding an address. A host serves it however it
//! receives requests, e.g. a Node-API addon behind a Neon Function's fetch
//! handler calls `router.oneshot(request)` per request.
//!
//! Build with `default-features = false` and `features = ["embed", "postgres"]`
//! (or `"sqlite"`) for the smallest server; add `swagger`, `ui`, `proxy`,
//! `storage-proxy` or `files` for those surfaces. Load the configuration with
//! [`Config::from_env`] when there is no config file.
//!
//! Like `serve`, [`start`] never migrates a durable backend and fails unless its
//! schema is current: apply migrations out of band with [`migrate`].
//!
//! ```no_run
//! # async fn example() -> unitycatalog_common::Result<()> {
//! use unitycatalog_server::config::Config;
//! use unitycatalog_server::embed;
//!
//! let config = Config::from_env().map_err(unitycatalog_common::Error::Generic)?;
//! let service = embed::start(&config).await?;
//! let router: axum::Router = service.router;
//! # let _ = router;
//! # Ok(())
//! # }
//! ```

use axum::Router;
use unitycatalog_common::Result;

use crate::config::Config;

pub use crate::run::migrate;

/// A started server: the router to serve requests through.
///
/// The router is cheap to clone. Dropping the last clone closes the backend's
/// connection pool.
#[derive(Clone)]
pub struct Service {
    /// The whole service: the Unity Catalog API, `/health`, `/version` and
    /// `/capabilities`, plus whichever optional surfaces the config enables.
    pub router: Router,
}

/// Connect the backend, check its schema and build the service from `config`.
pub async fn start(config: &Config) -> Result<Service> {
    let router = crate::run::build_app(config).await?;
    Ok(Service { router })
}
