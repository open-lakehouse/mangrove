#![cfg_attr(docsrs, feature(doc_cfg))]
//! Portable Unity Catalog **Open Sharing** / **Delta Sharing** REST API.
//!
//! This crate owns the sharing *semantics* — share/schema/table discovery, the
//! kernel-backed NDJSON query path (table version / metadata / query), and the
//! storage-backed asset surface (volumes, agent skills) — behind a narrow
//! backend [port](backend::SharingBackend). Any server can serve the identical
//! `/api/v1/delta-sharing` and `/api/v1/open-sharing` surfaces by implementing
//! [`SharingBackend`] over its own share store, table/volume resolution,
//! credential vending, and authorization; all the sharing business logic is
//! shared here, so the behavior is identical by construction.
//!
//! Unlike the fork-free `olai-uc-delta-api`, this crate deliberately owns the
//! DataFusion + `delta-kernel` query path (via `olai-uc-datafusion`'s
//! `ReconciledLogProvider`), isolating those git-pinned dependencies behind one
//! crate so downstream servers stay free of them.
//!
//! The wire types come from `olai-uc-sharing-client`
//! ([`unitycatalog_sharing_client`]).
//!
//! # Layout
//! - [`error`] — the decoupled error contract ([`SharingApiError`] +
//!   [`SharingBackendError`]).
//! - [`backend`] — the [`SharingBackend`] port trait + its coordinate/spec types.
//! - [`kernel`] — the `ObjectStoreFactory` port + kernel engine builder.
//! - [`session`] — the DataFusion/kernel NDJSON query path.
//! - [`handler`] — the handler traits + the generic blanket impl.
//! - [`router`] — the axum routers for the Delta Sharing + Open Sharing surfaces.

pub mod backend;
pub mod codegen;
pub mod error;
pub mod handler;
pub mod kernel;
#[cfg(feature = "axum")]
pub mod router;
pub mod session;
#[cfg(feature = "testing")]
pub mod testing;

/// The default context type for the generated sharing handler traits.
///
/// The traits are generic over a context `Cx`; this crate-neutral alias is the
/// default so the portable crate does not name any server's context type. A
/// server plugs in its own request-context type when it implements the port.
pub type DefaultContext = ();

pub use backend::{SharingBackend, SharingCapabilities};
pub use error::{Error, Result};
pub use handler::{SharingApiHandler, SharingQueryHandler};
pub use kernel::ObjectStoreFactory;
#[cfg(feature = "axum")]
pub use router::{get_router, open_sharing_router};

/// The generated handler traits (`SharingHandler`, `SharingVolumeHandler`,
/// `SharingSkillHandler`), re-exported at the crate root.
pub use codegen::sharing::SharingHandler;
pub use codegen::sharing_skill::SharingSkillHandler;
pub use codegen::sharing_volume::SharingVolumeHandler;

/// The crate's error type, re-exported under a descriptive alias.
pub type SharingApiError = Error;
/// The crate's result type, re-exported under a descriptive alias.
pub type SharingApiResult<T> = Result<T>;
