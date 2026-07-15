use itertools::Itertools;

use unitycatalog_common::models::functions::v1::*;
use unitycatalog_common::models::{ObjectLabel, ResourceIdent, ResourceName};

use super::{RequestContext, SecuredAction};
use crate::Result;
pub use crate::codegen::functions::FunctionHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::store::ResourceStore;

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext>> FunctionHandler<RequestContext> for T {
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
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
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
            routine_body_language: function_info.routine_body_language,
            comment: function_info.comment,
            properties: function_info.properties,
            ..Default::default()
        };
        Ok(self.create(resource.into()).await?.0.try_into()?)
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
        let ident = request.resource();
        let resource = Function {
            owner: request.owner,
            ..Default::default()
        };
        Ok(self.update(&ident, resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_function(
        &self,
        request: DeleteFunctionRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.delete(&request.resource()).await?)
    }
}

impl SecuredAction for CreateFunctionRequest {
    fn resource(&self) -> ResourceIdent {
        let info = self.function_info.as_option();
        ResourceIdent::function(ResourceName::new([
            info.map(|i| i.catalog_name.as_str()).unwrap_or_default(),
            info.map(|i| i.schema_name.as_str()).unwrap_or_default(),
            info.map(|i| i.name.as_str()).unwrap_or_default(),
        ]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListFunctionsRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::function(ResourceName::new([
            self.catalog_name.as_str(),
            self.schema_name.as_str(),
        ]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetFunctionRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::function(ResourceName::from_naive_str_split(self.name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateFunctionRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::function(ResourceName::from_naive_str_split(self.name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteFunctionRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::function(ResourceName::from_naive_str_split(self.name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}
