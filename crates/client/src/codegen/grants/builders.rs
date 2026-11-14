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
use unitycatalog_common::models::grants::v1::*;

/// Builder for grants
pub struct GetGrantsBuilder {
    client: GrantClient,
    request: GetGrantsRequest,
}

impl GetGrantsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `GrantClient`.
    pub(crate) fn new(
        client: GrantClient,
        securable_type: impl Into<String>,
        full_name: impl Into<String>,
    ) -> Self {
        let request = GetGrantsRequest {
            securable_type: securable_type.into(),
            full_name: full_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }

    /// Only return the privileges of this principal (user or group).
    pub fn with_principal(mut self, principal: impl Into<Option<String>>) -> Self {
        self.request.principal = principal.into();
        self
    }

    /// The maximum number of privilege assignments per page.
    ///
    /// If unset, every assignment is returned. If 0, the server's default page size
    /// is used. Negative values are rejected.
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

impl IntoFuture for GetGrantsBuilder {
    type Output = Result<GetGrantsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_grants(&request).await })
    }
}

/// Builder for grants
pub struct UpdateGrantsBuilder {
    client: GrantClient,
    request: UpdateGrantsRequest,
}

impl UpdateGrantsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `GrantClient`.
    pub(crate) fn new(
        client: GrantClient,
        securable_type: impl Into<String>,
        full_name: impl Into<String>,
        changes: Vec<GrantChange>,
    ) -> Self {
        let request = UpdateGrantsRequest {
            securable_type: securable_type.into(),
            full_name: full_name.into(),
            changes,
            ..Default::default()
        };
        Self { client, request }
    }
}

impl IntoFuture for UpdateGrantsBuilder {
    type Output = Result<UpdateGrantsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_grants(&request).await })
    }
}
