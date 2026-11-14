use itertools::Itertools;

use unitycatalog_common::models::functions::v1::*;
use unitycatalog_common::models::{ObjectLabel, ResourceName};

use super::{RequestContext, RequestResource};
use crate::Result;
use crate::policy::Authorizable;
use crate::policy::{Policy, process_resources};
use crate::services::ServerHandler;
use crate::store::{ResourceStore, ResourceStoreReader};
pub use unitycatalog_handlers::functions::FunctionHandler;

#[async_trait::async_trait]
impl FunctionHandler<RequestContext> for ServerHandler<RequestContext> {
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn list_functions(
        &self,
        request: ListFunctionsRequest,
        context: RequestContext,
    ) -> Result<ListFunctionsResponse> {
        self.check_required(&request, &context).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::Function,
                Some(&ResourceName::new([
                    &request.catalog_name,
                    &request.schema_name,
                ])),
                super::page_size(request.max_results)?,
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &mut resources).await?;
        Ok(ListFunctionsResponse {
            functions: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_function(
        &self,
        request: CreateFunctionRequest,
        context: RequestContext,
    ) -> Result<Function> {
        self.check_required(&request, &context).await?;
        let securable = request.securable();
        let function_info = request
            .function_info
            .ok_or_else(|| crate::Error::invalid_argument("function_info must be provided"))?;
        tracing::Span::current().record("resource_name", &function_info.name);
        let full_name = format!(
            "{}.{}.{}",
            function_info.catalog_name, function_info.schema_name, function_info.name
        );
        let resource = Function {
            name: function_info.name,
            catalog_name: function_info.catalog_name,
            schema_name: function_info.schema_name,
            full_name,
            data_type: function_info.data_type,
            full_data_type: function_info.full_data_type,
            input_params: Some(function_info.input_params.into_option().unwrap_or_default()).into(),
            parameter_style: function_info.parameter_style,
            is_deterministic: function_info.is_deterministic,
            sql_data_access: function_info.sql_data_access,
            is_null_call: function_info.is_null_call,
            security_type: function_info.security_type,
            routine_body: function_info.routine_body,
            routine_definition: function_info.routine_definition,
            external_language: function_info.external_language,
            comment: function_info.comment,
            properties: function_info.properties,
            return_params: function_info.return_params,
            routine_dependencies: function_info.routine_dependencies,
            specific_name: function_info.specific_name,
            ..Default::default()
        };
        let created: Function = self.create(resource.into()).await?.0.try_into()?;
        self.own(&securable, &context).await?;
        Ok(created)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_function(
        &self,
        request: GetFunctionRequest,
        context: RequestContext,
    ) -> Result<Function> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_function(
        &self,
        request: UpdateFunctionRequest,
        context: RequestContext,
    ) -> Result<Function> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let (securable, renamed, owner) =
            (request.securable(), None::<String>, request.owner.clone());
        let ident = request.resource();
        let resource = Function {
            owner: request.owner,
            ..Default::default()
        };
        let updated: Function = self.update(&ident, resource.into()).await?.0.try_into()?;
        self.regrant(&securable, renamed.as_deref(), owner.as_deref())
            .await?;
        Ok(updated)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_function(
        &self,
        request: DeleteFunctionRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        self.delete(&request.resource()).await?;
        self.ungrant(&request.securable()).await
    }
}
