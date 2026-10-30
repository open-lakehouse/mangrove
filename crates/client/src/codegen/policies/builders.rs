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
    /** The maximum number of results per page that should be returned.

    If unset or 0, the server's default page size is used. Negative values are rejected.*/
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
/// Builder for creating a policy
pub struct CreatePolicyBuilder {
    client: PolicyServiceClient,
    request: CreatePolicyRequest,
}
impl CreatePolicyBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `PolicyServiceClient`.
    pub(crate) fn new(
        client: PolicyServiceClient,
        name: impl Into<String>,
        on_securable_type: impl Into<String>,
        on_securable_fullname: impl Into<String>,
        policy_type: PolicyType,
    ) -> Self {
        let request = CreatePolicyRequest {
            name: name.into(),
            on_securable_type: on_securable_type.into(),
            on_securable_fullname: on_securable_fullname.into(),
            policy_type: buffa::EnumValue::Known(policy_type),
            ..Default::default()
        };
        Self { client, request }
    }
    /// User-provided free-form text description of the policy.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /// The principals the policy applies to. Empty means "all principals".
    pub fn with_to_principals<I>(mut self, to_principals: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        self.request.to_principals = to_principals.into_iter().collect();
        self
    }
    /// The principals explicitly excluded from the policy, even if matched by to_principals.
    pub fn with_except_principals<I>(mut self, except_principals: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        self.request.except_principals = except_principals.into_iter().collect();
        self
    }
    /// The type of securables the policy takes effect on.
    pub fn with_for_securable_type(
        mut self,
        for_securable_type: impl Into<Option<String>>,
    ) -> Self {
        self.request.for_securable_type = for_securable_type.into();
        self
    }
    /// An opaque condition expression evaluated by the query engine at enforcement time.
    pub fn with_when_condition(mut self, when_condition: impl Into<Option<String>>) -> Self {
        self.request.when_condition = when_condition.into();
        self
    }
    /// The columns referenced by the policy function, and the aliases they are bound to.
    pub fn with_match_columns<I>(mut self, match_columns: I) -> Self
    where
        I: IntoIterator<Item = MatchColumn>,
    {
        self.request.match_columns = match_columns.into_iter().collect();
        self
    }
    /// The row-filter function to invoke. Set only when policy_type is POLICY_TYPE_ROW_FILTER.
    pub fn with_row_filter(mut self, row_filter: impl Into<Option<FunctionRef>>) -> Self {
        self.request.row_filter = {
            let row_filter: ::core::option::Option<_> = row_filter.into();
            buffa::MessageField::from(row_filter)
        };
        self
    }
    /// The column-mask function to invoke. Set only when policy_type is POLICY_TYPE_COLUMN_MASK.
    pub fn with_column_mask(mut self, column_mask: impl Into<Option<FunctionRef>>) -> Self {
        self.request.column_mask = {
            let column_mask: ::core::option::Option<_> = column_mask.into();
            buffa::MessageField::from(column_mask)
        };
        self
    }
    /// The privileges to deny. Set only when policy_type is POLICY_TYPE_DENY.
    pub fn with_deny(mut self, deny: impl Into<Option<PrivilegeOptions>>) -> Self {
        self.request.deny = {
            let deny: ::core::option::Option<_> = deny.into();
            buffa::MessageField::from(deny)
        };
        self
    }
    /// The privileges to grant. Set only when policy_type is POLICY_TYPE_GRANT.
    pub fn with_grant(mut self, grant: impl Into<Option<PrivilegeOptions>>) -> Self {
        self.request.grant = {
            let grant: ::core::option::Option<_> = grant.into();
            buffa::MessageField::from(grant)
        };
        self
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
    ) -> Self {
        let request = UpdatePolicyRequest {
            on_securable_type: on_securable_type.into(),
            on_securable_fullname: on_securable_fullname.into(),
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// The kind of enforcement this policy applies.
    pub fn with_policy_type(mut self, policy_type: impl Into<Option<PolicyType>>) -> Self {
        self.request.policy_type = policy_type.into().map(buffa::EnumValue::Known);
        self
    }
    /// User-provided free-form text description of the policy.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /// The principals the policy applies to. Empty means "all principals".
    pub fn with_to_principals<I>(mut self, to_principals: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        self.request.to_principals = to_principals.into_iter().collect();
        self
    }
    /// The principals explicitly excluded from the policy, even if matched by to_principals.
    pub fn with_except_principals<I>(mut self, except_principals: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        self.request.except_principals = except_principals.into_iter().collect();
        self
    }
    /// The type of securables the policy takes effect on.
    pub fn with_for_securable_type(
        mut self,
        for_securable_type: impl Into<Option<String>>,
    ) -> Self {
        self.request.for_securable_type = for_securable_type.into();
        self
    }
    /// An opaque condition expression evaluated by the query engine at enforcement time.
    pub fn with_when_condition(mut self, when_condition: impl Into<Option<String>>) -> Self {
        self.request.when_condition = when_condition.into();
        self
    }
    /// The columns referenced by the policy function, and the aliases they are bound to.
    pub fn with_match_columns<I>(mut self, match_columns: I) -> Self
    where
        I: IntoIterator<Item = MatchColumn>,
    {
        self.request.match_columns = match_columns.into_iter().collect();
        self
    }
    /// The row-filter function to invoke. Set only when policy_type is POLICY_TYPE_ROW_FILTER.
    pub fn with_row_filter(mut self, row_filter: impl Into<Option<FunctionRef>>) -> Self {
        self.request.row_filter = {
            let row_filter: ::core::option::Option<_> = row_filter.into();
            buffa::MessageField::from(row_filter)
        };
        self
    }
    /// The column-mask function to invoke. Set only when policy_type is POLICY_TYPE_COLUMN_MASK.
    pub fn with_column_mask(mut self, column_mask: impl Into<Option<FunctionRef>>) -> Self {
        self.request.column_mask = {
            let column_mask: ::core::option::Option<_> = column_mask.into();
            buffa::MessageField::from(column_mask)
        };
        self
    }
    /// The privileges to deny. Set only when policy_type is POLICY_TYPE_DENY.
    pub fn with_deny(mut self, deny: impl Into<Option<PrivilegeOptions>>) -> Self {
        self.request.deny = {
            let deny: ::core::option::Option<_> = deny.into();
            buffa::MessageField::from(deny)
        };
        self
    }
    /// The privileges to grant. Set only when policy_type is POLICY_TYPE_GRANT.
    pub fn with_grant(mut self, grant: impl Into<Option<PrivilegeOptions>>) -> Self {
        self.request.grant = {
            let grant: ::core::option::Option<_> = grant.into();
            buffa::MessageField::from(grant)
        };
        self
    }
    /** The fields to update, as a comma-separated list of field names.

    If empty or "*", every field set in the request is updated. Otherwise only the
    listed fields are updated, and a listed field that is unset in the request is cleared.*/
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
