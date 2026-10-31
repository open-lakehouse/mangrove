// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::client::RecipientServiceClient;
use unitycatalog_common::models::recipients::v1::*;
#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::recipients::RecipientHandler<Cx>
    for RecipientServiceClient
{
    async fn list_recipients(
        &self,
        request: ListRecipientsRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListRecipientsResponse> {
        RecipientServiceClient::list_recipients(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn create_recipient(
        &self,
        request: CreateRecipientRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Recipient> {
        RecipientServiceClient::create_recipient(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn get_recipient(
        &self,
        request: GetRecipientRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Recipient> {
        RecipientServiceClient::get_recipient(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn update_recipient(
        &self,
        request: UpdateRecipientRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Recipient> {
        RecipientServiceClient::update_recipient(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn delete_recipient(
        &self,
        request: DeleteRecipientRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        RecipientServiceClient::delete_recipient(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn rotate_recipient_token(
        &self,
        request: RotateRecipientTokenRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Recipient> {
        RecipientServiceClient::rotate_recipient_token(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
