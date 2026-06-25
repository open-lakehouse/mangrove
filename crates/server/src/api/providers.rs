use itertools::Itertools;

use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::providers::v1::*;
use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};

use super::{RequestContext, SecuredAction};
use crate::Result;
pub use crate::codegen::providers::ProviderHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::store::ResourceStore;

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext>> ProviderHandler<RequestContext> for T {
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_provider(
        &self,
        request: CreateProviderRequest,
        context: RequestContext,
    ) -> Result<Provider> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let resource = Provider {
            name: request.name,
            authentication_type: request.authentication_type,
            owner: request.owner,
            comment: request.comment,
            recipient_profile_str: request.recipient_profile_str,
            properties: request.properties,
            ..Default::default()
        };
        let info = self.create(resource.into()).await?.0.try_into()?;
        Ok(info)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_provider(
        &self,
        request: DeleteProviderRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.delete(&request.resource()).await?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_provider(
        &self,
        request: GetProviderRequest,
        context: RequestContext,
    ) -> Result<Provider> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context))]
    async fn list_providers(
        &self,
        request: ListProvidersRequest,
        context: RequestContext,
    ) -> Result<ListProvidersResponse> {
        self.check_required(&request, &context).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::Provider,
                None,
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        Ok(ListProvidersResponse {
            providers: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_provider(
        &self,
        request: UpdateProviderRequest,
        context: RequestContext,
    ) -> Result<Provider> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let ident = request.resource();
        let current: Provider = self.get(&ident).await?.0.try_into()?;

        let updated = Provider {
            name: request.new_name.unwrap_or(request.name),
            owner: request.owner.or(current.owner),
            comment: request.comment.or(current.comment),
            recipient_profile_str: request
                .recipient_profile_str
                .or(current.recipient_profile_str),
            properties: if request.properties.is_empty() {
                current.properties
            } else {
                request.properties
            },
            // Preserve all fields managed by the store.
            ..current
        };

        Ok(self.update(&ident, updated.into()).await?.0.try_into()?)
    }
}

impl SecuredAction for CreateProviderRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::provider(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListProvidersRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::provider(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetProviderRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::provider(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateProviderRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::provider(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteProviderRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::provider(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}
