use itertools::Itertools;

use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::volumes::v1::*;
use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};

use super::staging_tables::{child_location, resolve_managed_parent_location};
use super::{RequestContext, SecuredAction};
pub use crate::codegen::volumes::VolumeHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::services::location::StorageLocationUrl;
use crate::services::object_store::validate_external_storage_location;
use crate::services::{ProvidesLocalStoragePolicy, ProvidesManagedStorageRoot};
use crate::store::ResourceStore;
use crate::{Error, Result};

#[async_trait::async_trait]
impl<
    T: ResourceStore
        + Policy<RequestContext>
        + ProvidesLocalStoragePolicy
        + ProvidesManagedStorageRoot,
> VolumeHandler<RequestContext> for T
{
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_volume(
        &self,
        request: CreateVolumeRequest,
        context: RequestContext,
    ) -> Result<Volume> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;

        let volume_type = request.volume_type.as_known().unwrap_or_default();

        // Empty unless a managed volume allocates its id below. An empty
        // `volume_id` leaves id assignment to the store (UUID v7), matching every
        // other resource; a managed volume sets it so the storage path can embed
        // it and the store persists the row under that same id.
        let mut volume_id = String::new();
        let storage_location = match volume_type {
            VolumeType::External => {
                // External volumes MUST have an explicit storage location that
                // lives within a registered external location and does not
                // overlap any existing table or volume.
                let location = request
                    .storage_location
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        Error::invalid_argument("storage_location is required for EXTERNAL volumes")
                    })?;
                let parsed = StorageLocationUrl::parse(&location)?;
                validate_external_storage_location(self, &parsed).await?;
                location
            }
            VolumeType::Managed => {
                // Managed volumes derive their storage location from the managed
                // parent location resolved for the schema/catalog (schema →
                // catalog → metastore), exactly like managed tables. That parent
                // already carries the `__unitystorage/{schemas,catalogs}/<id>`
                // segment; we append a `volumes/{volume_id}` segment, mirroring
                // the `tables/{uuid}` convention in `create_staging_table`. The id
                // is allocated here and persisted as `volume_id` (the store honors
                // a pre-set id), so the path segment equals the volume's id and
                // survives renames. A caller cannot supply its own location for a
                // managed volume.
                let parent = resolve_managed_parent_location(
                    self,
                    &request.catalog_name,
                    &request.schema_name,
                )
                .await?;
                volume_id = uuid::Uuid::now_v7().hyphenated().to_string();
                child_location(&parent, "volumes", &volume_id)
            }
            VolumeType::VOLUME_TYPE_UNSPECIFIED => {
                return Err(Error::invalid_argument(
                    "volume_type must be specified (EXTERNAL or MANAGED)",
                ));
            }
        };

        let full_name = format!(
            "{}.{}.{}",
            request.catalog_name, request.schema_name, request.name
        );
        let resource = Volume {
            full_name,
            name: request.name,
            catalog_name: request.catalog_name,
            schema_name: request.schema_name,
            volume_type: request.volume_type,
            storage_location,
            volume_id,
            comment: request.comment,
            ..Default::default()
        };
        Ok(self.create(resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context))]
    async fn list_volumes(
        &self,
        request: ListVolumesRequest,
        context: RequestContext,
    ) -> Result<ListVolumesResponse> {
        self.check_required(&request, &context).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::Volume,
                Some(&ResourceName::new([
                    &request.catalog_name,
                    &request.schema_name,
                ])),
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        Ok(ListVolumesResponse {
            volumes: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_volume(
        &self,
        request: GetVolumeRequest,
        context: RequestContext,
    ) -> Result<Volume> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_volume(
        &self,
        request: UpdateVolumeRequest,
        context: RequestContext,
    ) -> Result<Volume> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let ident = request.resource();
        let name = ResourceName::from_naive_str_split(request.name.as_str());
        let [catalog_name, schema_name, volume_name] = name.as_ref() else {
            return Err(Error::invalid_argument(
                "Invalid volume name - expected <catalog_name>.<schema_name>.<volume_name>",
            ));
        };
        let new_name = request.new_name.as_deref().unwrap_or(volume_name);
        let resource = Volume {
            name: new_name.to_owned(),
            comment: request.comment,
            owner: request.owner,
            catalog_name: catalog_name.to_owned(),
            schema_name: schema_name.to_owned(),
            full_name: format!("{}.{}.{}", catalog_name, schema_name, new_name),
            ..Default::default()
        };
        Ok(self.update(&ident, resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_volume(
        &self,
        request: DeleteVolumeRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.delete(&request.resource()).await?)
    }
}

impl SecuredAction for CreateVolumeRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::volume(ResourceName::new([
            self.catalog_name.as_str(),
            self.schema_name.as_str(),
            self.name.as_str(),
        ]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListVolumesRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::volume(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetVolumeRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::volume(ResourceName::from_naive_str_split(self.name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateVolumeRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::volume(ResourceName::from_naive_str_split(self.name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteVolumeRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::volume(ResourceName::from_naive_str_split(self.name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use unitycatalog_common::models::catalogs::v1::{CreateCatalogRequest, GetCatalogRequest};
    use unitycatalog_common::models::credentials::v1::{
        AwsIamRoleConfig, CreateCredentialRequest, Purpose,
    };
    use unitycatalog_common::models::external_locations::v1::CreateExternalLocationRequest;
    use unitycatalog_common::models::schemas::v1::CreateSchemaRequest;
    use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

    use super::*;
    use crate::api::{CatalogHandler, CredentialHandler, ExternalLocationHandler, SchemaHandler};
    use crate::memory::InMemoryResourceStore;
    use crate::policy::ConstantPolicy;
    use crate::services::ServerHandler;

    fn handler() -> ServerHandler<RequestContext> {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
        ServerHandler::try_new_tokio(policy, store.clone(), store).unwrap()
    }

    fn ctx() -> RequestContext {
        RequestContext {
            recipient: crate::policy::Principal::anonymous(),
        }
    }

    /// Register a credential and an external location at `s3://bucket/ext` so
    /// external volumes can be created beneath it.
    async fn setup_external_location(h: &ServerHandler<RequestContext>) {
        h.create_credential(
            CreateCredentialRequest {
                name: "cred".to_string(),
                purpose: ::buffa::EnumValue::Known(Purpose::Storage),
                aws_iam_role: ::buffa::MessageField::some(AwsIamRoleConfig {
                    role_arn: "arn:aws:iam::123456789012:role/test".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_external_location(
            CreateExternalLocationRequest {
                name: "ext".to_string(),
                url: "s3://bucket/ext".to_string(),
                credential_name: "cred".to_string(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
    }

    fn create_external_volume(name: &str, location: Option<&str>) -> CreateVolumeRequest {
        CreateVolumeRequest {
            catalog_name: "cat".to_string(),
            schema_name: "sch".to_string(),
            name: name.to_string(),
            volume_type: ::buffa::EnumValue::Known(VolumeType::External),
            storage_location: location.map(str::to_string),
            comment: None,
            ..Default::default()
        }
    }

    /// Register a credential + external location at `url` so a managed
    /// `storage_root` under it passes the coverage check. `tag` keeps the
    /// credential/location names unique across multiple roots in one test.
    async fn make_covering_location(h: &ServerHandler<RequestContext>, tag: &str, url: &str) {
        h.create_credential(
            CreateCredentialRequest {
                name: format!("{tag}-cred"),
                purpose: ::buffa::EnumValue::Known(Purpose::Storage),
                aws_iam_role: ::buffa::MessageField::some(AwsIamRoleConfig {
                    role_arn: "arn:aws:iam::123456789012:role/test".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_external_location(
            CreateExternalLocationRequest {
                name: format!("{tag}-el"),
                url: url.to_string(),
                credential_name: format!("{tag}-cred"),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
    }

    /// Create catalog `cat` (rooted at `storage_root`) and schema `sch` so a
    /// managed volume can resolve a managed storage root. Registers a covering
    /// external location for the catalog root so create_catalog's coverage check
    /// passes.
    async fn setup_managed_namespace(h: &ServerHandler<RequestContext>, storage_root: &str) {
        make_covering_location(h, "cat", storage_root).await;
        h.create_catalog(
            CreateCatalogRequest {
                name: "cat".to_string(),
                storage_root: Some(storage_root.to_string()),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_schema(
            CreateSchemaRequest {
                name: "sch".to_string(),
                catalog_name: "cat".to_string(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
    }

    fn create_managed_volume(name: &str) -> CreateVolumeRequest {
        CreateVolumeRequest {
            catalog_name: "cat".to_string(),
            schema_name: "sch".to_string(),
            name: name.to_string(),
            volume_type: ::buffa::EnumValue::Known(VolumeType::Managed),
            storage_location: None,
            comment: None,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn managed_volume_location_uses_volume_id() {
        let h = handler();
        setup_managed_namespace(&h, "s3://bucket/cat").await;

        let v = h
            .create_volume(create_managed_volume("vol"), ctx())
            .await
            .unwrap();

        // The path segment equals the persisted volume id, not the name.
        assert!(
            uuid::Uuid::parse_str(&v.volume_id).is_ok(),
            "volume_id should be a uuid, got {}",
            v.volume_id
        );
        // The location nests under the catalog's materialized location, which
        // embeds the catalog id: {root}/__unitystorage/catalogs/<cid>/volumes/<vid>.
        let catalog = h
            .get_catalog(
                GetCatalogRequest {
                    name: "cat".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        let cat_loc = catalog.storage_location.unwrap();
        assert_eq!(
            v.storage_location,
            format!("{cat_loc}/volumes/{}", v.volume_id)
        );
        assert!(
            v.storage_location
                .starts_with("s3://bucket/cat/__unitystorage/catalogs/"),
            "got {}",
            v.storage_location
        );
        assert!(
            !v.storage_location.ends_with("/volumes/vol"),
            "managed volume should not use its name in the path: {}",
            v.storage_location
        );

        // The id round-trips: a get returns the same id and location.
        let got = h
            .get_volume(
                GetVolumeRequest {
                    name: "cat.sch.vol".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(got.volume_id, v.volume_id);
        assert_eq!(got.storage_location, v.storage_location);
    }

    #[tokio::test]
    async fn managed_volume_schema_location_takes_precedence() {
        let h = handler();
        // Both the catalog and schema roots must be covered by registered external
        // locations (they are disjoint, so two distinct locations).
        make_covering_location(&h, "cat", "s3://bucket/cat").await;
        make_covering_location(&h, "sch", "s3://other/sch").await;
        h.create_catalog(
            CreateCatalogRequest {
                name: "cat".to_string(),
                storage_root: Some("s3://bucket/cat".to_string()),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_schema(
            CreateSchemaRequest {
                name: "sch".to_string(),
                catalog_name: "cat".to_string(),
                storage_root: Some("s3://other/sch".to_string()),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();

        let v = h
            .create_volume(create_managed_volume("vol"), ctx())
            .await
            .unwrap();
        // Schema location embeds the schema id and roots under the schema's own
        // storage root, not the catalog's.
        assert!(
            v.storage_location
                .starts_with("s3://other/sch/__unitystorage/schemas/"),
            "got {}",
            v.storage_location
        );
        assert!(
            v.storage_location
                .ends_with(&format!("/volumes/{}", v.volume_id)),
            "got {}",
            v.storage_location
        );
    }

    #[tokio::test]
    async fn managed_volume_recreate_yields_new_path() {
        // Dropping and recreating a managed volume with the same name yields a
        // fresh id and therefore a distinct path — name-based collisions are gone.
        let h = handler();
        setup_managed_namespace(&h, "s3://bucket/cat").await;

        let first = h
            .create_volume(create_managed_volume("vol"), ctx())
            .await
            .unwrap();
        h.delete_volume(
            DeleteVolumeRequest {
                name: "cat.sch.vol".to_string(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        let second = h
            .create_volume(create_managed_volume("vol"), ctx())
            .await
            .unwrap();

        assert_ne!(first.volume_id, second.volume_id);
        assert_ne!(first.storage_location, second.storage_location);
    }

    #[tokio::test]
    async fn managed_volume_resolved_local_location_outside_allowlist_is_rejected() {
        // Managed volumes resolve their parent location lazily at create time and
        // validate it against the local-storage allowlist. The default handler has
        // an empty allowlist, so a catalog whose stored location is a file:// path
        // (written directly, bypassing create-time validation) is rejected.
        use unitycatalog_common::models::catalogs::v1::Catalog;
        let h = handler();
        setup_managed_namespace(&h, "s3://bucket/cat").await;
        let cat_ident = ResourceIdent::catalog(ResourceName::new(["cat"]));
        let local = Catalog {
            name: "cat".to_string(),
            storage_location: Some("file:///forbidden/cat/__unitystorage/catalogs/cid".to_string()),
            ..Default::default()
        };
        h.update(&cat_ident, local.into()).await.unwrap();

        let res = h.create_volume(create_managed_volume("vol"), ctx()).await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn managed_volume_duplicate_name_is_rejected() {
        // Uniqueness on the catalog.schema.name triplet is preserved even though
        // the store now honors a pre-allocated id.
        let h = handler();
        setup_managed_namespace(&h, "s3://bucket/cat").await;

        h.create_volume(create_managed_volume("vol"), ctx())
            .await
            .unwrap();
        let err = h
            .create_volume(create_managed_volume("vol"), ctx())
            .await
            .expect_err("duplicate triplet must be rejected");
        assert_eq!(err.error_code(), "RESOURCE_ALREADY_EXISTS", "{err:?}");
    }

    #[tokio::test]
    async fn external_volume_outside_external_location_is_rejected() {
        let h = handler();
        setup_external_location(&h).await;
        // Path not contained in any external location.
        let res = h
            .create_volume(
                create_external_volume("v", Some("s3://bucket/other/vol")),
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn external_volume_inside_managed_region_is_rejected() {
        // An external volume whose path falls inside a reserved managed-storage
        // (`__unitystorage`) region is rejected even though it is contained by a
        // registered external location — managed and external storage must not
        // be mixed.
        let h = handler();
        setup_external_location(&h).await;
        let res = h
            .create_volume(
                create_external_volume("v", Some("s3://bucket/ext/__unitystorage/volumes/x")),
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn external_volume_within_external_location_succeeds() {
        let h = handler();
        setup_external_location(&h).await;
        let created = h
            .create_volume(
                create_external_volume("v", Some("s3://bucket/ext/vol")),
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(created.storage_location, "s3://bucket/ext/vol");
    }

    #[tokio::test]
    async fn external_volume_overlapping_existing_volume_is_rejected() {
        let h = handler();
        setup_external_location(&h).await;
        h.create_volume(
            create_external_volume("v1", Some("s3://bucket/ext/vol")),
            ctx(),
        )
        .await
        .unwrap();
        // A nested path under an existing volume overlaps it.
        let res = h
            .create_volume(
                create_external_volume("v2", Some("s3://bucket/ext/vol/inner")),
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }
}
