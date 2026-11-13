use unitycatalog_common::models::model_versions::v1::ModelVersion;
use unitycatalog_common::models::tables::v1::Table;
use unitycatalog_common::models::temporary_credentials::v1::*;
use unitycatalog_common::models::volumes::v1::Volume;
use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};

use super::RequestContext;
use crate::error::from_store;
use crate::policy::{Permission, Policy};
use crate::services::location::StorageLocationUrl;
use crate::services::vending::{mint, to_temporary_credential};
use crate::services::{ProvidesManagedStorageRoot, ServerHandler};
use crate::store::ResourceStoreReader;
use crate::{Error, Result};
pub use unitycatalog_handlers::temporary_credentials::TemporaryCredentialHandler;

use buffa::Enumeration;
use olai_storage_credentials::{AccessLevel, VendedCredential};
use unitycatalog_storage_access::ProvidesStorageAccessStore;

/// Mint a credential for a securable stored at `location`, through the handler's
/// vendor.
///
/// Local (`file://`) storage needs neither an external location nor a cloud
/// credential: the client builds a `LocalFileSystem` addressed by full path. Any
/// other location is minted from the credential of the external location
/// governing it, or from the metastore root's credential for managed storage
/// under the root. The caller has authorized the securable.
pub(crate) async fn vend_location<Cx: Send + Sync + 'static>(
    handler: &ServerHandler<Cx>,
    location: &StorageLocationUrl,
    access: AccessLevel,
) -> Result<VendedCredential> {
    mint(handler.vendor(), location, access).await
}

/// [`vend_location`] as the UC REST response.
pub(crate) async fn vend_for_location<Cx: Send + Sync + 'static>(
    handler: &ServerHandler<Cx>,
    location: &StorageLocationUrl,
    access: AccessLevel,
) -> Result<TemporaryCredential> {
    let vended = vend_location(handler, location, access).await?;
    Ok(to_temporary_credential(&vended, location))
}

/// Authorize a temporary *path* credential for `storage_url` at `access`.
///
/// Local storage is governed by no external location, so there is nothing to
/// authorize against. Otherwise the caller needs the access on the concrete
/// external location covering the path. Paths in the metastore's managed storage
/// that no external location covers are denied: managed storage is reached
/// through table and staging-table credentials, never by path.
pub(crate) async fn authorize_path_vend(
    handler: &ServerHandler<RequestContext>,
    storage_url: &StorageLocationUrl,
    access: AccessLevel,
    context: &RequestContext,
) -> Result<()> {
    if storage_url.is_local() {
        return Ok(());
    }
    let covering = handler
        .storage_access()
        .covering_location(storage_url.location())
        .await
        .map_err(from_store)?;
    match covering {
        Some(location) => {
            handler
                .authorize_checked(&(&location).into(), &required_permission(access), context)
                .await
        }
        None if in_managed_root(handler, storage_url) => Err(Error::NotAllowed),
        None => Err(Error::NotFound),
    }
}

/// Whether `storage_url` lies under the metastore's managed storage root.
fn in_managed_root(
    handler: &ServerHandler<RequestContext>,
    storage_url: &StorageLocationUrl,
) -> bool {
    handler
        .managed_storage_root()
        .and_then(|root| StorageLocationUrl::parse(root).ok())
        .is_some_and(|root| root.scope().covers(storage_url.location()))
}

/// Mint a credential for the table at `table_ident` (already authorized).
pub(crate) async fn vend_table(
    handler: &ServerHandler<RequestContext>,
    table_ident: &ResourceIdent,
    access: AccessLevel,
) -> Result<(VendedCredential, StorageLocationUrl)> {
    let (resource, _) = handler.get(table_ident).await?;
    let table: Table = resource.try_into()?;
    let location = table
        .storage_location
        .ok_or_else(|| Error::invalid_argument("table does not have a storage location"))?;
    let storage_url = StorageLocationUrl::parse(&location)?;
    let vended = vend_location(handler, &storage_url, access).await?;
    Ok((vended, storage_url))
}

