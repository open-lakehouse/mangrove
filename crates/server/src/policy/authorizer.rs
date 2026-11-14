//! A [`Policy`] that asks an [`olai_auth::Authorizer`], such as a Cedar policy
//! decision point, for every decision.
//!
//! Each `(resource, permission)` becomes a Cedar request in the `UnityCatalog`
//! namespace (the [`CEDAR_SCHEMA`] fragment):
//!
//! - the principal is the verified caller, `Lakehouse::User` or
//!   `Lakehouse::ServiceAccount`;
//! - the action is the permission (`UnityCatalog::Action::"external_use"`);
//! - the resource is the securable, keyed by its full name
//!   (`UnityCatalog::Table::"main.sales.orders"`). It goes along as
//!   `resource_entity` with every ancestor (schema, catalog, metastore) as a
//!   parent, so the decision point needs no access to the catalog's tables.
//!
//! It fails closed: a request without a verified caller, a securable that can't be
//! resolved, and any error or timeout of the authorizer are all a deny.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use olai_auth::{AuthContext, AuthorizeRequest, Authorizer, EntityRef, EntityTypes};
use serde_json::json;
use unitycatalog_common::models::{ObjectLabel, Resource, ResourceExt, ResourceIdent, ResourceRef};

use super::{Decision, Permission, Policy};
use crate::Result;
use crate::api::RequestContext;
use crate::store::ResourceStoreReader;

/// The Cedar schema fragment of the server's vocabulary (`namespace
/// UnityCatalog`), for a host to compose with its `Lakehouse` fragment (which
/// declares `User` and `ServiceAccount`) and its own.
pub const CEDAR_SCHEMA: &str = include_str!("../../cedar/UnityCatalog.cedarschema");

/// The Cedar namespace of [`CEDAR_SCHEMA`].
pub const CEDAR_NAMESPACE: &str = "UnityCatalog";

/// The id of the root entity, `UnityCatalog::Metastore::"metastore"`.
pub const METASTORE_ID: &str = "metastore";

/// A [`Policy`] backed by an [`Authorizer`]. See the [module docs](self).
pub struct AuthorizerPolicy {
    authorizer: Arc<dyn Authorizer>,
    store: Arc<dyn ResourceStoreReader>,
    types: EntityTypes,
}

impl AuthorizerPolicy {
    /// Decides with `authorizer`, resolving securables addressed by id through
    /// `store`.
    pub fn new(authorizer: impl Authorizer, store: Arc<dyn ResourceStoreReader>) -> Self {
        Self {
            authorizer: Arc::new(authorizer),
            store,
            types: EntityTypes::new(
                "Lakehouse::User",
                "Lakehouse::ServiceAccount",
                format!("{CEDAR_NAMESPACE}::Action"),
            ),
        }
    }

    /// Decides with the policy decision point at `url`, authenticating with
    /// `token`, and reuses its decisions for `cache_ttl` (none when zero).
    pub fn remote(
        url: &str,
        token: &str,
        timeout: Option<Duration>,
        cache_ttl: Duration,
        store: Arc<dyn ResourceStoreReader>,
    ) -> Self {
        let mut pdp = olai_auth::HttpAuthorizer::new(url, token);
        if let Some(timeout) = timeout {
            pdp = pdp.with_timeout(timeout);
        }
        if cache_ttl.is_zero() {
            Self::new(pdp, store)
        } else {
            Self::new(
                olai_auth::CachedAuthorizer::new(pdp).with_ttl(cache_ttl),
                store,
            )
        }
    }

    /// The request for `permission` on `resource`, or `None` when it is denied
    /// without asking.
    async fn request(
        &self,
        resource: &ResourceIdent,
        permission: &Permission,
        cx: &RequestContext,
    ) -> Option<AuthorizeRequest> {
        let principal = cx.verified_principal()?;
        let parts = match self.full_name(resource).await {
            Ok(parts) => parts,
            Err(e) => {
                tracing::warn!(%resource, error = %e, "denied: the securable can't be resolved");
                return None;
            }
        };
        let Some(securable) = Securable::of(resource.label(), &parts) else {
            tracing::warn!(%resource, "denied: not a securable name");
            return None;
        };
        let context = AuthContext::for_principal(principal).request_time(SystemTime::now());
        Some(
            self.types
                .request(principal, permission.as_ref(), securable.entity.clone())
                .with_context(context)
                .with_resource_entity(securable.to_entity_json()),
        )
    }

