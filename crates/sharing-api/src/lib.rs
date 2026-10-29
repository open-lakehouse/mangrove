#![cfg_attr(docsrs, feature(doc_cfg))]
//! Portable **Open Sharing** / **Delta Sharing** REST API.
//!
//! This crate owns the sharing *semantics* — share / schema / asset discovery,
//! the table query path (version / metadata / query) and the storage-backed asset
//! surface (volumes, agent skills) — and expresses them over three narrow ports a
//! host plugs implementations into:
//!
//! - [`ShareCatalog`] — which shares and objects a recipient sees, and where a
//!   shared object lives (backed by any source catalog);
//! - [`StorageAccess`] — the server's own reads of a shared asset, presigned file
//!   URLs and temporary credentials for recipients;
//! - [`TableFormat`] — serving one table format; Delta (via `delta_kernel`, no
//!   query engine) is built in behind the `delta` feature.
//!
//! [`SharingService`] composes them and implements every handler; the [`router`]
//! serves it over axum. See `ARCHITECTURE.md` for the overall design.
//!
//! The crate depends on no catalog. An existing catalog becomes a sharing server
//! by implementing the ports over its own discovery, authorization and
//! credential machinery; `olai-sharing-server` is one such host.
//!
//! The wire types come from `olai-sharing-common`
//! ([`open_sharing_common`]), shared with the client.
//!
//! # Layout
//! - [`catalog`] — the [`ShareCatalog`] port and the share / asset types.
//! - [`access`] — the [`StorageAccess`] port and [`FileSigner`](access::FileSigner).
//! - [`format`] — the [`TableFormat`] port, the format registry, the Delta format
//!   and predicate-hint translation.
//! - [`service`] — [`SharingService`], the composition root.
//! - [`handler`] — the handler traits, implemented by [`SharingService`].
//! - [`router`] — the axum routers for the Delta Sharing + Open Sharing surfaces.
//! - [`auth`] — the [`RecipientAuthenticator`](auth::RecipientAuthenticator) port
//!   and the bearer-token layer.
//! - [`error`] — the error contract.

pub mod access;
#[cfg(feature = "axum")]
pub mod auth;
pub mod catalog;
pub mod codegen;
pub mod error;
pub mod format;
pub mod handler;
#[cfg(feature = "delta")]
mod kernel;
#[cfg(feature = "axum")]
pub mod router;
pub mod service;
#[cfg(feature = "testing")]
pub mod testing;

/// The default context type for the generated sharing handler traits.
///
/// The traits are generic over a context `Cx`; this crate-neutral alias is the
/// default so the portable crate does not name any server's context type. A
/// server plugs in its own request-context type when it implements the port.
pub type DefaultContext = ();

pub use access::StorageAccess;
pub use catalog::ShareCatalog;
pub use error::{Error, Result};
pub use format::TableFormat;
pub use handler::{SharingApiHandler, SharingQueryHandler};
#[cfg(feature = "axum")]
pub use router::{get_router, open_sharing_router, sharing_router};
pub use service::SharingService;

/// The generated handler traits (`SharingHandler`, `SharingVolumeHandler`,
/// `SharingSkillHandler`), re-exported at the crate root.
pub use codegen::sharing::SharingHandler;
pub use codegen::sharing_skill::SharingSkillHandler;
pub use codegen::sharing_volume::SharingVolumeHandler;

/// The crate's error type, re-exported under a descriptive alias.
pub type SharingApiError = Error;
/// The crate's result type, re-exported under a descriptive alias.
pub type SharingApiResult<T> = Result<T>;
