use itertools::Itertools;

use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::schemas::v1::*;
use unitycatalog_common::models::{ResourceIdent, ResourceName};

use super::{RequestContext, RequestResource};
use crate::policy::Authorizable;
use crate::policy::{Policy, process_resources};
use crate::services::ServerHandler;
use crate::services::location::StorageLocationUrl;
use crate::store::{ResourceStore, ResourceStoreReader};
use crate::{Error, Result};
pub use unitycatalog_handlers::schemas::SchemaHandler;

#[async_trait::async_trait]
impl SchemaHandler<RequestContext> for ServerHandler<RequestContext> {
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_schema(
        &self,
        request: CreateSchemaRequest,
        context: RequestContext,
    ) -> Result<Schema> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let securable = request.securable();

        // The parent catalog must exist. Check it before the storage-root
        // validation and the insert, so a refused request stores nothing.
        // TODO: the reference server answers `CATALOG_DOES_NOT_EXIST` here; we
        // answer the generic NOT_FOUND (`RESOURCE_NOT_FOUND`) until that error
        // code exists.
        let catalog = ResourceIdent::catalog(ResourceName::new([&request.catalog_name]));
        self.get(&catalog).await.map_err(|e| match e {
            unitycatalog_common::Error::NotFound => Error::NotFound,
            other => other.into(),
        })?;

        // When the schema is created with its own storage root, echo the root
        // back on the schema (`storage_root`) and materialize a managed storage
        // location that embeds the schema id (`storage_location` =
        // `<root>/__unitystorage/schemas/<schema_id>`), mirroring the reference's
        // `SchemaRepository`/`SchemaInfo`. The location is built from the schema's
        // own root, independent of the parent catalog. The id is pre-allocated so
        // the store persists the row under it (else a v7 is minted). When no root
        // is provided, both fields stay empty and managed securables fall back to
        // the parent catalog's storage location.
        let schema_id = uuid::Uuid::now_v7().hyphenated().to_string();
        let storage_root = request.storage_root.filter(|s| !s.is_empty());
        let storage_location = match storage_root.as_deref() {
            // A client-supplied schema root must pass the local-storage policy, lie
            // outside any reserved `__unitystorage` region, and be covered by a
            // registered external location — mirroring the reference's
            // `SchemaService` `AuthorizeExpression`.
            // TODO(auth): also authorize CREATE_MANAGED_STORAGE/OWNER on the
            // covering external location once the policy layer exists
            // (feedback_auth_pattern).
            Some(root) => {
                let url = StorageLocationUrl::parse(root)?;
                crate::services::object_store::validate_managed_storage_root(self, &url).await?;
                Some(super::staging_tables::schema_location(root, &schema_id))
            }
            None => None,
        };

        let resource = Schema {
            schema_id: Some(schema_id),
            full_name: format!("{}.{}", request.catalog_name, request.name),
            name: request.name,
            catalog_name: request.catalog_name,
            comment: request.comment,
            properties: request.properties,
            storage_root,
            storage_location,
            ..Default::default()
        };
        // TODO:
        // - update the schema with the current actor as owner
        // - create updated_* relations
        let created: Schema = self.create(resource.into()).await?.0.try_into()?;
        self.own(&securable, &context).await?;
        Ok(created)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_schema(
        &self,
        request: DeleteSchemaRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        self.delete(&request.resource()).await?;
        self.ungrant(&request.securable()).await
    }

