// @generated — do not edit by hand.
#![allow(unused_imports)]

use super::client::CredentialServiceClient;
use unitycatalog_common::models::credentials::v1::*;

#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::credentials::CredentialHandler<Cx>
    for CredentialServiceClient
{
    async fn list_credentials(
        &self,
        request: ListCredentialsRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListCredentialsResponse> {
        CredentialServiceClient::list_credentials(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn create_credential(
        &self,
        request: CreateCredentialRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Credential> {
        CredentialServiceClient::create_credential(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn get_credential(
        &self,
        request: GetCredentialRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Credential> {
        CredentialServiceClient::get_credential(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn update_credential(
        &self,
        request: UpdateCredentialRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Credential> {
        CredentialServiceClient::update_credential(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn delete_credential(
        &self,
        request: DeleteCredentialRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        CredentialServiceClient::delete_credential(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
