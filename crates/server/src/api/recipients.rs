use itertools::Itertools;

use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::recipients::v1::*;
use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};

use super::{RequestContext, SecuredAction};
use crate::Result;
pub use crate::codegen::recipients::RecipientHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::store::ResourceStore;

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext>> RecipientHandler<RequestContext> for T {
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_recipient(
        &self,
        request: CreateRecipientRequest,
        context: RequestContext,
    ) -> Result<Recipient> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let resource = Recipient {
            name: request.name,
            authentication_type: request.authentication_type,
            comment: request.comment,
            properties: request.properties,
            ..Default::default()
        };

        // When authentication_type is TOKEN, an initial RecipientToken should be created
        // here and stored via the SecretManager.  The token's activation_url would be
        // returned to the caller so they can share it with the recipient.  This requires
        // a SecretManager to be accessible from the handler — once that is available,
        // generate a token, store it with the given expiration_time, and embed it in the
        // returned Recipient's `tokens` field.  Until then, TOKEN-type recipients are
        // created without a pre-generated token; they can be activated out-of-band.

        let info = self.create(resource.into()).await?.0.try_into()?;
        Ok(info)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_recipient(
        &self,
        request: DeleteRecipientRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.delete(&request.resource()).await?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_recipient(
        &self,
        request: GetRecipientRequest,
        context: RequestContext,
    ) -> Result<Recipient> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context))]
    async fn list_recipients(
        &self,
        request: ListRecipientsRequest,
        context: RequestContext,
    ) -> Result<ListRecipientsResponse> {
        self.check_required(&request, &context).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::Recipient,
                None,
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        Ok(ListRecipientsResponse {
            recipients: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_recipient(
        &self,
        request: UpdateRecipientRequest,
        context: RequestContext,
    ) -> Result<Recipient> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let ident = request.resource();
        let current: Recipient = self.get(&ident).await?.0.try_into()?;

        // Apply the mutable fields from the request onto the existing recipient.
        // Token expiration updates (request.expiration_time) are acknowledged here but
        // cannot yet be persisted: the `Recipient` protobuf stores tokens as
        // `Vec<RecipientToken>` and token lifecycle management (create / rotate / expire)
        // requires a SecretManager, which is not available through the ResourceStore +
        // Policy bound used by RecipientHandler.  Token handling should be added once
        // a dedicated token-management service is wired into the handler context.
        let updated = Recipient {
            name: request.new_name.unwrap_or(request.name),
            owner: request.owner.or(current.owner),
            comment: request.comment.or(current.comment),
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

impl SecuredAction for CreateRecipientRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::recipient(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListRecipientsRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::recipient(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetRecipientRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::recipient(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateRecipientRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::recipient(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteRecipientRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::recipient(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}