    /// The full name of `resource`: as given, looked up for an id, empty for none.
    async fn full_name(&self, resource: &ResourceIdent) -> Result<Vec<String>> {
        Ok(match resource.reference() {
            ResourceRef::Name(name) => name.path().to_vec(),
            ResourceRef::Undefined => Vec::new(),
            ResourceRef::Uuid(_) => {
                let (found, _) = self.store.get(resource).await?;
                match found {
                    // A staging table's own name is not qualified by its schema.
                    Resource::StagingTable(t) => vec![t.catalog_name, t.schema_name, t.name],
                    other => other.resource_name().path().to_vec(),
                }
            }
        })
    }
}

fn decision(allowed: bool) -> Decision {
    if allowed {
        Decision::Allow
    } else {
        Decision::Deny
    }
}

/// Keep the ids of the policies behind an allow on the request's
/// [`DecisionTrail`](unitycatalog_handlers::DecisionTrail), for audit records.
fn record_allowing(context: &RequestContext, response: &olai_auth::AuthorizeResponse) {
    if response.is_allowed() {
        context
            .decision_trail()
            .record(response.policies.iter().cloned());
    }
}

fn log_denial(request: &AuthorizeRequest, errors: &[String]) {
    if !errors.is_empty() {
        tracing::warn!(
            resource = ?request.resource,
            action = %request.action.id,
            ?errors,
            "denied: the authorizer could not decide"
        );
    }
}

#[async_trait]
impl Policy<RequestContext> for AuthorizerPolicy {
    async fn authorize(
        &self,
        resource: &ResourceIdent,
        permission: &Permission,
        context: &RequestContext,
    ) -> Result<Decision> {
        let Some(request) = self.request(resource, permission, context).await else {
            return Ok(Decision::Deny);
        };
        let response = self.authorizer.authorize(&request).await;
        log_denial(&request, &response.errors);
        record_allowing(context, &response);
        Ok(decision(response.is_allowed()))
    }

    async fn authorize_many(
        &self,
        resources: &[ResourceIdent],
        permission: &Permission,
        context: &RequestContext,
    ) -> Result<Vec<Decision>> {
        let mut requests = Vec::with_capacity(resources.len());
        for resource in resources {
            requests.push(self.request(resource, permission, context).await);
        }
        let asked: Vec<AuthorizeRequest> = requests.iter().flatten().cloned().collect();
        let mut responses = self.authorizer.authorize_batch(&asked).await.into_iter();
        Ok(requests
            .iter()
            .map(|request| match request {
                None => Decision::Deny,
                // A short batch answer denies the rest.
                Some(request) => responses.next().map_or(Decision::Deny, |response| {
                    log_denial(request, &response.errors);
                    record_allowing(context, &response);
                    decision(response.is_allowed())
                }),
            })
            .collect())
    }
}

/// A securable as a Cedar entity, with all its ancestors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Securable {
    pub(crate) entity: EntityRef,
    pub(crate) ancestors: Vec<EntityRef>,
}

/// The containers a schema-scoped securable sits in, outermost first.
const IN_SCHEMA: &[&str] = &["Catalog", "Schema"];

impl Securable {
    /// The securable of `label` named `parts`.
    ///
    /// A full name (e.g. three parts for a table) is the securable itself. A
    /// shorter one, as list and create requests use, is its container: two parts
    /// of a table name are the schema, none is the metastore. A longer one, or an
    /// empty part, is not a securable.
    pub(crate) fn of(label: &ObjectLabel, parts: &[String]) -> Option<Self> {
        let (kind, containers): (&str, &[&str]) = match label {
            ObjectLabel::Catalog => ("Catalog", &[]),
            ObjectLabel::Schema => ("Schema", &["Catalog"]),
            ObjectLabel::Table => ("Table", IN_SCHEMA),
            ObjectLabel::Volume => ("Volume", IN_SCHEMA),
            ObjectLabel::Function => ("Function", IN_SCHEMA),
            ObjectLabel::RegisteredModel => ("RegisteredModel", IN_SCHEMA),
            ObjectLabel::Agent => ("Agent", IN_SCHEMA),
            ObjectLabel::AgentSkill => ("AgentSkill", IN_SCHEMA),
            ObjectLabel::StagingTable => ("StagingTable", IN_SCHEMA),
            ObjectLabel::ModelVersion => {
                ("ModelVersion", &["Catalog", "Schema", "RegisteredModel"])
            }
            ObjectLabel::Column => ("Column", &["Catalog", "Schema", "Table"]),
            ObjectLabel::ExternalLocation => ("ExternalLocation", &[]),
            ObjectLabel::Credential => ("Credential", &[]),
            ObjectLabel::Share => ("Share", &[]),
            ObjectLabel::Recipient => ("Recipient", &[]),
            ObjectLabel::Provider => ("Provider", &[]),
            ObjectLabel::TagPolicy => ("TagPolicy", &[]),
            ObjectLabel::PolicyInfo => ("Policy", &[]),
        };
        if parts.len() > containers.len() + 1 || parts.iter().any(String::is_empty) {
            return None;
        }
        let entity_at = |depth: usize| match depth {
            0 => entity("Metastore", METASTORE_ID.to_string()),
            d if d <= containers.len() => entity(containers[d - 1], parts[..d].join(".")),
            d => entity(kind, parts[..d].join(".")),
        };
        Some(Self {
            entity: entity_at(parts.len()),
            ancestors: (0..parts.len()).rev().map(entity_at).collect(),
        })
    }

