#![cfg_attr(docsrs, feature(doc_cfg))]
//! Same-origin Unity Catalog **storage byte-proxy**.
//!
//! A browser/wasm query engine points a stock
//! [`object_store::http::HttpStore`] at this proxy instead of reaching cloud
//! object storage directly. The proxy authorizes the request, vends a scoped
//! credential, and **streams object bytes** (read + write) to/from storage
//! server-side. Because the wasm side never speaks a cloud-native protocol
//! (SigV4 / Azure SAS / GCS auth) this makes S3/GCP work on wasm for free and
//! removes the per-cloud CORS configuration burden — the browser only ever talks
//! to its own origin.
//!
//! The crate owns the proxy *semantics* behind a narrow [port](StorageProxyBackend):
//! the wire contract (ranged `GET` / `HEAD` / whole-object `PUT` with a
//! server-enforced `If-Match`), the streaming HTTP mapping, and the
//! confused-deputy path-scope guard. Any server serves the identical surface by
//! implementing [`StorageProxyBackend`] over its own storage, credential vending,
//! and authorization.
//!
//! # Wire contract
//!
//! ```text
//! GET  {base}/{securable}/{key}            -> 200 + body + Content-Length + ETag
//! GET  {base}/{securable}/{key}  Range: .. -> 206 + Content-Range (+ Accept-Ranges)
//! HEAD {base}/{securable}/{key}            -> headers only
//! PUT  {base}/{securable}/{key}  [body]    -> 200 + ETag   (whole-object upload)
//! ```
//!
//! `{securable}` is a single typed path segment — `table:<fqn>`, `vol:<fqn>`, or
//! `path:<percent-encoded cloud URL>` (see [`Securable`]) — and `{key}` is the
//! object key relative to that securable's root. `GET`/`HEAD` open a read-scoped
//! store; `PUT` opens a write-scoped store. A conditional write rides an
//! `If-Match` HTTP header the proxy relays to storage, mapping a mismatch to
//! `412 Precondition Failed`.
//!
//! # Example
//!
//! Implement [`StorageProxyBackend`] over your own storage, then mount it with
//! [`router_with_context`]. Here the in-memory backend from the `testing` feature
//! stands in for a real one:
//!
//! ```
//! # #[cfg(feature = "testing")] {
//! use std::sync::Arc;
//! use unitycatalog_storage_proxy::{ContextExtractor, router_with_context_at};
//! use unitycatalog_storage_proxy::testing::InMemoryStorageProxyBackend;
//!
//! let backend = InMemoryStorageProxyBackend::table("main.default.events");
//! let extract_cx: ContextExtractor<()> = Arc::new(|_parts| Box::pin(async { Ok(()) }));
//! // `base = ""` yields relative routes; a host adds the `/storage-proxy` prefix
//! // via `.nest`. Pass `"/storage-proxy"` instead to `.merge` the surface directly.
//! let proxy: axum::Router = router_with_context_at("", Arc::new(backend), extract_cx);
//! # let _ = proxy;
//! # }
//! ```

#[cfg(feature = "bin")]
pub mod auth;
#[cfg(feature = "server")]
pub mod backend;
#[cfg(feature = "bin")]
pub mod cli;
#[cfg(feature = "client-arm")]
pub mod client;
#[cfg(feature = "server")]
pub mod config;
#[cfg(feature = "server")]
pub mod error;
#[cfg(feature = "server")]
pub mod handler;
#[cfg(feature = "server")]
pub mod router;
#[cfg(feature = "testing")]
pub mod testing;

#[cfg(feature = "bin")]
pub use auth::{AuthLayer, AuthMode, ForwardedIdentity};
#[cfg(feature = "server")]
pub use backend::{
    ForwardedUser, ProxyCapabilities, ProxyReq, ProxyVerb, Securable, StorageProxyBackend,
};
#[cfg(feature = "client-arm")]
pub use client::UnityFactoryProxyBackend;
#[cfg(feature = "server")]
pub use error::{ProxyError, ProxyResult};
#[cfg(feature = "server")]
pub use handler::StorageProxyHandler;
#[cfg(feature = "server")]
pub use router::{
    ContextExtractor, router_from_extension, router_from_extension_at, router_with_context,
    router_with_context_at,
};
