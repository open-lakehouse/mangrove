use itertools::Itertools;

use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::registered_models::v1::*;
use unitycatalog_common::models::{ResourceIdent, ResourceName};

use super::staging_tables::{child_location, resolve_managed_parent_location};
use super::{RequestContext, SecuredAction};
pub use crate::codegen::registered_models::RegisteredModelHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::services::{ProvidesLocalStoragePolicy, ProvidesManagedStorageRoot};
use crate::store::ResourceStore;
use crate::{Error, Result};

#[async_trait::async_trait]
impl<
    T: ResourceStore
        + Policy<RequestContext>
        + ProvidesLocalStoragePolicy
        + ProvidesManagedStorageRoot,
> RegisteredModelHandler<RequestContext> for T
{
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn list_registered_models(
        &self,
        request: ListRegisteredModelsRequest,
        context: RequestContext,
    ) -> Result<ListRegisteredModelsResponse> {
        self.check_required(&request, &context).await?;
        // `catalog_name`/`schema_name` are optional filters. When both are set we
        // scope the listing to that schema; otherwise we list across the broader
        // namespace (matching the UC OSS "list all models" semantics).
        let namespace = match (&request.catalog_name, &request.schema_name) {
            (Some(catalog), Some(schema)) => Some(ResourceName::new([catalog, schema])),
            (Some(catalog), None) => Some(ResourceName::new([catalog])),
            _ => None,
        };
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::RegisteredModel,
                namespace.as_ref(),
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        Ok(ListRegisteredModelsResponse {
            registered_models: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_registered_model(
        &self,
        request: CreateRegisteredModelRequest,
        context: RequestContext,
    ) -> Result<RegisteredModel> {
        self.check_required(&request, &context).await?;
        tracing::Span::current().record("resource_name", &request.name);

        // A registered model owns a managed storage location under which its model
        // versions' artifacts are stored. Derive it from the managed parent location
        // resolved for the schema/catalog (schema → catalog → metastore), mirroring
        // managed volumes/tables. The id is allocated here so the path segment equals
        // the model's id and survives renames.
        let parent =
            resolve_managed_parent_location(self, &request.catalog_name, &request.schema_name)
                .await?;
        let id = uuid::Uuid::now_v7().hyphenated().to_string();
        let storage_location = child_location(&parent, "models", &id);

        let full_name = format!(
            "{}.{}.{}",
            request.catalog_name, request.schema_name, request.name
        );
        let resource = RegisteredModel {
            name: request.name,
            catalog_name: request.catalog_name,
            schema_name: request.schema_name,
            full_name,
            storage_location: Some(storage_location),
            comment: request.comment,
            id: Some(id),
            ..Default::default()
        };
        Ok(self.create(resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_registered_model(
        &self,
        request: GetRegisteredModelRequest,
        context: RequestContext,
    ) -> Result<RegisteredModel> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_registered_model(
        &self,
        request: UpdateRegisteredModelRequest,
        context: RequestContext,
    ) -> Result<RegisteredModel> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        let ident = request.resource();
        let name = ResourceName::from_naive_str_split(request.full_name.as_str());
        let [catalog_name, schema_name, model_name] = name.as_ref() else {
            return Err(Error::invalid_argument(
                "Invalid model name - expected <catalog_name>.<schema_name>.<model_name>",
            ));
        };
        let new_name = request.new_name.as_deref().unwrap_or(model_name);
        let resource = RegisteredModel {
            name: new_name.to_owned(),
            catalog_name: catalog_name.to_owned(),
            schema_name: schema_name.to_owned(),
            full_name: format!("{}.{}.{}", catalog_name, schema_name, new_name),
            comment: request.comment,
            owner: request.owner,
            ..Default::default()
        };
        Ok(self.update(&ident, resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_registered_model(
        &self,
        request: DeleteRegisteredModelRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        Ok(self.delete(&request.resource()).await?)
    }
}

impl SecuredAction for CreateRegisteredModelRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::registered_model(ResourceName::new([
            self.catalog_name.as_str(),
            self.schema_name.as_str(),
            self.name.as_str(),
        ]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListRegisteredModelsRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::registered_model(ResourceName::new(
            [self.catalog_name.as_deref(), self.schema_name.as_deref()]
                .into_iter()
                .flatten(),
        ))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetRegisteredModelRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::registered_model(ResourceName::from_naive_str_split(self.full_name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateRegisteredModelRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::registered_model(ResourceName::from_naive_str_split(self.full_name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteRegisteredModelRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::registered_model(ResourceName::from_naive_str_split(self.full_name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}
