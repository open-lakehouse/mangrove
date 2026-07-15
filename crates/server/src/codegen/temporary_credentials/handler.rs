// @generated — do not edit by hand.
//! Handler trait for [`TemporaryCredentialHandler`].
//!
//! Implement this trait to provide a custom backend for this service, then mount the
//! generated handler functions (in the sibling `server` module) onto an `axum::Router`
//! with your implementation as state.
//!
//! # Composability
//!
//! A single struct can implement multiple handler traits to serve multiple
//! services. Use [`axum::Router::merge`] to compose per-service routers together.
//!
//! Service for generating temporary credentials to access tables and storage paths.
//! Credentials are short-lived and scoped to a specific operation (read or read/write).
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::temporary_credentials::v1::*;
#[async_trait]
pub trait TemporaryCredentialHandler<Cx = crate::api::RequestContext>:
    Send + Sync + 'static
{
    /// Generate a new set of credentials for a table.
    async fn generate_temporary_table_credentials(
        &self,
        request: GenerateTemporaryTableCredentialsRequest,
        context: Cx,
    ) -> Result<TemporaryCredential>;
    /// Generate a new set of credentials for a path.
    async fn generate_temporary_path_credentials(
        &self,
        request: GenerateTemporaryPathCredentialsRequest,
        context: Cx,
    ) -> Result<TemporaryCredential>;
    /// Generate a new set of credentials for a volume.
    ///
    /// The metastore must have the `external_access_enabled` flag set to true
    /// (default false). The caller must have the `EXTERNAL_USE_SCHEMA`
    /// privilege on the parent schema (granted by a catalog owner).
    async fn generate_temporary_volume_credentials(
        &self,
        request: GenerateTemporaryVolumeCredentialsRequest,
        context: Cx,
    ) -> Result<TemporaryCredential>;
    /// Generate a new set of credentials for a model version.
    ///
    /// The metastore must have the `external_access_enabled` flag set to true
    /// (default false). The caller must have the `EXTERNAL_USE_SCHEMA`
    /// privilege on the parent schema (granted by a catalog owner).
    async fn generate_temporary_model_version_credentials(
        &self,
        request: GenerateTemporaryModelVersionCredentialsRequest,
        context: Cx,
    ) -> Result<TemporaryCredential>;
}
