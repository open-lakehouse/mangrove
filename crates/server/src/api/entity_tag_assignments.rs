use unitycatalog_common::models::tags::v1::*;
use unitycatalog_common::models::{
    AssociationLabel, ObjectLabel, PropertyMap, ResourceIdent, ResourceName, ResourceRef,
};

use super::{RequestContext, SecuredAction};
use crate::Result;
pub use crate::codegen::entity_tag_assignments::EntityTagAssignmentHandler;
use crate::policy::{Permission, Policy};
use crate::store::ResourceStore;

/// Property-map key under which a tag assignment's value is stored on the association edge.
const TAG_VALUE_KEY: &str = "tag_value";

/// Map a Databricks `entity_type` string to the corresponding [`ObjectLabel`].
fn entity_label(entity_type: &str) -> Result<ObjectLabel> {
    Ok(match entity_type {
        "catalogs" => ObjectLabel::Catalog,
        "schemas" => ObjectLabel::Schema,
        "tables" => ObjectLabel::Table,
        "columns" => ObjectLabel::Column,
        "volumes" => ObjectLabel::Volume,
        other => {
            return Err(crate::Error::invalid_argument(format!(
                "unsupported entity_type '{other}': expected one of catalogs, schemas, tables, columns, volumes"
            )));
        }
    })
}

/// Build the [`ResourceIdent`] of the entity being tagged from its type and fully qualified name.
fn entity_ident(entity_type: &str, entity_name: &str) -> Result<ResourceIdent> {
    let label = entity_label(entity_type)?;
    Ok(label.to_ident(ResourceName::from_naive_str_split(entity_name)))
}

/// Build the [`ResourceIdent`] of the TagPolicy referenced by a tag key.
fn tag_ident(tag_key: &str) -> ResourceIdent {
    ResourceIdent::tag_policy(ResourceName::new([tag_key]))
}

/// Read the tag value out of an association's property map.
fn tag_value_from_props(props: Option<PropertyMap>) -> Option<String> {
    props
        .and_then(|mut p| p.remove(TAG_VALUE_KEY))
        .and_then(|v| match v {
            serde_json::Value::String(s) => Some(s),
            _ => None,
        })
}

