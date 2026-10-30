use unitycatalog_common::models::credentials::v1::GetCredentialRequest;
use unitycatalog_common::models::model_versions::v1::ModelVersion;
use unitycatalog_common::models::tables::v1::Table;
use unitycatalog_common::models::temporary_credentials::v1::*;
use unitycatalog_common::models::volumes::v1::Volume;
use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};

use super::RequestContext;
use crate::api::CredentialHandler;
use crate::api::credentials::CredentialHandlerExt;
pub use crate::codegen::temporary_credentials::TemporaryCredentialHandler;
use crate::policy::{Permission, Policy};
use crate::services::location::StorageLocationUrl;
use crate::services::object_store::find_external_location_for_url;
use crate::services::vending::{mint, to_temporary_credential};
use crate::store::ResourceStore;
use crate::{Error, Result};

use buffa::Enumeration;
use olai_storage_credentials::AccessLevel;

/// Vend a temporary credential for a securable stored at `location`.
///
/// Local (`file://`) storage needs neither an external location nor a cloud
/// credential: the client builds a `LocalFileSystem` addressed by full path. Any
/// other location is minted from the credential of the external location governing
/// it.
pub(crate) async fn vend_for_location<T>(
    handler: &T,
    location: &StorageLocationUrl,
    access: AccessLevel,
) -> Result<TemporaryCredential>
where
    T: ResourceStore + CredentialHandlerExt + ?Sized,
{
    let credential = if location.is_local() {
        None
    } else {
        let ext_loc = find_external_location_for_url(location, handler).await?;
        Some(
            handler
                .get_credential_internal(GetCredentialRequest {
                    name: ext_loc.credential_name.clone(),
                    ..Default::default()
                })
                .await?,
        )
    };
    let vended = mint(credential.as_ref(), location, access).await?;
    Ok(to_temporary_credential(&vended, location))
}

/// The permission a vend operation requires from the policy.
///
/// Read-only vending requires [`Permission::Read`]; read-write vending requires
/// [`Permission::Write`] so the policy can deny write access independently.
fn required_permission(access: AccessLevel) -> Permission {
    match access {
        AccessLevel::ReadWrite => Permission::Write,
        _ => Permission::Read,
    }
}

/// Map the proto `operation` integer to an [`AccessLevel`].
///
/// The path, table, and volume request enums all encode the same semantics with
/// the same numeric values (read = 1, write/read-write = 2; path adds
/// create-table = 3), so a single mapping serves all three:
///
/// - `PATH_CREATE_TABLE`, `READ_WRITE`, and `WRITE_VOLUME` are treated as `ReadWrite`.
/// - `READ`, `READ_VOLUME`, and `Unspecified` default to `Read` (least privilege).
fn to_access_level(operation: i32) -> AccessLevel {
    use generate_temporary_path_credentials_request::Operation as PathOp;
    use generate_temporary_table_credentials_request::Operation as TableOp;
    use generate_temporary_volume_credentials_request::Operation as VolumeOp;

    // Check path operations first (values 0–3 are defined for both enums,
    // but semantically we just need to distinguish read from read-write).
    match PathOp::from_i32(operation) {
        Some(PathOp::PATH_READ_WRITE | PathOp::PATH_CREATE_TABLE) => {
            return AccessLevel::ReadWrite;
        }
        Some(PathOp::PATH_READ | PathOp::UNSPECIFIED) | None => {}
    }
    // The table (READ_WRITE = 2), volume (WRITE_VOLUME = 2), and model-version
    // (READ_WRITE_MODEL_VERSION = 2) write operations all share the same numeric
    // value, so any of these enums resolves write access identically.
    match (TableOp::from_i32(operation), VolumeOp::from_i32(operation)) {
        (Some(TableOp::READ_WRITE), _) | (_, Some(VolumeOp::WRITE_VOLUME)) => {
            AccessLevel::ReadWrite
        }
        _ => AccessLevel::Read,
    }
}

#[async_trait::async_trait]
impl<
    T: ResourceStore
        + Policy<RequestContext>
        + CredentialHandler<RequestContext>
        + CredentialHandlerExt,
