// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::client::EntityTagAssignmentClient;
use unitycatalog_common::models::tags::v1::*;
#[::async_trait::async_trait]
impl<Cx: Send + 'static>
    unitycatalog_handlers::entity_tag_assignments::EntityTagAssignmentHandler<Cx>
    for EntityTagAssignmentClient
{
    async fn list_entity_tag_assignments(
        &self,
        request: ListEntityTagAssignmentsRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListEntityTagAssignmentsResponse> {
        EntityTagAssignmentClient::list_entity_tag_assignments(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn create_entity_tag_assignment(
        &self,
        request: CreateEntityTagAssignmentRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<EntityTagAssignment> {
        EntityTagAssignmentClient::create_entity_tag_assignment(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn get_entity_tag_assignment(
        &self,
        request: GetEntityTagAssignmentRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<EntityTagAssignment> {
        EntityTagAssignmentClient::get_entity_tag_assignment(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn update_entity_tag_assignment(
        &self,
        request: UpdateEntityTagAssignmentRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<EntityTagAssignment> {
        EntityTagAssignmentClient::update_entity_tag_assignment(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn delete_entity_tag_assignment(
        &self,
        request: DeleteEntityTagAssignmentRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        EntityTagAssignmentClient::delete_entity_tag_assignment(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
