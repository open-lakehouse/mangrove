use itertools::Itertools;

use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::schemas::v1::*;
use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};

use super::{RequestContext, SecuredAction};
pub use crate::codegen::schemas::SchemaHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::services::ProvidesLocalStoragePolicy;
use crate::services::location::StorageLocationUrl;
use crate::store::ResourceStore;
use crate::{Error, Result};

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext> + ProvidesLocalStoragePolicy>
    SchemaHandler<RequestContext> for T
{
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_schema(
        &self,
        request: CreateSchemaRequest,
        context: RequestContext,
    ) -> Result<Schema> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;

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
        Ok(self.create(resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_schema(
        &self,
        request: DeleteSchemaRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        Ok(self.delete(&request.resource()).await?)
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
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        Ok(ListSchemasResponse {
            schemas: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
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
        Ok(self.update(&ident, resource.into()).await?.0.try_into()?)
    }
}

impl SecuredAction for CreateSchemaRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::schema(ResourceName::new([
            self.catalog_name.as_str(),
            self.name.as_str(),
        ]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListSchemasRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::schema(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetSchemaRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::schema(ResourceName::from_naive_str_split(self.full_name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateSchemaRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::schema(ResourceName::from_naive_str_split(self.full_name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteSchemaRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::schema(ResourceName::from_naive_str_split(self.full_name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
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

    use super::*;
    use crate::api::{CredentialHandler, ExternalLocationHandler};
    use crate::memory::InMemoryResourceStore;
    use crate::policy::{ConstantPolicy, Principal};
    use crate::services::ServerHandler;

    fn handler() -> ServerHandler<RequestContext> {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
        ServerHandler::try_new_tokio(policy, store).unwrap()
    }

    fn ctx() -> RequestContext {
        RequestContext {
            recipient: Principal::anonymous(),
        }
    }

    /// Register a credential + external location at `url` so a managed schema
    /// `storage_root` under it passes the coverage check. (`create_schema` does
    /// not look up the parent catalog, so no catalog needs to exist.)
    async fn make_covering_location(h: &ServerHandler<RequestContext>, name: &str, url: &str) {
        h.create_credential(
            CreateCredentialRequest {
                name: format!("{name}-cred"),
                purpose: Purpose::Storage as i32,
                aws_iam_role: Some(AwsIamRoleConfig {
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
        let schema = h
            .create_schema(create_req("cat", "sch", None), ctx())
            .await
            .unwrap();
        assert!(schema.storage_root.is_none());
        assert!(schema.storage_location.is_none());
    }

    #[tokio::test]
    async fn schema_with_covered_root_succeeds() {
        // A client-supplied root under a registered external location is accepted;
        // the location is materialized under the schema id.
        let h = handler();
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
