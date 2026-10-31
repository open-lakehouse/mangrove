// @generated — do not edit by hand.
#![allow(unused_imports)]

use super::client::TagPolicyServiceClient;
use unitycatalog_common::models::tags::v1::*;

#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::tag_policies::TagPolicyHandler<Cx>
    for TagPolicyServiceClient
{
    async fn list_tag_policies(
        &self,
        request: ListTagPoliciesRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListTagPoliciesResponse> {
        TagPolicyServiceClient::list_tag_policies(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn create_tag_policy(
        &self,
        request: CreateTagPolicyRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<TagPolicy> {
        TagPolicyServiceClient::create_tag_policy(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn get_tag_policy(
        &self,
        request: GetTagPolicyRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<TagPolicy> {
        TagPolicyServiceClient::get_tag_policy(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn update_tag_policy(
        &self,
        request: UpdateTagPolicyRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<TagPolicy> {
        TagPolicyServiceClient::update_tag_policy(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn delete_tag_policy(
        &self,
        request: DeleteTagPolicyRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        TagPolicyServiceClient::delete_tag_policy(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
