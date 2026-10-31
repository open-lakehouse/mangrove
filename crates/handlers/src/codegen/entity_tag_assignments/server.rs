// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::EntityTagAssignmentHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::tags::v1::*;
pub async fn list_entity_tag_assignments<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListEntityTagAssignmentsRequest,
) -> Result<::axum::Json<ListEntityTagAssignmentsResponse>>
where
    T: EntityTagAssignmentHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler
        .list_entity_tag_assignments(request, context)
        .await?;
    Ok(axum::Json(result))
}
pub async fn create_entity_tag_assignment<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateEntityTagAssignmentRequest,
) -> Result<::axum::Json<EntityTagAssignment>>
where
    T: EntityTagAssignmentHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler
        .create_entity_tag_assignment(request, context)
        .await?;
    Ok(axum::Json(result))
}
pub async fn get_entity_tag_assignment<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetEntityTagAssignmentRequest,
) -> Result<::axum::Json<EntityTagAssignment>>
where
    T: EntityTagAssignmentHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_entity_tag_assignment(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_entity_tag_assignment<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateEntityTagAssignmentRequest,
) -> Result<::axum::Json<EntityTagAssignment>>
where
    T: EntityTagAssignmentHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler
        .update_entity_tag_assignment(request, context)
        .await?;
    Ok(axum::Json(result))
}
pub async fn delete_entity_tag_assignment<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteEntityTagAssignmentRequest,
) -> Result<()>
where
    T: EntityTagAssignmentHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler
        .delete_entity_tag_assignment(request, context)
        .await?;
    Ok(())
}
