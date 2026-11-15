pub use agent_skills::AgentSkillHandler;
pub use agents::AgentHandler;
pub use catalogs::CatalogHandler;
pub use credentials::CredentialHandler;
pub use entity_tag_assignments::EntityTagAssignmentHandler;
pub use external_locations::ExternalLocationHandler;
pub use functions::FunctionHandler;
pub use grants::GrantHandler;
pub use model_versions::ModelVersionHandler;
pub use policies::PolicyHandler;
pub use registered_models::RegisteredModelHandler;
pub use schemas::SchemaHandler;
pub use staging_tables::StagingTableHandler;
pub use tables::TableHandler;
pub use tag_policies::TagPolicyHandler;
pub use temporary_credentials::TemporaryCredentialHandler;
pub use volumes::VolumeHandler;

use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};

use crate::policy::{Authorizable, Facts, Operation, Policy, Securable};

// TODO: implement once AssociationLabel::CreatedBy and AssociationLabel::UpdatedBy variants are
// added to unitycatalog_common (they are currently absent from the AssociationLabel enum in
// crates/common/src/models/mod.rs). Once those variants exist, this function should be called
// from create_* and update_* handlers to record who created or last updated a resource.
//
// Proposed signature:
// pub async fn record_audit<S: ResourceStore>(
//     store: &S,
//     resource: &ResourceIdent,
//     principal: &Principal,
//     action: &str, // "created" | "updated"
// ) -> Result<()>

pub mod agent_skills;
pub mod agents;
pub mod catalogs;
pub mod credentials;
pub mod entity_tag_assignments;
pub mod external_locations;
pub mod functions;
pub mod grants;
pub mod model_versions;
pub mod policies;
pub mod registered_models;
pub mod schemas;
pub mod staging_tables;
pub mod tables;
pub mod tag_policies;
pub mod temporary_credentials;
pub mod volumes;

/// Translate a list request's `max_results` into a store page size.
///
/// Unset and `0` both mean "use the server default", which the store reads as `None`.
/// Negative values are rejected.
pub(crate) fn page_size(max_results: Option<i32>) -> crate::Result<Option<usize>> {
    match max_results {
        Some(v) if v < 0 => Err(crate::Error::invalid_argument(
            "max_results must not be negative",
        )),
        Some(v) if v > 0 => Ok(Some(v as usize)),
        _ => Ok(None),
    }
}

/// The fields an update request's `update_mask` selects.
///
/// An empty mask or `*` selects every field set in the request ([`UpdateMask::All`]);
/// otherwise only the listed fields are selected, and a listed field that is unset in the
/// request is cleared. Names outside `updatable` are rejected.
pub(crate) enum UpdateMask<'a> {
    All,
    Fields(Vec<&'a str>),
}

impl<'a> UpdateMask<'a> {
    pub(crate) fn parse(mask: Option<&'a str>, updatable: &[&str]) -> crate::Result<Self> {
        let fields: Vec<&str> = mask
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|f| !f.is_empty())
            .collect();
        if fields.is_empty() || fields == ["*"] {
            return Ok(Self::All);
        }
        if let Some(unknown) = fields.iter().find(|f| !updatable.contains(f)) {
            return Err(crate::Error::invalid_argument(format!(
                "update_mask names a field that cannot be updated: '{unknown}'"
            )));
        }
        Ok(Self::Fields(fields))
    }

    /// Whether `field` is explicitly listed in the mask.
    pub(crate) fn names(&self, field: &str) -> bool {
        matches!(self, Self::Fields(fields) if fields.contains(&field))
    }
}

pub use unitycatalog_handlers::RequestContext;

/// The store ident of the resource a request addresses by name: its
/// [`Authorizable::securable`], which for gets, updates and deletes is the
/// resource itself.
pub trait RequestResource {
    /// The resource the request addresses.
    fn resource(&self) -> ResourceIdent;
}

impl<T: Authorizable + ?Sized> RequestResource for T {
    fn resource(&self) -> ResourceIdent {
        let securable = self.securable();
        match securable.label {
            Some(label) => label.to_ident(ResourceName::new(securable.name.iter())),
            // Only lists act on the root, and they address no single resource.
            None => ResourceIdent::catalog(ResourceRef::Undefined),
        }
    }
}

/// A request whose securable comes from its content (a securable type and a
/// name), so codegen leaves it without an [`Authorizable`] impl: tag
/// assignments, ABAC policies.
pub(crate) trait DynamicSecurable: Send + Sync {
    /// The operation the request performs.
    fn operation(&self) -> Operation;

    /// The securable it acts on, or an error for a type it can't name.
    fn securable(&self) -> crate::Result<Securable>;
}

/// Authorize a [`DynamicSecurable`] request: its operation on its securable,
/// with no facts.
pub(crate) async fn check_dynamic<Cx: Send + Sync + 'static>(
    policy: &(impl Policy<Cx> + ?Sized),
    request: &dyn DynamicSecurable,
    context: &Cx,
) -> crate::Result<()> {
    let securable = request.securable()?;
    policy
        .authorize_checked(
            request.operation().into(),
            &securable,
            &Facts::default(),
            context,
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::page_size;

    #[test]
    fn test_page_size() {
        assert_eq!(page_size(None).unwrap(), None);
        assert_eq!(page_size(Some(0)).unwrap(), None);
        assert_eq!(page_size(Some(25)).unwrap(), Some(25));
        assert!(page_size(Some(-1)).is_err());
    }
}
