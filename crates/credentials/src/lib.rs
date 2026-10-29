//! Storage credential vending for Unity Catalog and Open Sharing servers.
//!
//! This crate owns the server side of temporary storage credentials: given a
//! registered [`StorageCredential`] (an AWS IAM role, an Azure service principal
//! / managed identity / storage key) and a storage URL, it mints a short-lived
//! [`TemporaryCredential`] downscoped to that URL and operation — an STS session
//! with an inline policy, or a user-delegation / service SAS.
//!
//! - [`model`] defines the credential shapes. They are the crate's own, so a
//!   host needs no catalog models to vend; they serialize like the Unity Catalog
//!   REST models.
//! - [`location`] parses storage URLs and maps a path onto the most specific
//!   registered location.
//! - [`vending`] mints (and caches) the temporary credentials.
//! - [`resolve`] is the seam a host implements to look up which credential
//!   governs a location.
//!
//! [`vending`] and [`resolve`] call cloud providers and sit behind the `vending`
//! feature (on by default); [`model`] and [`location`] build on every target,
//! including `wasm32`.
//!
//! The Unity Catalog server and the Open Sharing server share this crate, so there
//! is exactly one vending implementation. Turning a vended credential into an
//! object store (or presigned URLs) is the consumer side, owned by
//! `olai-uc-object-store`.

mod error;
pub mod location;
pub mod model;
#[cfg(feature = "vending")]
pub mod resolve;
#[cfg(feature = "vending")]
pub mod vending;

pub use error::{Error, Result};
pub use location::{StorageLocationScheme, StorageLocationUrl};
pub use model::{StorageCredential, TemporaryCredential, VendedCredential};
#[cfg(feature = "vending")]
pub use resolve::{CredentialResolver, vend_for_location};
#[cfg(feature = "vending")]
pub use vending::{VendOperation, local_path_credential, vend_credential};
