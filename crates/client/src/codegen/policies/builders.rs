// @generated — do not edit by hand.
#![allow(unused_mut)]
#![allow(unused_imports)]
#[cfg(not(target_arch = "wasm32"))]
type BoxFut<'a, T> = ::futures::future::BoxFuture<'a, T>;
#[cfg(target_arch = "wasm32")]
type BoxFut<'a, T> = ::futures::future::LocalBoxFuture<'a, T>;
use super::client::*;
use crate::Result;
use std::future::IntoFuture;
use unitycatalog_common::models::policies::v1::*;
/// Builder for policies
pub struct ListPoliciesBuilder {
    client: PolicyServiceClient,
    request: ListPoliciesRequest,
}
impl ListPoliciesBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `PolicyServiceClient`.
    pub(crate) fn new(
        client: PolicyServiceClient,
        on_securable_type: impl Into<String>,
        on_securable_fullname: impl Into<String>,
    ) -> Self {
        let request = ListPoliciesRequest {
            on_securable_type: on_securable_type.into(),
            on_securable_fullname: on_securable_fullname.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /** When true, also return policies defined on the securable's ancestors
    (e.g. for a table: its schema and catalog). Each returned PolicyInfo still
    carries its own on_securable_type / on_securable_fullname, so callers can see
    where it was defined.*/
    pub fn with_include_inherited(mut self, include_inherited: impl Into<Option<bool>>) -> Self {
        self.request.include_inherited = include_inherited.into();
        self
    }
    /// The maximum number of results per page that should be returned.
    pub fn with_max_results(mut self, max_results: impl Into<Option<i32>>) -> Self {
        self.request.max_results = max_results.into();
        self
    }
    /// Opaque pagination token to go to next page based on previous query.
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
    }
}
impl IntoFuture for ListPoliciesBuilder {
    type Output = Result<ListPoliciesResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_policies(&request).await })
    }
}
/// Builder for policy
pub struct CreatePolicyBuilder {
    client: PolicyServiceClient,
    request: CreatePolicyRequest,
}
impl CreatePolicyBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `PolicyServiceClient`.
    pub(crate) fn new(
        client: PolicyServiceClient,
        on_securable_type: impl Into<String>,
        on_securable_fullname: impl Into<String>,
        policy_info: PolicyInfo,
    ) -> Self {
        let request = CreatePolicyRequest {
            on_securable_type: on_securable_type.into(),
            on_securable_fullname: on_securable_fullname.into(),
            policy_info: buffa::MessageField::some(policy_info),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for CreatePolicyBuilder {
    type Output = Result<PolicyInfo>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_policy(&request).await })
    }
}
/// Builder for getting a policy
pub struct GetPolicyBuilder {
    client: PolicyServiceClient,
    request: GetPolicyRequest,
}
impl GetPolicyBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `PolicyServiceClient`.
    pub(crate) fn new(
        client: PolicyServiceClient,
        on_securable_type: impl Into<String>,
        on_securable_fullname: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        let request = GetPolicyRequest {
            on_securable_type: on_securable_type.into(),
            on_securable_fullname: on_securable_fullname.into(),
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GetPolicyBuilder {
    type Output = Result<PolicyInfo>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_policy(&request).await })
    }
}
/// Builder for updating a policy
pub struct UpdatePolicyBuilder {
    client: PolicyServiceClient,
    request: UpdatePolicyRequest,
}
impl UpdatePolicyBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `PolicyServiceClient`.
    pub(crate) fn new(
        client: PolicyServiceClient,
        on_securable_type: impl Into<String>,
        on_securable_fullname: impl Into<String>,
        name: impl Into<String>,
        policy_info: PolicyInfo,
    ) -> Self {
        let request = UpdatePolicyRequest {
            on_securable_type: on_securable_type.into(),
            on_securable_fullname: on_securable_fullname.into(),
            name: name.into(),
            policy_info: buffa::MessageField::some(policy_info),
            ..Default::default()
        };
        Self { client, request }
    }
    /// The list of fields to update, as a comma-separated string.
    pub fn with_update_mask(mut self, update_mask: impl Into<Option<String>>) -> Self {
        self.request.update_mask = update_mask.into();
        self
    }
}
impl IntoFuture for UpdatePolicyBuilder {
    type Output = Result<PolicyInfo>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_policy(&request).await })
    }
}
/// Builder for deleting a policy
pub struct DeletePolicyBuilder {
    client: PolicyServiceClient,
    request: DeletePolicyRequest,
}
impl DeletePolicyBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `PolicyServiceClient`.
    pub(crate) fn new(
        client: PolicyServiceClient,
        on_securable_type: impl Into<String>,
        on_securable_fullname: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        let request = DeletePolicyRequest {
            on_securable_type: on_securable_type.into(),
            on_securable_fullname: on_securable_fullname.into(),
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for DeletePolicyBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_policy(&request).await })
    }
}