/// The permission a vend operation requires from the policy.
///
/// Read-only vending requires [`Permission::Read`]; read-write vending requires
/// [`Permission::Write`] so the policy can deny write access independently.
pub(crate) fn required_permission(access: AccessLevel) -> Permission {
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
impl TemporaryCredentialHandler<RequestContext> for ServerHandler<RequestContext> {
    #[tracing::instrument(skip(self, context))]
    async fn generate_temporary_path_credentials(
        &self,
        request: GenerateTemporaryPathCredentialsRequest,
        context: RequestContext,
    ) -> Result<TemporaryCredential> {
        let operation = to_access_level(request.operation.to_i32());
        let storage_url = StorageLocationUrl::parse(&request.url)?;
        // Authorize against the concrete external location and the operation
        // actually requested, rather than the unscoped `SecuredAction` default.
        authorize_path_vend(self, &storage_url, operation, &context).await?;
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
        let (vended, storage_url) = vend_table(self, &table_ident, operation).await?;
        Ok(to_temporary_credential(&vended, &storage_url))
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
        let result = vend_for_location(&handler, &location, AccessLevel::Read).await;
        assert!(matches!(result, Err(Error::NotFound)), "{result:?}");
    }

    #[cfg(feature = "memory")]
    fn memory_handler(
        decision: crate::policy::Decision,
    ) -> crate::services::ServerHandler<RequestContext> {
        use std::sync::Arc;

        use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

        use crate::memory::InMemoryResourceStore;
        use crate::policy::ConstantPolicy;
        use crate::services::ServerHandler;

        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::new(decision));
        ServerHandler::try_new_tokio(policy, store).unwrap()
    }

    #[cfg(feature = "memory")]
    fn ctx() -> RequestContext {
        RequestContext {
            recipient: crate::policy::Principal::anonymous(),
        }
    }

    #[cfg(feature = "memory")]
    async fn register(
        handler: &crate::services::ServerHandler<RequestContext>,
        credential: &str,
        location: Option<&str>,
    ) {
        use unitycatalog_common::models::credentials::v1::{
            AwsIamRoleConfig, CreateCredentialRequest, Purpose,
        };
        use unitycatalog_common::models::external_locations::v1::CreateExternalLocationRequest;

        use crate::api::{CredentialHandler, ExternalLocationHandler};

        handler
            .create_credential(
                CreateCredentialRequest {
                    name: credential.to_string(),
                    purpose: Purpose::Storage.into(),
                    aws_iam_role: Some(AwsIamRoleConfig {
                        role_arn: format!("arn:aws:iam::123456789012:role/{credential}"),
                        ..Default::default()
                    })
                    .into(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        if let Some(url) = location {
            handler
                .create_external_location(
                    CreateExternalLocationRequest {
                        name: format!("{credential}-location"),
                        url: url.to_string(),
                        credential_name: credential.to_string(),
                        ..Default::default()
                    },
                    ctx(),
                )
                .await
                .unwrap();
        }
    }

    /// The role ARN the resolver picks for `location`, or `None` when nothing
    /// governs it. Resolution only: no credential is minted (no STS call).
    #[cfg(feature = "memory")]
    async fn resolved_role(
        handler: &crate::services::ServerHandler<RequestContext>,
        location: &str,
    ) -> Option<String> {
        use olai_credential_vending::{CredentialResolver, RegisteredCredential};

        let url = StorageLocationUrl::parse(location).unwrap();
        let resolver = crate::services::vending::CatalogResolver::new(
            handler.storage_access_arc(),
            handler.managed_root_credential(),
        );
        match resolver.resolve(url.location()).await {
            Ok(RegisteredCredential::AwsIamRole(role)) => Some(role.role_arn),
            Ok(other) => panic!("unexpected credential {other:?}"),
            Err(olai_credential_vending::Error::NotFound { .. }) => None,
            Err(e) => panic!("{e}"),
        }
    }

    #[cfg(feature = "memory")]
    #[tokio::test]
    async fn managed_root_credential_governs_uncovered_managed_storage() {
        let handler = memory_handler(crate::policy::Decision::Allow)
            .with_managed_storage_root(Some("s3://bucket/meta"))
            .with_managed_storage_root_credential(Some("root"));
        register(&handler, "root", None).await;
        register(&handler, "ext", Some("s3://bucket/ext")).await;

        let root_role = Some("arn:aws:iam::123456789012:role/root".to_string());
        assert_eq!(
            resolved_role(&handler, "s3://bucket/meta/__unitystorage/tables/t1").await,
            root_role,
            "managed storage under the root resolves to the root credential"
        );
        assert_eq!(
            resolved_role(&handler, "s3://bucket/ext/data").await,
            Some("arn:aws:iam::123456789012:role/ext".to_string()),
            "external locations still govern their paths"
        );
        assert_eq!(
            resolved_role(&handler, "s3://bucket/metadata/x").await,
            None,
            "segment-aware: a sibling of the root is not under it"
        );
    }

    #[cfg(feature = "memory")]
    #[tokio::test]
    async fn managed_root_without_credential_is_not_governed() {
        let handler = memory_handler(crate::policy::Decision::Allow)
            .with_managed_storage_root(Some("s3://bucket/meta"));
        assert_eq!(
            resolved_role(&handler, "s3://bucket/meta/__unitystorage/tables/t1").await,
            None
        );
    }

    /// Managed storage is reached through table / staging credentials, never by
    /// path: a temporary path credential inside the managed root is refused even
    /// though the root has a credential.
    #[cfg(feature = "memory")]
    #[tokio::test]
    async fn path_credentials_inside_managed_storage_are_refused() {
        let handler = memory_handler(crate::policy::Decision::Allow)
            .with_managed_storage_root(Some("s3://bucket/meta"))
            .with_managed_storage_root_credential(Some("root"));
        register(&handler, "root", None).await;
        let url = StorageLocationUrl::parse("s3://bucket/meta/__unitystorage/tables/t1").unwrap();
        let result = authorize_path_vend(&handler, &url, AccessLevel::Read, &ctx()).await;
        assert!(matches!(result, Err(Error::NotAllowed)), "{result:?}");

        let outside = StorageLocationUrl::parse("s3://elsewhere/x").unwrap();
        let result = authorize_path_vend(&handler, &outside, AccessLevel::Read, &ctx()).await;
        assert!(matches!(result, Err(Error::NotFound)), "{result:?}");
    }
}
