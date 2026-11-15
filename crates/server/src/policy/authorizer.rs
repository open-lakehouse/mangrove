//! A [`Policy`] that asks an [`olai_auth::Authorizer`], such as a Cedar policy
//! decision point, for every decision.
//!
//! Each `(action, securable, facts)` becomes a Cedar request in the
//! `UnityCatalog` namespace (the [`CEDAR_SCHEMA_FRAGMENTS`](super::CEDAR_SCHEMA_FRAGMENTS)):
//!
//! - the principal is the verified caller, `Lakehouse::User` or
//!   `Lakehouse::ServiceAccount`;
//! - the action is the operation (`UnityCatalog::Action::"GetTable"`);
//! - the resource is the securable, keyed by its full name
//!   (`UnityCatalog::Table::"main.sales.orders"`). It and every ancestor
//!   (schema, catalog, metastore) go along as entities, each with its ancestors
//!   as parents and as typed attributes (`resource.schema`, `resource.catalog`),
//!   so a policy can ask about any of them. The decision point's entity loader
//!   adds what the caller holds on each (`privileges`, `inherited_privileges`);
//! - the context is how the caller authenticated, when, and the request's
//!   [`Facts`].
//!
//! It fails closed: a request without a verified caller, a securable that isn't a
//! full name, and any error or timeout of the authorizer are all a deny.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use olai_auth::{AuthContext, AuthorizeRequest, Authorizer, EntityRef, EntityTypes};
use serde_json::{Map, Value, json};
use unitycatalog_common::models::ObjectLabel;

use super::{
    CEDAR_NAMESPACE, Decision, Facts, METASTORE_ID, Policy, Securable, ServerAction, entity_kind,
};
use crate::Result;
use crate::api::RequestContext;

/// A [`Policy`] backed by an [`Authorizer`]. See the [module docs](self).
pub struct AuthorizerPolicy {
    authorizer: Arc<dyn Authorizer>,
    types: EntityTypes,
}

impl AuthorizerPolicy {
    /// Decides with `authorizer`.
    pub fn new(authorizer: impl Authorizer) -> Self {
        Self {
            authorizer: Arc::new(authorizer),
            types: EntityTypes::new(
                "Lakehouse::User",
                "Lakehouse::ServiceAccount",
                format!("{CEDAR_NAMESPACE}::Action"),
            ),
        }
    }

    /// Decides with the policy decision point at `url`, authenticating with
    /// `token`, and reuses its decisions for `cache_ttl` (none when zero).
    pub fn remote(url: &str, token: &str, timeout: Option<Duration>, cache_ttl: Duration) -> Self {
        let mut pdp = olai_auth::HttpAuthorizer::new(url, token);
        if let Some(timeout) = timeout {
            pdp = pdp.with_timeout(timeout);
        }
        if cache_ttl.is_zero() {
            Self::new(pdp)
        } else {
            Self::new(olai_auth::CachedAuthorizer::new(pdp).with_ttl(cache_ttl))
        }
    }

    /// The request for `action` on `securable`, or `None` when it is denied
    /// without asking.
    fn request(
        &self,
        action: ServerAction,
        securable: &Securable,
        facts: &Facts,
        cx: &RequestContext,
    ) -> Option<AuthorizeRequest> {
        let principal = cx.verified_principal()?;
        let Some(entities) = Entities::of(securable.label.as_ref(), &securable.name) else {
            tracing::warn!(%action, name = ?securable.name, "denied: not a securable name");
            return None;
        };
        let context = with_facts(
            AuthContext::for_principal(principal).request_time(SystemTime::now()),
            facts,
        );
        Some(
            self.types
                .request(principal, action.as_str(), entities.resource().clone())
                .with_context(context.into_map())
                .with_entities(entities.to_json()),
        )
    }
}

/// `context` with the request's facts as its optional keys.
fn with_facts(mut context: AuthContext, facts: &Facts) -> AuthContext {
    if let Some(operation) = facts.credential_operation {
        context = context.with("credential_operation", operation.as_str());
    }
    if let Some(url) = &facts.url {
        context = context.with("url", url.clone());
    }
    if let Some(fields) = &facts.update_fields {
        context = context.with("update_fields", fields.clone());
    }
    if let Some(is_creator) = facts.is_creator {
        context = context.with("is_creator", is_creator);
    }
    context
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
        action: ServerAction,
        securable: &Securable,
        facts: &Facts,
        context: &RequestContext,
    ) -> Result<Decision> {
        let Some(request) = self.request(action, securable, facts, context) else {
            return Ok(Decision::Deny);
        };
        let response = self.authorizer.authorize(&request).await;
        log_denial(&request, &response.errors);
        record_allowing(context, &response);
        Ok(decision(response.is_allowed()))
    }

    async fn authorize_many(
        &self,
        action: ServerAction,
        securables: &[Securable],
        facts: &Facts,
        context: &RequestContext,
    ) -> Result<Vec<Decision>> {
        let requests: Vec<Option<AuthorizeRequest>> = securables
            .iter()
            .map(|securable| self.request(action, securable, facts, context))
            .collect();
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

/// A securable and every ancestor as Cedar entities, outermost (the metastore)
/// first, so `chain[d]` is the entity at depth `d`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entities {
    chain: Vec<EntityRef>,
}