    #[tracing::instrument(skip(self, context))]
    async fn list_schemas(
        &self,
        request: ListSchemasRequest,
        context: RequestContext,
    ) -> Result<ListSchemasResponse> {
        self.check_required(&request, &context).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::Schema,
                Some(&ResourceName::new([&request.catalog_name])),
                super::page_size(request.max_results)?,
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &mut resources).await?;
        Ok(ListSchemasResponse {
            schemas: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_schema(
        &self,
        request: GetSchemaRequest,
        context: RequestContext,
    ) -> Result<Schema> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_schema(
        &self,
        request: UpdateSchemaRequest,
        context: RequestContext,
    ) -> Result<Schema> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        let (securable, renamed, owner) = (
            request.securable(),
            request.new_name.clone(),
            None::<String>,
        );
        let ident = request.resource();
        let name = ResourceName::from_naive_str_split(request.full_name);
        let [catalog_name, schema_name] = name.as_ref() else {
            return Err(Error::invalid_argument(
                "Invalid schema name - expected <catalog_name>.<schema_name>",
            ));
        };
        let new_name = request.new_name.unwrap_or(schema_name.to_owned());
        let resource = Schema {
            name: new_name.clone(),
            comment: request.comment,
            properties: request.properties,
            catalog_name: catalog_name.to_owned(),
            full_name: format!("{}.{}", catalog_name, new_name),
            ..Default::default()
        };
        // TODO:
        // - add update_* relations
        // - update owner if necessary
        let updated: Schema = self.update(&ident, resource.into()).await?.0.try_into()?;
        self.regrant(&securable, renamed.as_deref(), owner.as_deref())
            .await?;
        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use unitycatalog_common::models::credentials::v1::{
        AwsIamRoleConfig, CreateCredentialRequest, Purpose,
    };
    use unitycatalog_common::models::external_locations::v1::CreateExternalLocationRequest;
    use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

    use unitycatalog_common::models::catalogs::v1::Catalog;

    use super::*;
    use crate::api::{CredentialHandler, ExternalLocationHandler};
    use crate::memory::InMemoryResourceStore;
    use crate::policy::ConstantPolicy;
    use crate::services::ServerHandler;

    fn handler() -> ServerHandler<RequestContext> {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
        ServerHandler::try_new_tokio(policy, store).unwrap()
    }

    fn ctx() -> RequestContext {
        RequestContext::anonymous()
    }

    /// Store a bare catalog `name` directly, so `create_schema` finds its parent.
    /// (The catalog handler would require a managed storage root; the schema
    /// tests only need the catalog to exist.)
    async fn make_catalog(h: &ServerHandler<RequestContext>, name: &str) {
        let catalog = Catalog {
            name: name.to_string(),
            ..Default::default()
        };
        h.create(catalog.into()).await.unwrap();
    }

    /// Register a credential + external location at `url` so a managed schema
    /// `storage_root` under it passes the coverage check. (The parent catalog
    /// is created separately with [`make_catalog`].)
    async fn make_covering_location(h: &ServerHandler<RequestContext>, name: &str, url: &str) {
        h.create_credential(
            CreateCredentialRequest {
                name: format!("{name}-cred"),
                purpose: Purpose::Storage.into(),
                aws_iam_role: Some(AwsIamRoleConfig {
                    role_arn: "arn:aws:iam::123456789012:role/test".to_string(),
                    ..Default::default()
                })
                .into(),
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

    fn create_req(catalog: &str, name: &str, storage_root: Option<&str>) -> CreateSchemaRequest {
        CreateSchemaRequest {
            name: name.to_string(),
            catalog_name: catalog.to_string(),
            storage_root: storage_root.map(str::to_string),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn schema_without_root_has_no_storage_fields() {
        // No storage_root ⇒ no coverage check, no materialized location; managed
        // securables fall back to the catalog at table-create time.
        let h = handler();
        make_catalog(&h, "cat").await;
        let schema = h
            .create_schema(create_req("cat", "sch", None), ctx())
            .await
            .unwrap();
        assert!(schema.storage_root.is_none());
        assert!(schema.storage_location.is_none());
    }

    #[tokio::test]
    async fn schema_in_existing_catalog_is_stored() {
        // The parent catalog exists ⇒ the schema is created and can be read back.
        let h = handler();
        make_catalog(&h, "cat").await;
        let created = h
            .create_schema(create_req("cat", "sch", None), ctx())
            .await
            .unwrap();
        assert_eq!(created.full_name, "cat.sch");
        assert_eq!(created.catalog_name, "cat");
        let fetched = h
            .get_schema(
                GetSchemaRequest {
                    full_name: "cat.sch".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(fetched.schema_id, created.schema_id);
    }

    #[rstest::rstest]
    #[case::no_root(None)]
    #[case::with_root(Some("s3://bucket/sch"))]
    #[tokio::test]
    async fn schema_in_missing_catalog_is_not_found_and_not_stored(
        #[case] storage_root: Option<&str>,
    ) {
        // No parent catalog ⇒ NOT_FOUND, checked before the storage root, and
        // nothing is stored (issue #372).
        let h = handler();
        make_covering_location(&h, "el", "s3://bucket").await;
        let res = h
            .create_schema(create_req("missing", "sch", storage_root), ctx())
            .await;
        assert!(matches!(res, Err(Error::NotFound)), "{res:?}");

        let got = h
            .get_schema(
                GetSchemaRequest {
                    full_name: "missing.sch".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await;
        assert!(
            matches!(
                got,
                Err(Error::NotFound
                    | Error::Common {
                        source: unitycatalog_common::Error::NotFound
                    })
            ),
            "{got:?}"
        );
        let listed = h
            .list_schemas(
                ListSchemasRequest {
                    catalog_name: "missing".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert!(listed.schemas.is_empty(), "{listed:?}");
    }

    #[tokio::test]
    async fn schema_with_covered_root_succeeds() {
        // A client-supplied root under a registered external location is accepted;
        // the location is materialized under the schema id.
        let h = handler();
        make_catalog(&h, "cat").await;
        make_covering_location(&h, "el", "s3://bucket").await;
        let schema = h
            .create_schema(create_req("cat", "sch", Some("s3://bucket/sch")), ctx())
            .await
            .unwrap();
        assert_eq!(schema.storage_root.as_deref(), Some("s3://bucket/sch"));
        let id = schema.schema_id.as_deref().expect("schema id");
        assert_eq!(
            schema.storage_location.as_deref(),
            Some(format!("s3://bucket/sch/__unitystorage/schemas/{id}").as_str())
        );
    }

    #[tokio::test]
    async fn schema_with_uncovered_root_is_rejected() {
        // No registered external location covers the root ⇒ reject.
        let h = handler();
        make_catalog(&h, "cat").await;
        let res = h
            .create_schema(create_req("cat", "sch", Some("s3://bucket/sch")), ctx())
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn schema_root_under_managed_prefix_is_rejected() {
        // A root inside a reserved `__unitystorage` region is rejected even when an
        // external location covers it.
        let h = handler();
        make_catalog(&h, "cat").await;
        make_covering_location(&h, "el", "s3://bucket").await;
        let res = h
            .create_schema(
                create_req("cat", "sch", Some("s3://bucket/__unitystorage/sch")),
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }
}
