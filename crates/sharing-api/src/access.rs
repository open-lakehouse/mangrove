//! The [`StorageAccess`] port: how the server reaches a shared asset's data, and
//! how it hands that access on to recipients.
//!
//! Two kinds of access come out of one resolution:
//!
//! - the server's **own** reads (a table's log, to list its files), through an
//!   [`ObjectStore`](object_store::ObjectStore);
//! - **recipient** access, either as presigned file URLs ([`FileSigner`], used by
//!   the Delta Sharing `query` response) or as temporary storage credentials
//!   ([`StorageAccess::vend_read_credential`], used by the Open Sharing
//!   `temporary-*-credentials` endpoints).
//!
//! An implementation typically either *delegates* — asks the source catalog for a
//! short-lived, downscoped credential and derives everything from it, storing no
//! secrets — or holds storage credentials itself and vends locally.

use std::sync::Arc;

use async_trait::async_trait;
use object_store::DynObjectStore;
use url::Url;

use unitycatalog_sharing_client::models::open_sharing::v1::SharingTemporaryCredentials;

use crate::catalog::SharedAsset;
use crate::error::Result;

/// A presigned (or otherwise directly readable) URL for one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedFile {
    /// The URL the recipient reads the file from.
    pub url: String,
    /// When `url` stops being valid, in milliseconds since the Unix epoch.
    /// `None` when it does not expire (for example a local `file://` URL).
    pub expiration_timestamp: Option<i64>,
}

/// Turns storage URLs of files under a shared asset into URLs a recipient can read.
#[async_trait]
pub trait FileSigner: Send + Sync {
    /// Sign `files` (absolute storage URLs), returning one [`SignedFile`] per input,
    /// in order.
    async fn sign(&self, files: &[Url]) -> Result<Vec<SignedFile>>;
}

/// Server-side access to a shared table: a store for reading it, and a signer for
/// handing its files to recipients.
#[derive(Clone)]
pub struct TableStorage {
    /// An object store over the asset's location.
    ///
    /// It must address objects by their path from the root of the location's
    /// bucket / container (the shape [`object_store::parse_url`] returns), since
    /// the table reader derives object paths from the full location URL.
    pub store: Arc<DynObjectStore>,
    /// Presigns the table's data files for the query response.
    pub signer: Arc<dyn FileSigner>,
}

/// The storage access port. See the module docs.
#[async_trait]
pub trait StorageAccess<Cx = crate::DefaultContext>: Send + Sync + 'static {
    /// Storage access for serving a shared table.
    async fn table_storage(&self, asset: &SharedAsset, cx: &Cx) -> Result<TableStorage>;

    /// Read-only temporary credentials for a shared asset's location, in the Open
    /// Sharing credential envelope. Open Sharing only grants read access.
    async fn vend_read_credential(
        &self,
        asset: &SharedAsset,
        cx: &Cx,
    ) -> Result<SharingTemporaryCredentials>;
}

/// A [`FileSigner`] that returns each URL unchanged and never expires it.
///
/// For storage the recipient can read directly — a local `file://` table in
/// tests and development, or a public bucket.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnsignedUrls;

#[async_trait]
impl FileSigner for UnsignedUrls {
    async fn sign(&self, files: &[Url]) -> Result<Vec<SignedFile>> {
        Ok(files
            .iter()
            .map(|url| SignedFile {
                url: url.to_string(),
                expiration_timestamp: None,
            })
            .collect())
    }
}