impl Entities {
    /// The entities of the securable of `label` named `parts`.
    ///
    /// A full name (e.g. three parts for a table) is the securable itself. A
    /// shorter one, as list requests use, is its container: two parts of a table
    /// name are the schema, none is the metastore. A longer one, or an empty
    /// part, is not a securable. The service root (`None`) is the metastore.
    pub(crate) fn of(label: Option<&ObjectLabel>, parts: &[String]) -> Option<Self> {
        let Some(label) = label else {
            return parts.is_empty().then(|| Self {
                chain: vec![entity("Metastore", METASTORE_ID.to_string())],
            });
        };
        let (kind, containers) = entity_kind(*label);
        if parts.len() > containers.len() + 1 || parts.iter().any(String::is_empty) {
            return None;
        }
        let entity_at = |depth: usize| match depth {
            0 => entity("Metastore", METASTORE_ID.to_string()),
            d if d <= containers.len() => entity(containers[d - 1], parts[..d].join(".")),
            d => entity(kind, parts[..d].join(".")),
        };
        Some(Self {
            chain: (0..=parts.len()).map(entity_at).collect(),
        })
    }

    /// The securable itself.
    pub(crate) fn resource(&self) -> &EntityRef {
        self.chain
            .last()
            .expect("the chain has the metastore at least")
    }

    /// Its ancestors, innermost first.
    #[cfg(test)]
    pub(crate) fn ancestors(&self) -> Vec<EntityRef> {
        self.chain[..self.chain.len() - 1]
            .iter()
            .rev()
            .cloned()
            .collect()
    }

    /// Every entity as Cedar JSON: its ancestors as parents (innermost first)
    /// and the typed attributes of its kind (`catalog`, `schema`,
    /// `registered_model`, `table`). The decision point's loader adds the
    /// privilege attributes.
    pub(crate) fn to_json(&self) -> Vec<Value> {
        (0..self.chain.len())
            .map(|depth| {
                let uid = &self.chain[depth];
                let parents: Vec<&EntityRef> = self.chain[..depth].iter().rev().collect();
                json!({ "uid": uid, "attrs": self.attrs(depth), "parents": parents })
            })
            .collect()
    }