/// Build the property map stored on the association edge for a tag value.
fn props_for_value(tag_value: &Option<String>) -> Option<PropertyMap> {
    tag_value.as_ref().map(|value| {
        let mut map = PropertyMap::new();
        map.insert(
            TAG_VALUE_KEY.to_string(),
            serde_json::Value::String(value.clone()),
        );
        map
    })
}

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext>> EntityTagAssignmentHandler<RequestContext> for T {
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_entity_tag_assignment(
        &self,
        request: CreateEntityTagAssignmentRequest,
        context: RequestContext,
    ) -> Result<EntityTagAssignment> {
        self.check_required(&request, &context).await?;
        let assignment = request
            .tag_assignment
            .ok_or_else(|| crate::Error::invalid_argument("tag_assignment must be provided"))?;
        tracing::Span::current().record("resource_name", &assignment.entity_name);

        let entity = entity_ident(&assignment.entity_type, &assignment.entity_name)?;
        let tag = tag_ident(&assignment.tag_key);
        self.add_association(
            &entity,
            &tag,
            &AssociationLabel::Tagged,
            props_for_value(&assignment.tag_value),
        )
        .await?;
        Ok(assignment)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_entity_tag_assignment(
        &self,
        request: DeleteEntityTagAssignmentRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.entity_name);
        self.check_required(&request, &context).await?;
        let entity = entity_ident(&request.entity_type, &request.entity_name)?;
        let tag = tag_ident(&request.tag_key);
        self.remove_association(&entity, &tag, &AssociationLabel::Tagged)
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_entity_tag_assignment(
        &self,
        request: GetEntityTagAssignmentRequest,
        context: RequestContext,
    ) -> Result<EntityTagAssignment> {
        tracing::Span::current().record("resource_name", &request.entity_name);
        self.check_required(&request, &context).await?;
        let entity = entity_ident(&request.entity_type, &request.entity_name)?;
        let tag = tag_ident(&request.tag_key);
        let (edges, _) = self
            .list_associations_with_properties(
                &entity,
                &AssociationLabel::Tagged,
                Some(&tag),
                None,
                None,
            )
            .await?;
        let (_, props) = edges.into_iter().next().ok_or(crate::Error::NotFound)?;
        Ok(EntityTagAssignment {
            entity_type: request.entity_type,
            entity_name: request.entity_name,
            tag_key: request.tag_key,
            tag_value: tag_value_from_props(props),
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn list_entity_tag_assignments(
        &self,
        request: ListEntityTagAssignmentsRequest,
        context: RequestContext,
    ) -> Result<ListEntityTagAssignmentsResponse> {
        tracing::Span::current().record("resource_name", &request.entity_name);
        self.check_required(&request, &context).await?;
        let entity = entity_ident(&request.entity_type, &request.entity_name)?;
        let (edges, next_page_token) = self
            .list_associations_with_properties(
                &entity,
                &AssociationLabel::Tagged,
                None,
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        let mut tag_assignments = Vec::with_capacity(edges.len());
        for (tag_ref, props) in edges {
            // The association target is the TagPolicy; resolve it to recover its tag key.
            // The edge may reference the target by UUID (e.g. the Postgres backend), so fetch
            // the resource rather than reading the key off the ident.
            let tag_key = tag_key_for(self, &tag_ref).await?;
            tag_assignments.push(EntityTagAssignment {
                entity_type: request.entity_type.clone(),
                entity_name: request.entity_name.clone(),
                tag_key,
                tag_value: tag_value_from_props(props),
            });
        }
        Ok(ListEntityTagAssignmentsResponse {
            tag_assignments,
            next_page_token,
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_entity_tag_assignment(
        &self,
        request: UpdateEntityTagAssignmentRequest,
        context: RequestContext,
    ) -> Result<EntityTagAssignment> {
        tracing::Span::current().record("resource_name", &request.entity_name);
        self.check_required(&request, &context).await?;
        let assignment = request
            .tag_assignment
            .ok_or_else(|| crate::Error::invalid_argument("tag_assignment must be provided"))?;
        let entity = entity_ident(&request.entity_type, &request.entity_name)?;
        let tag = tag_ident(&request.tag_key);
        // The edge store's `add` rejects an existing edge rather than upserting, so
        // updating the tag value is remove-then-add (both carry the inverse edge).
        // A missing edge on remove is fine — treat update as an upsert.
        match self
            .remove_association(&entity, &tag, &AssociationLabel::Tagged)
            .await
        {
            Ok(()) | Err(unitycatalog_common::Error::NotFound) => {}
            Err(e) => return Err(e.into()),
        }
        self.add_association(
            &entity,
            &tag,
            &AssociationLabel::Tagged,
            props_for_value(&assignment.tag_value),
        )
        .await?;
        Ok(EntityTagAssignment {
            entity_type: request.entity_type,
            entity_name: request.entity_name,
            tag_key: request.tag_key,
            tag_value: assignment.tag_value,
        })
    }
}

/// Resolve the tag key for an association target.
///
/// The target may be referenced by name or (e.g. on the Postgres backend) by UUID, so this
/// looks the TagPolicy up in the store and returns its `tag_key`.
async fn tag_key_for<S: ResourceStore>(store: &S, tag_ref: &ResourceIdent) -> Result<String> {
    // Fast path: the ident already carries the single-segment name.
    if let ResourceRef::Name(name) = tag_ref.as_ref()
        && let Some(last) = name.iter().last()
    {
        return Ok(last.clone());
    }
    let (resource, _) = store.get(tag_ref).await?;
    let policy: TagPolicy = resource.try_into()?;
    Ok(policy.tag_key)
}

// Authorization is checked against the *entity* being tagged: tagging is a modification of the
// entity's metadata, so the caller needs the corresponding permission on the entity.
impl SecuredAction for CreateEntityTagAssignmentRequest {
    fn resource(&self) -> ResourceIdent {
        match self.tag_assignment.as_ref() {
            Some(a) => entity_ident(&a.entity_type, &a.entity_name)
                .unwrap_or_else(|_| ResourceIdent::catalog(ResourceRef::Undefined)),
            None => ResourceIdent::catalog(ResourceRef::Undefined),
        }
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for ListEntityTagAssignmentsRequest {
    fn resource(&self) -> ResourceIdent {
        entity_ident(&self.entity_type, &self.entity_name)
            .unwrap_or_else(|_| ResourceIdent::catalog(ResourceRef::Undefined))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetEntityTagAssignmentRequest {
    fn resource(&self) -> ResourceIdent {
        entity_ident(&self.entity_type, &self.entity_name)
            .unwrap_or_else(|_| ResourceIdent::catalog(ResourceRef::Undefined))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateEntityTagAssignmentRequest {
    fn resource(&self) -> ResourceIdent {
        entity_ident(&self.entity_type, &self.entity_name)
            .unwrap_or_else(|_| ResourceIdent::catalog(ResourceRef::Undefined))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteEntityTagAssignmentRequest {
    fn resource(&self) -> ResourceIdent {
        entity_ident(&self.entity_type, &self.entity_name)
            .unwrap_or_else(|_| ResourceIdent::catalog(ResourceRef::Undefined))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use unitycatalog_common::models::catalogs::v1::Catalog;
    use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

    use super::*;
    use crate::memory::InMemoryResourceStore;
    use crate::policy::ConstantPolicy;
    use crate::services::ServerHandler;

    // Seed a catalog (the tagged entity) and two TagPolicy objects, since add_association
    // requires both endpoints of the edge to exist as stored objects.
    async fn handler() -> ServerHandler<RequestContext> {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        store
            .create(
                Catalog {
                    name: "cat".to_string(),
                    ..Default::default()
                }
                .into(),
            )
            .await
            .unwrap();
        for key in ["pii", "team"] {
            store
                .create(
                    TagPolicy {
                        tag_key: key.to_string(),
                        ..Default::default()
                    }
                    .into(),
                )
                .await
                .unwrap();
        }
        let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
        ServerHandler::try_new_tokio(policy, store).unwrap()
    }

    fn ctx() -> RequestContext {
        RequestContext {
            recipient: crate::policy::Principal::anonymous(),
        }
    }

    fn assignment(tag_key: &str, tag_value: Option<&str>) -> EntityTagAssignment {
        EntityTagAssignment {
            entity_type: "catalogs".to_string(),
            entity_name: "cat".to_string(),
            tag_key: tag_key.to_string(),
            tag_value: tag_value.map(str::to_string),
        }
    }

    #[tokio::test]
    async fn assignment_crud_and_multiple_tags() {
        let h = handler().await;

        // assign two distinct tags to the same entity (exercises the multimap)
        h.create_entity_tag_assignment(
            CreateEntityTagAssignmentRequest {
                tag_assignment: Some(assignment("pii", Some("true"))),
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_entity_tag_assignment(
            CreateEntityTagAssignmentRequest {
                tag_assignment: Some(assignment("team", Some("data-eng"))),
            },
            ctx(),
        )
        .await
        .unwrap();

        // list returns BOTH tags for the entity
        let listed = h
            .list_entity_tag_assignments(
                ListEntityTagAssignmentsRequest {
                    entity_type: "catalogs".to_string(),
                    entity_name: "cat".to_string(),
                    max_results: None,
                    page_token: None,
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(listed.tag_assignments.len(), 2, "entity should hold 2 tags");
        let mut by_key: Vec<_> = listed
            .tag_assignments
            .iter()
            .map(|a| (a.tag_key.as_str(), a.tag_value.as_deref()))
            .collect();
        by_key.sort();
        assert_eq!(
            by_key,
            vec![("pii", Some("true")), ("team", Some("data-eng"))]
        );

        // get a single assignment with its value
        let got = h
            .get_entity_tag_assignment(
                GetEntityTagAssignmentRequest {
                    entity_type: "catalogs".to_string(),
                    entity_name: "cat".to_string(),
                    tag_key: "pii".to_string(),
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(got.tag_value.as_deref(), Some("true"));

        // update the value
        let updated = h
            .update_entity_tag_assignment(
                UpdateEntityTagAssignmentRequest {
                    entity_type: "catalogs".to_string(),
                    entity_name: "cat".to_string(),
                    tag_key: "pii".to_string(),
                    tag_assignment: Some(assignment("pii", Some("false"))),
                    update_mask: None,
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(updated.tag_value.as_deref(), Some("false"));
        // and confirm the update persisted via get
        let re_got = h
            .get_entity_tag_assignment(
                GetEntityTagAssignmentRequest {
                    entity_type: "catalogs".to_string(),
                    entity_name: "cat".to_string(),
                    tag_key: "pii".to_string(),
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(re_got.tag_value.as_deref(), Some("false"));

        // delete one tag; the other remains
        h.delete_entity_tag_assignment(
            DeleteEntityTagAssignmentRequest {
                entity_type: "catalogs".to_string(),
                entity_name: "cat".to_string(),
                tag_key: "pii".to_string(),
            },
            ctx(),
        )
        .await
        .unwrap();
        let after = h
            .list_entity_tag_assignments(
                ListEntityTagAssignmentsRequest {
                    entity_type: "catalogs".to_string(),
                    entity_name: "cat".to_string(),
                    max_results: None,
                    page_token: None,
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(after.tag_assignments.len(), 1);
        assert_eq!(after.tag_assignments[0].tag_key, "team");

        // get the deleted tag → NotFound
        let missing = h
            .get_entity_tag_assignment(
                GetEntityTagAssignmentRequest {
                    entity_type: "catalogs".to_string(),
                    entity_name: "cat".to_string(),
                    tag_key: "pii".to_string(),
                },
                ctx(),
            )
            .await;
        assert!(missing.is_err(), "deleted assignment should be NotFound");
    }

    #[tokio::test]
    async fn create_rejects_unknown_entity_type() {
        let h = handler().await;
        let mut bad = assignment("pii", Some("x"));
        bad.entity_type = "notebooks".to_string();
        let result = h
            .create_entity_tag_assignment(
                CreateEntityTagAssignmentRequest {
                    tag_assignment: Some(bad),
                },
                ctx(),
            )
            .await;
        assert!(result.is_err(), "unsupported entity_type must be rejected");
    }
}
