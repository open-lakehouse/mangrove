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
use unitycatalog_common::models::tags::v1::*;
/// Builder for entity tag assignments
pub struct ListEntityTagAssignmentsBuilder {
    client: EntityTagAssignmentClient,
    request: ListEntityTagAssignmentsRequest,
}
impl ListEntityTagAssignmentsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `EntityTagAssignmentClient`.
    pub(crate) fn new(
        client: EntityTagAssignmentClient,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
    ) -> Self {
        let request = ListEntityTagAssignmentsRequest {
            entity_type: entity_type.into(),
            entity_name: entity_name.into(),
            ..Default::default()
        };
        Self { client, request }
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
impl IntoFuture for ListEntityTagAssignmentsBuilder {
    type Output = Result<ListEntityTagAssignmentsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_entity_tag_assignments(&request).await })
    }
}
/// Builder for entity tag assignment
pub struct CreateEntityTagAssignmentBuilder {
    client: EntityTagAssignmentClient,
    request: CreateEntityTagAssignmentRequest,
}
impl CreateEntityTagAssignmentBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `EntityTagAssignmentClient`.
    pub(crate) fn new(
        client: EntityTagAssignmentClient,
        tag_assignment: EntityTagAssignment,
    ) -> Self {
        let request = CreateEntityTagAssignmentRequest {
            tag_assignment: buffa::MessageField::some(tag_assignment),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for CreateEntityTagAssignmentBuilder {
    type Output = Result<EntityTagAssignment>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_entity_tag_assignment(&request).await })
    }
}
/// Builder for entity tag assignment
pub struct GetEntityTagAssignmentBuilder {
    client: EntityTagAssignmentClient,
    request: GetEntityTagAssignmentRequest,
}
impl GetEntityTagAssignmentBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `EntityTagAssignmentClient`.
    pub(crate) fn new(
        client: EntityTagAssignmentClient,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
        tag_key: impl Into<String>,
    ) -> Self {
        let request = GetEntityTagAssignmentRequest {
            entity_type: entity_type.into(),
            entity_name: entity_name.into(),
            tag_key: tag_key.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GetEntityTagAssignmentBuilder {
    type Output = Result<EntityTagAssignment>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_entity_tag_assignment(&request).await })
    }
}
/// Builder for entity tag assignment
pub struct UpdateEntityTagAssignmentBuilder {
    client: EntityTagAssignmentClient,
    request: UpdateEntityTagAssignmentRequest,
}
impl UpdateEntityTagAssignmentBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `EntityTagAssignmentClient`.
    pub(crate) fn new(
        client: EntityTagAssignmentClient,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
        tag_key: impl Into<String>,
        tag_assignment: EntityTagAssignment,
    ) -> Self {
        let request = UpdateEntityTagAssignmentRequest {
            entity_type: entity_type.into(),
            entity_name: entity_name.into(),
            tag_key: tag_key.into(),
            tag_assignment: buffa::MessageField::some(tag_assignment),
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
impl IntoFuture for UpdateEntityTagAssignmentBuilder {
    type Output = Result<EntityTagAssignment>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_entity_tag_assignment(&request).await })
    }
}
/// Builder for entity tag assignment
pub struct DeleteEntityTagAssignmentBuilder {
    client: EntityTagAssignmentClient,
    request: DeleteEntityTagAssignmentRequest,
}
impl DeleteEntityTagAssignmentBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `EntityTagAssignmentClient`.
    pub(crate) fn new(
        client: EntityTagAssignmentClient,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
        tag_key: impl Into<String>,
    ) -> Self {
        let request = DeleteEntityTagAssignmentRequest {
            entity_type: entity_type.into(),
            entity_name: entity_name.into(),
            tag_key: tag_key.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for DeleteEntityTagAssignmentBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_entity_tag_assignment(&request).await })
    }
}