> TemporaryCredentialHandler<RequestContext> for T
{
    #[tracing::instrument(skip(self, context))]
    async fn generate_temporary_path_credentials(
        &self,
        request: GenerateTemporaryPathCredentialsRequest,
        context: RequestContext,
    ) -> Result<TemporaryCredential> {
        let operation = to_access_level(request.operation.to_i32());
        let storage_url = StorageLocationUrl::parse(&request.url)?;

        // Local (`file://`) storage is governed by no external location, so there is
        // nothing to authorize against; vending it is what lets managed tables be
        // created on a purely-local dev server.
        if storage_url.is_local() {
            return vend_for_location(self, &storage_url, operation).await;
        }

        let ext_loc = find_external_location_for_url(&storage_url, self).await?;
        // Authorize against the concrete external location and the operation
        // actually requested, rather than the unscoped `SecuredAction` default.
        self.authorize_checked(
            &(&ext_loc).into(),
            &required_permission(operation),
            &context,
        )
        .await?;
        vend_for_location(self, &storage_url, operation).await
    }

    /// Generate temporary credentials for a volume.
    ///
    /// Resolves the volume by its UUID, authorizes the requested operation
    /// against the concrete volume, then vends credentials scoped to the
    /// volume's storage location — mirroring
    /// `generate_temporary_table_credentials`.
    ///
    /// See: <https://docs.databricks.com/api/workspace/temporaryvolumecredentials/generatetemporaryvolumecredentials>
    #[tracing::instrument(skip(self, context))]
    async fn generate_temporary_volume_credentials(
        &self,
        request: GenerateTemporaryVolumeCredentialsRequest,
        context: RequestContext,
    ) -> Result<TemporaryCredential> {
        let operation = to_access_level(request.operation.to_i32());
        let volume_id = uuid::Uuid::parse_str(&request.volume_id)
            .map_err(|_| Error::invalid_argument("volume_id is not a valid UUID"))?;
        // Authorize against the concrete volume and the operation actually
        // requested, rather than the unscoped `SecuredAction` default.
        let volume_ident = ResourceIdent::Volume(ResourceRef::Uuid(volume_id));
        self.authorize_checked(&volume_ident, &required_permission(operation), &context)
            .await?;
        let (resource, _) = self.get(&volume_ident).await?;
        let volume: Volume = resource.try_into()?;
        let location = volume.storage_location;
        let storage_url = StorageLocationUrl::parse(&location)?;
        vend_for_location(self, &storage_url, operation).await
    }

    /// Generate temporary credentials for a model version.
    ///
    /// Resolves the model version by its `(catalog, schema, model, version)`
    /// composite name, authorizes the requested operation against the concrete
    /// version, then vends credentials scoped to the version's storage location —
    /// mirroring `generate_temporary_volume_credentials`.
    ///
    /// See: <https://docs.databricks.com/api/workspace/temporarymodelversioncredentials/generatetemporarymodelversioncredentials>
    #[tracing::instrument(skip(self, context))]
    async fn generate_temporary_model_version_credentials(
        &self,
        request: GenerateTemporaryModelVersionCredentialsRequest,
        context: RequestContext,
    ) -> Result<TemporaryCredential> {
        let operation = to_access_level(request.operation.to_i32());
        // The model version is keyed by the composite name that
        // `ModelVersion::resource_name` produces.
        let ident = ResourceIdent::model_version(ResourceName::new([
            request.catalog_name.clone(),
            request.schema_name.clone(),
            request.model_name.clone(),
            request.version.to_string(),
        ]));
        // Authorize against the concrete model version and the operation actually
        // requested, rather than the unscoped `SecuredAction` default.
        self.authorize_checked(&ident, &required_permission(operation), &context)
            .await?;
        let (resource, _) = self.get(&ident).await?;
        let model_version: ModelVersion = resource.try_into()?;
        let location = model_version.storage_location.ok_or_else(|| {
            Error::invalid_argument("model version does not have a storage location")
        })?;
        let storage_url = StorageLocationUrl::parse(&location)?;
        vend_for_location(self, &storage_url, operation).await
    }

    #[tracing::instrument(skip(self, context))]
    async fn generate_temporary_table_credentials(
        &self,
        request: GenerateTemporaryTableCredentialsRequest,
        context: RequestContext,
    ) -> Result<TemporaryCredential> {
        let operation = to_access_level(request.operation.to_i32());
        let table_id = uuid::Uuid::parse_str(&request.table_id)
            .map_err(|_| Error::invalid_argument("table_id is not a valid UUID"))?;
        // Authorize against the concrete table and the operation actually
        // requested, rather than the unscoped `SecuredAction` default.
        let table_ident = ResourceIdent::Table(ResourceRef::Uuid(table_id));
        self.authorize_checked(&table_ident, &required_permission(operation), &context)
            .await?;
        let (resource, _) = self.get(&table_ident).await?;
        let table: Table = resource.try_into()?;
        let location = table
            .storage_location
            .ok_or_else(|| Error::invalid_argument("table does not have a storage location"))?;
        let storage_url = StorageLocationUrl::parse(&location)?;
        vend_for_location(self, &storage_url, operation).await
    }
}
// NOTE: These request types intentionally do not implement `SecuredAction`.
// Authorization is performed inside the handlers against the *concrete* resolved
// resource (external location / table) and the *requested* operation
// (read vs. read-write), which a static `SecuredAction` impl cannot express.
// See `generate_temporary_path_credentials` / `generate_temporary_table_credentials`.

