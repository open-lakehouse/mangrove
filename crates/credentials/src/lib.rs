//! Storage credential vending for Unity Catalog and Open Sharing servers.
//!
//! This crate owns the server side of temporary storage credentials: given a
//! registered storage [`Credential`](unitycatalog_common::models::credentials::v1::Credential)
//! (an AWS IAM role, an Azure service principal / managed identity / storage key)
//! and a storage URL, it mints a short-lived credential downscoped to that URL and
//! operation — an STS session with an inline policy, or a user-delegation /
//! service SAS.
//!
//! - [`location`] parses storage URLs and maps a path onto the most specific
//!   registered location.
//! - [`vending`] mints (and caches) the temporary credentials.
//! - [`resolve`] is the seam a host implements to look up which credential
//!   governs a location.
//!
//! The Unity Catalog server and the Open Sharing server share this crate, so there
//! is exactly one vending implementation. Turning a vended credential into an
//! object store (or presigned URLs) is the consumer side, owned by
//! `olai-uc-object-store`.

mod error;
pub mod location;
pub mod resolve;
pub mod vending;

pub use error::{Error, Result};
pub use location::{StorageLocationScheme, StorageLocationUrl};
pub use resolve::{CredentialResolver, vend_for_location};
pub use vending::{VendOperation, local_path_credential, vend_credential};