    /// The typed ancestor attributes of the entity at `depth`, per the schema.
    fn attrs(&self, depth: usize) -> Map<String, Value> {
        let kind = self.chain[depth]
            .entity_type
            .strip_prefix(&format!("{CEDAR_NAMESPACE}::"))
            .unwrap_or_default();
        let names: &[&str] = match kind {
            "Schema" => &["catalog"],
            "Table" | "Volume" | "Function" | "RegisteredModel" | "Agent" | "AgentSkill"
            | "StagingTable" => &["catalog", "schema"],
            "ModelVersion" => &["catalog", "schema", "registered_model"],
            "Column" => &["catalog", "schema", "table"],
            _ => &[],
        };
        // The ancestors below the metastore are at depths 1, 2, 3, in that order.
        names
            .iter()
            .enumerate()
            .filter_map(|(i, name)| {
                let ancestor = self.chain.get(i + 1).filter(|_| i + 1 < depth)?;
                Some((name.to_string(), json!({ "__entity": ancestor })))
            })
            .collect()
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
    fn a_table_has_its_schema_catalog_and_metastore_as_ancestors() {
        let e = Entities::of(
            Some(&ObjectLabel::Table),
            &names(&["main", "sales", "orders"]),
        )
        .unwrap();
        assert_eq!(e.resource(), &uc("Table", "main.sales.orders"));
        assert_eq!(
            e.ancestors(),
            [
                uc("Schema", "main.sales"),
                uc("Catalog", "main"),
                uc("Metastore", "metastore")
            ]
        );
        let json = e.to_json();
        assert_eq!(json.len(), 4);
        assert_eq!(
            json[3],
            json!({
                "uid": { "type": "UnityCatalog::Table", "id": "main.sales.orders" },
                "attrs": {
                    "catalog": { "__entity": { "type": "UnityCatalog::Catalog", "id": "main" } },
                    "schema": { "__entity": { "type": "UnityCatalog::Schema", "id": "main.sales" } },
                },
                "parents": [
                    { "type": "UnityCatalog::Schema", "id": "main.sales" },
                    { "type": "UnityCatalog::Catalog", "id": "main" },
                    { "type": "UnityCatalog::Metastore", "id": "metastore" },
                ],
            })
        );
        // Every ancestor is an entity of its own, with its own ancestors.
        assert_eq!(
            json[2],
            json!({
                "uid": { "type": "UnityCatalog::Schema", "id": "main.sales" },
                "attrs": {
                    "catalog": { "__entity": { "type": "UnityCatalog::Catalog", "id": "main" } },
                },
                "parents": [
                    { "type": "UnityCatalog::Catalog", "id": "main" },
                    { "type": "UnityCatalog::Metastore", "id": "metastore" },
                ],
            })
        );
        assert_eq!(
            json[0],
            json!({
                "uid": { "type": "UnityCatalog::Metastore", "id": "metastore" },
                "attrs": {},
                "parents": [],
            })
        );
    }

    #[test]
    fn partial_names_are_the_container() {
        let e = Entities::of(Some(&ObjectLabel::Table), &names(&["main", "sales"])).unwrap();
        assert_eq!(e.resource(), &uc("Schema", "main.sales"));
        let e = Entities::of(Some(&ObjectLabel::Table), &names(&["main"])).unwrap();
        assert_eq!(e.resource(), &uc("Catalog", "main"));
        let e = Entities::of(Some(&ObjectLabel::Catalog), &[]).unwrap();
        assert_eq!(e.resource(), &uc("Metastore", "metastore"));
        assert!(e.ancestors().is_empty());
        let e = Entities::of(None, &[]).unwrap();
        assert_eq!(e.resource(), &uc("Metastore", "metastore"));
    }

    #[test]
    fn model_versions_and_columns_nest_one_deeper() {
        let e = Entities::of(
            Some(&ObjectLabel::ModelVersion),
            &names(&["m", "s", "model", "3"]),
        )
        .unwrap();
        assert_eq!(e.resource(), &uc("ModelVersion", "m.s.model.3"));
        assert_eq!(e.ancestors()[0], uc("RegisteredModel", "m.s.model"));
        assert_eq!(e.ancestors().len(), 4);
        let attrs = &e.to_json()[4]["attrs"];
        assert_eq!(attrs["registered_model"]["__entity"]["id"], "m.s.model");
        let e = Entities::of(Some(&ObjectLabel::Column), &names(&["m", "s", "t", "c"])).unwrap();
        assert_eq!(e.ancestors()[0], uc("Table", "m.s.t"));
        assert_eq!(e.to_json()[4]["attrs"]["table"]["__entity"]["id"], "m.s.t");
    }

    #[test]
    fn metastore_securables_sit_under_the_metastore() {
        let e = Entities::of(Some(&ObjectLabel::ExternalLocation), &names(&["landing"])).unwrap();
        assert_eq!(e.resource(), &uc("ExternalLocation", "landing"));
        assert_eq!(e.ancestors(), [uc("Metastore", "metastore")]);
        let e = Entities::of(Some(&ObjectLabel::PolicyInfo), &names(&["mask"])).unwrap();
        assert_eq!(e.resource(), &uc("PolicyInfo", "mask"));
    }

    #[test]
    fn malformed_names_are_not_securables() {
        assert!(Entities::of(Some(&ObjectLabel::Catalog), &names(&["a", "b"])).is_none());
        assert!(Entities::of(Some(&ObjectLabel::Table), &names(&["a", "", "c"])).is_none());
        assert!(Entities::of(Some(&ObjectLabel::Credential), &names(&["a", "b"])).is_none());
        assert!(Entities::of(None, &names(&["a"])).is_none());
    }

    #[test]
    fn facts_become_context_keys() {
        let facts = Facts::credential(super::super::CredentialOperation::ReadWrite)
            .with_url("s3://b/k")
            .with_update_fields(["owner"]);
        let facts = Facts {
            is_creator: Some(true),
            ..facts
        };
        let map = with_facts(AuthContext::new(), &facts).into_map();
        assert_eq!(map["credential_operation"], "READ_WRITE");
        assert_eq!(map["url"], "s3://b/k");
        assert_eq!(map["update_fields"], json!(["owner"]));
        assert_eq!(map["is_creator"], true);
        assert!(
            with_facts(AuthContext::new(), &Facts::default())
                .into_map()
                .is_empty()
        );
    }
}

#[cfg(all(test, feature = "memory"))]
#[path = "authorizer_tests.rs"]
mod handler_tests;
