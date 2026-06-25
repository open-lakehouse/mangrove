use itertools::Itertools;

use unitycatalog_common::models::catalogs::v1::*;
use unitycatalog_common::models::{ObjectLabel, ResourceIdent, ResourceName, ResourceRef};

use super::{RequestContext, SecuredAction};
pub use crate::codegen::catalogs::CatalogHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::services::location::StorageLocationUrl;
use crate::services::{ProvidesLocalStoragePolicy, ProvidesManagedStorageRoot};
use crate::store::ResourceStore;
use crate::{Error, Result};

#[async_trait::async_trait]
impl<
    T: ResourceStore
        + Policy<RequestContext>
        + ProvidesLocalStoragePolicy
        + ProvidesManagedStorageRoot,
> CatalogHandler<RequestContext> for T
{
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_catalog(
        &self,
        request: CreateCatalogRequest,
        context: RequestContext,
    ) -> Result<Catalog> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;

        // A Delta Sharing catalog references a share on a remote provider; it is
        // identified by `provider_name`/`share_name`, which must be set together
        // and never alongside a managed `storage_root`. These cross-field rules
        // are also expressed as protovalidate CEL on `CreateCatalogRequest` (for
        // client-side form validation), but protovalidate has no Rust runtime —
        // the server re-asserts them here as the authoritative check.
        let is_sharing = request.provider_name.is_some() || request.share_name.is_some();
        if request.provider_name.is_some() != request.share_name.is_some() {
            return Err(Error::invalid_argument(
                "provider_name and share_name must be set together for a Delta Sharing catalog",
            ));
        }
        let has_root = request
            .storage_root
            .as_deref()
            .is_some_and(|s| !s.is_empty());
        if is_sharing && has_root {
            return Err(Error::invalid_argument(
                "a Delta Sharing catalog must not set storage_root",
            ));
        }

        let catalog_type = if is_sharing {
            CatalogType::DeltasharingCatalog
        } else {
            CatalogType::ManagedCatalog
        };

        // A managed catalog must have a resolvable managed storage root: either
        // an explicit `storage_root` on the request, or the metastore-level
        // default. This mirrors Unity Catalog ("if the metastore has no managed
        // storage set, you must set one at the catalog level"). The resolved
        // root is materialized onto the catalog so an inherited metastore root
        // is recorded and table-time resolution finds it directly. Delta Sharing
        // catalogs have no managed storage and are exempt.
        let storage_root = if catalog_type == CatalogType::ManagedCatalog {
            match request.storage_root.filter(|s| !s.is_empty()) {
                // Client-supplied root: it must pass the local-storage policy, lie
                // outside any reserved `__unitystorage` region, and be covered by a
                // registered external location — mirroring the reference's
                // `CatalogService` `AuthorizeExpression`, which requires external
                // location coverage only for a client-supplied root.
                // TODO(auth): also authorize CREATE_MANAGED_STORAGE/OWNER on the
                // covering external location once the policy layer exists
                // (feedback_auth_pattern).
                Some(root) => {
                    let url = StorageLocationUrl::parse(&root)?;
                    crate::services::object_store::validate_managed_storage_root(self, &url)
                        .await?;
                    Some(root)
                }
                // No client root: fall back to the metastore-level managed storage
                // root. That is server configuration, so it is only checked against
                // the local-storage allowlist — it is not required to be covered by
                // a registered external location.
                None => {
                    let root =
                        self.managed_storage_root()
                            .map(str::to_string)
                            .ok_or_else(|| {
                                Error::invalid_argument(format!(
                                    "managed catalog '{}' requires a storage_root, or a metastore \
                                 managed storage root to be configured on the server",
                                    request.name
                                ))
                            })?;
                    // A local (file://) managed root must sit within an allowed host root.
                    self.local_storage_policy()
                        .check(&StorageLocationUrl::parse(&root)?)?;
                    Some(root)
                }
            }
        } else {
            None
        };

        // Pre-allocate the catalog id so the managed storage location can embed
        // it (`<root>/__unitystorage/catalogs/<id>`), mirroring the reference's
        // `CatalogRepository`. The store honors a pre-set id (else it mints a
        // v7). Managed catalogs with a resolvable root get a materialized
        // `storage_location`; sharing catalogs have no managed storage.
        let id = uuid::Uuid::now_v7().hyphenated().to_string();
        let storage_location = storage_root
            .as_deref()
            .map(|root| super::staging_tables::catalog_location(root, &id));

        let resource = Catalog {
            id: Some(id),
            name: request.name,
            comment: request.comment,
            properties: request.properties,
            storage_root,
            storage_location,
            provider_name: request.provider_name,
            share_name: request.share_name,
            catalog_type: Some(::buffa::EnumValue::Known(catalog_type)),
            ..Default::default()
        };
        let info = self.create(resource.into()).await?.0.try_into()?;

        // TODO:
        // - make current actor the owner of the catalog including permissions
        // - create updated_* relations

        Ok(info)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_catalog(
        &self,
        request: DeleteCatalogRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.delete(&request.resource()).await?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_catalog(
        &self,
        request: GetCatalogRequest,
        context: RequestContext,
    ) -> Result<Catalog> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context))]
    async fn list_catalogs(
        &self,
        request: ListCatalogsRequest,
        context: RequestContext,
    ) -> Result<ListCatalogsResponse> {
        self.check_required(&request, &context).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::Catalog,
                None,
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        Ok(ListCatalogsResponse {
            catalogs: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_catalog(
        &self,
        request: UpdateCatalogRequest,
        context: RequestContext,
    ) -> Result<Catalog> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let ident = request.resource();
        let resource = Catalog {
            name: request.new_name.unwrap_or(request.name),
            comment: request.comment,
            properties: request.properties,
            ..Default::default()
        };
        // TODO:
        // - add update_* relations
        // - update owner if necessary
        Ok(self.update(&ident, resource.into()).await?.0.try_into()?)
    }
}

impl SecuredAction for CreateCatalogRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::catalog(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListCatalogsRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::catalog(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetCatalogRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::catalog(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateCatalogRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::catalog(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteCatalogRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::catalog(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

    use unitycatalog_common::models::credentials::v1::{
        AwsIamRoleConfig, CreateCredentialRequest, Purpose,
    };
    use unitycatalog_common::models::external_locations::v1::CreateExternalLocationRequest;

    use super::*;
    use crate::api::{CredentialHandler, ExternalLocationHandler};
    use crate::memory::InMemoryResourceStore;
    use crate::policy::{ConstantPolicy, Principal};
    use crate::services::{LocalStoragePolicy, ServerHandler};

    /// Build a handler with an optional metastore managed storage root. When
    /// `allowed_root` is set, local (file://) storage beneath it is permitted.
    fn handler(
        metastore_root: Option<&str>,
        allowed_root: Option<&std::path::Path>,
    ) -> ServerHandler<RequestContext> {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
        let mut h = ServerHandler::try_new_tokio(policy, store.clone(), store).unwrap();
        if let Some(root) = allowed_root {
            h = h.with_local_storage_policy(LocalStoragePolicy::new([root]).unwrap());
        }
        h.with_managed_storage_root(metastore_root.map(str::to_string))
    }

    fn ctx() -> RequestContext {
        RequestContext {
            recipient: Principal::anonymous(),
        }
    }

    fn create_req(name: &str) -> CreateCatalogRequest {
        CreateCatalogRequest {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Register a credential and an external location at `url` so a managed
    /// `storage_root` under it passes the external-location coverage check. The
    /// credential name is derived from `name` (external-location create resolves
    /// `credential_name` → `credential_id`, so the credential must exist first).
    async fn make_covering_location(h: &ServerHandler<RequestContext>, name: &str, url: &str) {
        h.create_credential(
            CreateCredentialRequest {
                name: format!("{name}-cred"),
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
                name: name.to_string(),
                url: url.to_string(),
                credential_name: format!("{name}-cred"),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn managed_catalog_with_explicit_root_persists_it() {
        let h = handler(None, None);
        make_covering_location(&h, "el", "s3://bucket/cat").await;
        let cat = h
            .create_catalog(
                CreateCatalogRequest {
                    storage_root: Some("s3://bucket/cat".to_string()),
                    ..create_req("cat")
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(cat.storage_root.as_deref(), Some("s3://bucket/cat"));
        assert_eq!(
            cat.catalog_type,
            Some(::buffa::EnumValue::Known(CatalogType::ManagedCatalog))
        );
        // The managed storage location is materialized with the catalog id:
        // <root>/__unitystorage/catalogs/<id>.
        let id = cat.id.as_deref().expect("catalog should have an id");
        assert_eq!(
            cat.storage_location.as_deref(),
            Some(format!("s3://bucket/cat/__unitystorage/catalogs/{id}").as_str())
        );
    }

    #[tokio::test]
    async fn managed_catalog_without_root_and_no_metastore_default_is_rejected() {
        let h = handler(None, None);
        let res = h.create_catalog(create_req("cat"), ctx()).await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn managed_catalog_inherits_metastore_default() {
        let h = handler(Some("s3://bucket/meta"), None);
        let cat = h.create_catalog(create_req("cat"), ctx()).await.unwrap();
        // The inherited metastore root is materialized onto the catalog, and the
        // storage location nests under it with the catalog id.
        assert_eq!(cat.storage_root.as_deref(), Some("s3://bucket/meta"));
        let id = cat.id.as_deref().expect("catalog should have an id");
        assert_eq!(
            cat.storage_location.as_deref(),
            Some(format!("s3://bucket/meta/__unitystorage/catalogs/{id}").as_str())
        );
    }

    #[tokio::test]
    async fn explicit_root_takes_precedence_over_metastore_default() {
        let h = handler(Some("s3://bucket/meta"), None);
        make_covering_location(&h, "el", "s3://bucket/explicit").await;
        let cat = h
            .create_catalog(
                CreateCatalogRequest {
                    storage_root: Some("s3://bucket/explicit".to_string()),
                    ..create_req("cat")
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(cat.storage_root.as_deref(), Some("s3://bucket/explicit"));
    }

    #[tokio::test]
    async fn sharing_catalog_without_root_is_allowed() {
        let h = handler(None, None);
        let cat = h
            .create_catalog(
                CreateCatalogRequest {
                    provider_name: Some("prov".to_string()),
                    share_name: Some("shr".to_string()),
                    ..create_req("cat")
                },
                ctx(),
            )
            .await
            .unwrap();
        assert!(cat.storage_root.is_none());
        // A sharing catalog has no managed storage, so no location is materialized.
        assert!(cat.storage_location.is_none());
        assert_eq!(
            cat.catalog_type,
            Some(::buffa::EnumValue::Known(CatalogType::DeltasharingCatalog))
        );
    }

    #[tokio::test]
    async fn sharing_catalog_with_storage_root_is_rejected() {
        let h = handler(None, None);
        let res = h
            .create_catalog(
                CreateCatalogRequest {
                    provider_name: Some("prov".to_string()),
                    share_name: Some("shr".to_string()),
                    storage_root: Some("s3://bucket/cat".to_string()),
                    ..create_req("cat")
                },
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn provider_without_share_is_rejected() {
        let h = handler(None, None);
        let res = h
            .create_catalog(
                CreateCatalogRequest {
                    provider_name: Some("prov".to_string()),
                    ..create_req("cat")
                },
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn explicit_root_covered_by_external_location_succeeds() {
        // A client-supplied root nested under a registered external location is
        // accepted; the location is materialized under the catalog id.
        let h = handler(None, None);
        make_covering_location(&h, "el", "s3://bucket").await;
        let cat = h
            .create_catalog(
                CreateCatalogRequest {
                    storage_root: Some("s3://bucket/cat".to_string()),
                    ..create_req("cat")
                },
                ctx(),
            )
            .await
            .unwrap();
        let id = cat.id.as_deref().expect("catalog should have an id");
        assert_eq!(
            cat.storage_location.as_deref(),
            Some(format!("s3://bucket/cat/__unitystorage/catalogs/{id}").as_str())
        );
    }

    #[tokio::test]
    async fn explicit_root_without_external_location_is_rejected() {
        // No registered external location covers the client-supplied root ⇒ reject.
        let h = handler(None, None);
        let res = h
            .create_catalog(
                CreateCatalogRequest {
                    storage_root: Some("s3://bucket/cat".to_string()),
                    ..create_req("cat")
                },
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn explicit_root_under_managed_prefix_is_rejected() {
        // A client root inside a reserved `__unitystorage` region is rejected even
        // when an external location covers it — the server owns that layout.
        let h = handler(None, None);
        make_covering_location(&h, "el", "s3://bucket").await;
        let res = h
            .create_catalog(
                CreateCatalogRequest {
                    storage_root: Some("s3://bucket/__unitystorage/cat".to_string()),
                    ..create_req("cat")
                },
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn metastore_default_root_without_external_location_succeeds() {
        // The metastore-level default is server config and is exempt from the
        // external-location coverage requirement — no EL registered, still works.
        let h = handler(Some("s3://bucket/meta"), None);
        let cat = h.create_catalog(create_req("cat"), ctx()).await.unwrap();
        assert_eq!(cat.storage_root.as_deref(), Some("s3://bucket/meta"));
    }

    #[tokio::test]
    async fn local_root_outside_allowlist_is_rejected() {
        // No allowed roots ⇒ deny all file://.
        let h = handler(None, None);
        let res = h
            .create_catalog(
                CreateCatalogRequest {
                    storage_root: Some("file:///tmp/not-allowed".to_string()),
                    ..create_req("cat")
                },
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }
}