#[cfg(test)]
mod tests {
    use super::*;
    use generate_temporary_path_credentials_request::Operation as PathOp;
    use generate_temporary_table_credentials_request::Operation as TableOp;
    use generate_temporary_volume_credentials_request::Operation as VolumeOp;

    #[test]
    fn vend_operation_volume_read_is_read() {
        assert_eq!(
            to_access_level(VolumeOp::ReadVolume as i32),
            AccessLevel::Read
        );
    }

    #[test]
    fn vend_operation_volume_write_is_read_write() {
        assert_eq!(
            to_access_level(VolumeOp::WriteVolume as i32),
            AccessLevel::ReadWrite
        );
    }

    #[test]
    fn vend_operation_unspecified_defaults_to_read() {
        assert_eq!(
            to_access_level(VolumeOp::Unspecified as i32),
            AccessLevel::Read
        );
        // Out-of-range values are treated as least-privilege read, not an error.
        assert_eq!(to_access_level(99), AccessLevel::Read);
    }

    #[test]
    fn vend_operation_table_and_path_unchanged() {
        assert_eq!(to_access_level(TableOp::Read as i32), AccessLevel::Read);
        assert_eq!(
            to_access_level(TableOp::ReadWrite as i32),
            AccessLevel::ReadWrite
        );
        assert_eq!(
            to_access_level(PathOp::PathCreateTable as i32),
            AccessLevel::ReadWrite
        );
    }

    /// Local storage is vended without an external location or a credential, the
    /// same way for every securable kind, and the response names the location as
    /// given.
    #[cfg(feature = "memory")]
    #[tokio::test]
    async fn local_locations_vend_without_a_credential() {
        use std::sync::Arc;

        use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

        use crate::memory::InMemoryResourceStore;
        use crate::policy::ConstantPolicy;
        use crate::services::ServerHandler;

        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
        let handler = ServerHandler::try_new_tokio(policy, store).unwrap();

        let location = StorageLocationUrl::parse("file:///tmp/uc/tables/t1").unwrap();
        let credential = vend_for_location(&handler, &location, AccessLevel::ReadWrite)
            .await
            .unwrap();
        assert!(credential.credentials.is_none());
        assert_eq!(credential.url, "file:///tmp/uc/tables/t1");
        assert_eq!(credential.expiration_time, 0);

        // Cloud storage still needs a governing external location.
        let location = StorageLocationUrl::parse("s3://bucket/tables/t1").unwrap();
        assert!(matches!(
            vend_for_location(&handler, &location, AccessLevel::Read).await,
            Err(Error::NotFound)
        ));
    }
}
