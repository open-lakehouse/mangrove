// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::client::PolicyServiceClient;
use unitycatalog_common::models::policies::v1::*;
#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::policies::PolicyHandler<Cx>
    for PolicyServiceClient
{
    async fn list_policies(
        &self,
        request: ListPoliciesRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListPoliciesResponse> {
        PolicyServiceClient::list_policies(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn create_policy(
        &self,
        request: CreatePolicyRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<PolicyInfo> {
        PolicyServiceClient::create_policy(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn get_policy(
        &self,
        request: GetPolicyRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<PolicyInfo> {
        PolicyServiceClient::get_policy(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn update_policy(
        &self,
        request: UpdatePolicyRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<PolicyInfo> {
        PolicyServiceClient::update_policy(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn delete_policy(
        &self,
        request: DeletePolicyRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        PolicyServiceClient::delete_policy(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