    /// The Cedar JSON entity: the uid, no attributes, every ancestor a parent.
    pub(crate) fn to_entity_json(&self) -> serde_json::Value {
        json!({ "uid": self.entity, "attrs": {}, "parents": self.ancestors })
    }
}

fn entity(kind: &str, id: String) -> EntityRef {
    EntityRef::new(format!("{CEDAR_NAMESPACE}::{kind}"), id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|p| p.to_string()).collect()
    }

    fn uc(kind: &str, id: &str) -> EntityRef {
        entity(kind, id.to_string())
    }

    #[test]
    fn a_table_has_its_schema_catalog_and_metastore_as_parents() {
        let s = Securable::of(&ObjectLabel::Table, &names(&["main", "sales", "orders"])).unwrap();
        assert_eq!(s.entity, uc("Table", "main.sales.orders"));
        assert_eq!(
            s.ancestors,
            [
                uc("Schema", "main.sales"),
                uc("Catalog", "main"),
                uc("Metastore", "metastore")
            ]
        );
        assert_eq!(
            s.to_entity_json(),
            json!({
                "uid": { "type": "UnityCatalog::Table", "id": "main.sales.orders" },
                "attrs": {},
                "parents": [
                    { "type": "UnityCatalog::Schema", "id": "main.sales" },
                    { "type": "UnityCatalog::Catalog", "id": "main" },
                    { "type": "UnityCatalog::Metastore", "id": "metastore" },
                ],
            })
        );
    }

    #[test]
    fn partial_names_are_the_container() {
        let s = Securable::of(&ObjectLabel::Table, &names(&["main", "sales"])).unwrap();
        assert_eq!(s.entity, uc("Schema", "main.sales"));
        let s = Securable::of(&ObjectLabel::Table, &names(&["main"])).unwrap();
        assert_eq!(s.entity, uc("Catalog", "main"));
        let s = Securable::of(&ObjectLabel::Catalog, &[]).unwrap();
        assert_eq!(s.entity, uc("Metastore", "metastore"));
        assert!(s.ancestors.is_empty());
    }

    #[test]
    fn model_versions_and_columns_nest_one_deeper() {
        let s = Securable::of(
            &ObjectLabel::ModelVersion,
            &names(&["m", "s", "model", "3"]),
        )
        .unwrap();
        assert_eq!(s.entity, uc("ModelVersion", "m.s.model.3"));
        assert_eq!(s.ancestors[0], uc("RegisteredModel", "m.s.model"));
        assert_eq!(s.ancestors.len(), 4);
        let s = Securable::of(&ObjectLabel::Column, &names(&["m", "s", "t", "c"])).unwrap();
        assert_eq!(s.ancestors[0], uc("Table", "m.s.t"));
    }

    #[test]
    fn metastore_securables_sit_under_the_metastore() {
        let s = Securable::of(&ObjectLabel::ExternalLocation, &names(&["landing"])).unwrap();
        assert_eq!(s.entity, uc("ExternalLocation", "landing"));
        assert_eq!(s.ancestors, [uc("Metastore", "metastore")]);
        let s = Securable::of(&ObjectLabel::PolicyInfo, &names(&["mask"])).unwrap();
        assert_eq!(s.entity, uc("Policy", "mask"));
    }

    #[test]
    fn malformed_names_are_not_securables() {
        assert!(Securable::of(&ObjectLabel::Catalog, &names(&["a", "b"])).is_none());
        assert!(Securable::of(&ObjectLabel::Table, &names(&["a", "", "c"])).is_none());
        assert!(Securable::of(&ObjectLabel::Credential, &names(&["a", "b"])).is_none());
    }
}

#[cfg(all(test, feature = "memory"))]
#[path = "authorizer_tests.rs"]
mod handler_tests;
