//! Authorization for the Unity Catalog server.
//!
//! Every request is gated by a [`Policy`]: given the [`Operation`] it performs
//! (one per API operation, generated from the protos), the [`Securable`] it acts
//! on, the [`Facts`] of the request a decision may need, and a caller-supplied
//! context, it returns a [`Decision`] of [`Allow`](Decision::Allow) or
//! [`Deny`](Decision::Deny). What an operation requires (which privileges, on
//! which securable or ancestor, or ownership) is the policy's business: the
//! server only names the operation and gathers the facts.
//!
//! Request types implement the generated [`Authorizable`], which names the
//! operation and the securable of each request, so handlers call
//! [`Policy::check_required`] with the request itself. Requests whose securable
//! depends on their content (grants, tag assignments, ABAC policies, temporary
//! credentials addressed by id) are authorized with an explicit
//! [`Policy::authorize_checked`] once the securable is resolved.
//!
//! # Provided implementations
//!
//! [`ConstantPolicy`] returns a fixed decision; [`ConstantPolicy::default`]
//! allows everything, which suits development and deployments where a trusted
//! proxy enforces access control upstream. With the `olai-auth` feature,
//! [`AuthorizerPolicy`] asks a Cedar policy decision point, over the vocabulary
//! of [`CEDAR_SCHEMA_FRAGMENTS`].
//!
//! # Composing a policy into a handler
//!
//! A type that holds a policy behind an [`Arc`] implements [`ProvidesPolicy`],
//! and a blanket impl then makes that type act as a [`Policy`] by delegation, so
//! handlers can call [`check_required`](Policy::check_required) on `self`
//! directly. List endpoints use [`process_resources`] (or [`filter_authorized`]
//! for a `dyn`-typed policy) to drop entries the caller may not see: each entry
//! is checked with the `Get` operation of its kind.
//!
//! # Examples
//!
//! ```
//! use unitycatalog_server::policy::{ConstantPolicy, Decision, Facts, Operation, Policy, Securable};
//! use unitycatalog_common::models::ObjectLabel;
//!
//! # async fn run() -> unitycatalog_server::Result<()> {
//! // An allow-all policy authorized against the unit context.
//! let policy = ConstantPolicy::default();
//! let catalog = Securable::new(ObjectLabel::Catalog, vec!["main".to_string()]);
//!
//! let decision = policy
//!     .authorize(Operation::GetCatalog.into(), &catalog, &Facts::default(), &())
//!     .await?;
//! assert_eq!(decision, Decision::Allow);
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use olai_storage_credentials::AccessLevel;
use unitycatalog_common::models::{ObjectLabel, ResourceExt, ResourceIdent, ResourceName};
pub use unitycatalog_handlers::authz::{Authorizable, Operation, Securable, SecurableKind};

#[cfg(feature = "olai-auth")]
pub use self::authorizer::AuthorizerPolicy;
pub use self::constant::*;
use crate::{Error, Result};

#[cfg(feature = "olai-auth")]
mod authorizer;
mod constant;

pub use unitycatalog_handlers::Principal;

/// The Cedar namespace of the server's vocabulary.
pub const CEDAR_NAMESPACE: &str = "UnityCatalog";

/// The id of the root entity, `UnityCatalog::Metastore::"metastore"`.
pub const METASTORE_ID: &str = "metastore";

/// The entity kinds of the containers a schema-level securable sits in, below
/// the metastore, outermost first.
const IN_SCHEMA: &[&str] = &["Catalog", "Schema"];

/// The Cedar entity kind of `label` (`Table`), and the kinds of the containers it
/// sits in below the metastore, outermost first (`[Catalog, Schema]`).
pub fn entity_kind(label: ObjectLabel) -> (&'static str, &'static [&'static str]) {
    match label {
        ObjectLabel::Catalog => ("Catalog", &[]),
        ObjectLabel::Schema => ("Schema", &["Catalog"]),
        ObjectLabel::Table => ("Table", IN_SCHEMA),
        ObjectLabel::Volume => ("Volume", IN_SCHEMA),
        ObjectLabel::Function => ("Function", IN_SCHEMA),
        ObjectLabel::RegisteredModel => ("RegisteredModel", IN_SCHEMA),
        ObjectLabel::Agent => ("Agent", IN_SCHEMA),
        ObjectLabel::AgentSkill => ("AgentSkill", IN_SCHEMA),
        ObjectLabel::StagingTable => ("StagingTable", IN_SCHEMA),
        ObjectLabel::ModelVersion => ("ModelVersion", &["Catalog", "Schema", "RegisteredModel"]),
        ObjectLabel::Column => ("Column", &["Catalog", "Schema", "Table"]),
        ObjectLabel::ExternalLocation => ("ExternalLocation", &[]),
        ObjectLabel::Credential => ("Credential", &[]),
        ObjectLabel::Share => ("Share", &[]),
        ObjectLabel::Recipient => ("Recipient", &[]),
        ObjectLabel::Provider => ("Provider", &[]),
        ObjectLabel::TagPolicy => ("TagPolicy", &[]),
        ObjectLabel::PolicyInfo => ("PolicyInfo", &[]),
    }
}

/// The server's Cedar entities (`namespace UnityCatalog`): the securables, their
/// typed ancestors and loader-owned privilege attributes, and the request context.
pub const CEDAR_ENTITIES: &str = include_str!("../../cedar/UnityCatalog.cedarschema");

/// The Cedar actions of the Delta API that have no REST operation of their own.
pub const CEDAR_DELTA_ACTIONS: &str = include_str!("../../cedar/DeltaApi.cedarschema");

/// The Cedar actions of the storage byte-proxy (reading and writing a
/// securable's data through the server).
pub const CEDAR_STORAGE_PROXY_ACTIONS: &str = include_str!("../../cedar/StorageProxy.cedarschema");

/// The generated Cedar actions, one per API [`Operation`].
pub use unitycatalog_handlers::CEDAR_ACTIONS;

/// The server's Cedar vocabulary, as schema fragments: [`CEDAR_ENTITIES`], the
/// generated [`CEDAR_ACTIONS`], [`CEDAR_DELTA_ACTIONS`] and
/// [`CEDAR_STORAGE_PROXY_ACTIONS`].
///
/// Each declares `namespace UnityCatalog`, so they are composed as fragments
/// (Cedar rejects one schema text declaring a namespace twice). A host adds its
/// `Lakehouse` fragment, which declares the principals (`User`,
/// `ServiceAccount`) and their groups.
pub const CEDAR_SCHEMA_FRAGMENTS: &[&str] = &[
    CEDAR_ENTITIES,
    CEDAR_ACTIONS,
    CEDAR_DELTA_ACTIONS,
    CEDAR_STORAGE_PROXY_ACTIONS,
];

/// The Cedar actions of the Delta API without a REST operation, and of the
/// storage byte-proxy.
///
/// The Delta API asks the REST operation where one fits (see the header of
/// `cedar/DeltaApi.cedarschema`); these are the rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServerAction {
    /// One of the API's operations.
    Operation(Operation),
    /// Commit a new version of a table.
    CommitTable,
    /// Rename a table (asked under its current name).
    RenameTable,
    /// Turn a staging table into a managed table.
    AdoptStagingTable,
    /// Receive credentials for a staging table's location.
    GenerateTemporaryStagingTableCredentials,
    /// Read a securable's data through the storage byte-proxy.
    ReadData,
    /// Write a securable's data through the storage byte-proxy.
    WriteData,
}

impl ServerAction {
    /// The Cedar action id.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Operation(op) => op.as_str(),
            Self::CommitTable => "CommitTable",
            Self::RenameTable => "RenameTable",
            Self::AdoptStagingTable => "AdoptStagingTable",
            Self::GenerateTemporaryStagingTableCredentials => {
                "GenerateTemporaryStagingTableCredentials"
            }
            Self::ReadData => "ReadData",
            Self::WriteData => "WriteData",
        }
    }
}

impl From<Operation> for ServerAction {
    fn from(op: Operation) -> Self {
        Self::Operation(op)
    }
}

impl std::fmt::Display for ServerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The access a credential is asked for (`context.credential_operation`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialOperation {
    /// Read only (`READ`).
    Read,
    /// Read and write (`READ_WRITE`).
    ReadWrite,
}

impl CredentialOperation {
    /// The name the policy sees.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "READ",
            Self::ReadWrite => "READ_WRITE",
        }
    }
}

impl From<AccessLevel> for CredentialOperation {
    fn from(access: AccessLevel) -> Self {
        match access {
            AccessLevel::ReadWrite => Self::ReadWrite,
            _ => Self::Read,
        }
    }
}

/// The facts of a request a decision may need beyond the operation and the
/// securable. They become optional keys of the Cedar `RequestContext`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Facts {
    /// Credential vends: the access asked for.
    pub credential_operation: Option<CredentialOperation>,
    /// Path vends and path proxy requests: the storage URL asked for.
    pub url: Option<String>,
    /// Updates: the fields the request changes.
    pub update_fields: Option<Vec<String>>,
    /// Adopting a staging table: whether the caller reserved it.
    pub is_creator: Option<bool>,
}

impl Facts {
    /// The facts of a credential vend at `access`.
    pub fn credential(access: impl Into<CredentialOperation>) -> Self {
        Self {
            credential_operation: Some(access.into()),
            ..Default::default()
        }
    }

    /// These facts with the storage URL `url`.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// These facts with the fields an update changes.
    pub fn with_update_fields<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.update_fields = Some(fields.into_iter().map(Into::into).collect());
        self
    }
}

/// Decision made by a policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Allow the action.
    Allow,
    /// Deny the action.
    Deny,
}

/// Access-control decision point for the server.
///
/// Implementations decide whether a caller (described by the context `Cx`) may
/// perform an action on a securable. [`authorize`](Self::authorize) is the only
/// required method; every other method has a default that delegates to it. The
/// trait is `async` because real backends (a remote decision point, database
/// lookups) are I/O bound.
///
/// `Cx` is the authorization context — the identity or request state the policy
/// evaluates against, e.g. [`RequestContext`](crate::api::RequestContext) or `()`.
#[async_trait::async_trait]
pub trait Policy<Cx: Send + Sync + 'static>: Send + Sync + 'static {
    /// Decides whether `context` may perform `action` on `securable`, given the
    /// request's `facts`.
    ///
    /// This is the one method an implementation must provide.
    async fn authorize(
        &self,
        action: ServerAction,
        securable: &Securable,
        facts: &Facts,
        context: &Cx,
    ) -> Result<Decision>;

    /// Authorizes the same `action` on many securables, returning one
    /// [`Decision`] per securable in the same order.
    ///
    /// The default calls [`authorize`](Self::authorize) sequentially;
    /// implementations backed by a batchable service should override it.
    async fn authorize_many(
        &self,
        action: ServerAction,
        securables: &[Securable],
        facts: &Facts,
        context: &Cx,
    ) -> Result<Vec<Decision>> {
        let mut decisions = Vec::with_capacity(securables.len());
        for securable in securables {
            decisions.push(self.authorize(action, securable, facts, context).await?);
        }
        Ok(decisions)
    }

    /// Like [`authorize`](Self::authorize), but maps a [`Deny`](Decision::Deny)
    /// to [`Error::NotAllowed`].
    async fn authorize_checked(
        &self,
        action: ServerAction,
        securable: &Securable,
        facts: &Facts,
        context: &Cx,
    ) -> Result<()> {
        match self.authorize(action, securable, facts, context).await? {
            Decision::Allow => Ok(()),
            Decision::Deny => Err(Error::NotAllowed),
        }
    }

    /// Authorizes a request: its operation on its securable, with no facts.
    async fn check(&self, request: &dyn Authorizable, context: &Cx) -> Result<Decision> {
        self.authorize(
            request.operation().into(),
            &request.securable(),
            &Facts::default(),
            context,
        )
        .await
    }

    /// Like [`check`](Self::check), but maps a [`Deny`](Decision::Deny) to
    /// [`Error::NotAllowed`] so a handler can propagate it with `?`.
    async fn check_required(&self, request: &dyn Authorizable, context: &Cx) -> Result<()> {
        match self.check(request, context).await? {
            Decision::Allow => Ok(()),
            Decision::Deny => Err(Error::NotAllowed),
        }
    }
}

/// Types that own a [`Policy`] and want to act as one.
///
/// A blanket impl makes any `ProvidesPolicy<Cx>` implement `Policy<Cx>` by
/// delegating every method to [`policy`](Self::policy), so a composed handler
/// can call [`check_required`](Policy::check_required) on `self` without reaching
/// into the wrapped policy explicitly.
pub trait ProvidesPolicy<Cx: Send + Sync + 'static>: Send + Sync + 'static {
    /// Returns the policy this type delegates authorization to.
    fn policy(&self) -> &Arc<dyn Policy<Cx>>;
}

#[async_trait::async_trait]
impl<T: Policy<Cx>, Cx: Send + Sync + 'static> Policy<Cx> for Arc<T> {
    async fn authorize(
        &self,
        action: ServerAction,
        securable: &Securable,
        facts: &Facts,
        context: &Cx,
    ) -> Result<Decision> {
        T::authorize(self, action, securable, facts, context).await
    }

    async fn authorize_many(
        &self,
        action: ServerAction,
        securables: &[Securable],
        facts: &Facts,
        context: &Cx,
    ) -> Result<Vec<Decision>> {
        T::authorize_many(self, action, securables, facts, context).await
    }
}

/// The securable of `ident`: its label and, when addressed by name, its name.
///
/// A securable addressed by id has no name here; resolve it first.
pub fn securable_of(ident: &ResourceIdent) -> Securable {
    use unitycatalog_common::models::ResourceRef;
    let name = match ident.reference() {
        ResourceRef::Name(name) => name.path().to_vec(),
        _ => Vec::new(),
    };
    Securable {
        label: Some(*ident.label()),
        name,
    }
}

/// The store ident of `securable`, by name. The service root has none.
pub fn ident_of(securable: &Securable) -> Option<ResourceIdent> {
    securable
        .label
        .map(|label| label.to_ident(ResourceName::new(securable.name.iter())))
}

/// The securable `resource` is, by its full name (falling back to its own ident
/// when it has no name).
fn securable_of_resource<R: ResourceExt>(resource: &R) -> Securable {
    let name = resource.resource_name();
    if name.path().iter().all(String::is_empty) {
        securable_of(&resource.resource_ident())
    } else {
        Securable {
            label: Some(*resource.resource_ident().label()),
            name: name.path().to_vec(),
        }
    }
}

/// Filters a `Vec` of resources in place, keeping only those the context may
/// see.
///
/// Each resource is checked with the `Get` operation of its kind
/// ([`Operation::get_of`]), in one [`Policy::authorize_many`] batch per kind,
/// preserving order. A kind without a `Get` operation is dropped. List
/// endpoints call this before returning results. For a policy held behind a
/// trait object, use [`filter_authorized`].
pub async fn process_resources<
    T: Policy<Cx> + Sized,
    Cx: Send + Sync + 'static,
    R: ResourceExt + Send,
>(
    handler: &T,
    context: &Cx,
    resources: &mut Vec<R>,
) -> Result<()> {
    filter_authorized(handler, context, resources).await
}

/// [`process_resources`] for a `dyn`-typed policy.
pub async fn filter_authorized<Cx: Send + Sync + 'static, R: ResourceExt + Send>(
    policy: &dyn Policy<Cx>,
    context: &Cx,
    resources: &mut Vec<R>,
) -> Result<()> {
    let securables: Vec<Securable> = resources.iter().map(securable_of_resource).collect();
    let allowed = allowed(policy, context, &securables).await?;
    let mut allow = allowed.into_iter();
    resources.retain(|_| allow.next().unwrap_or(false));
    Ok(())
}

/// Whether `context` may see each of `securables`, in order: the `Get`
/// operation of its kind on it, batched per kind.
pub(crate) async fn allowed<Cx: Send + Sync + 'static>(
    policy: &dyn Policy<Cx>,
    context: &Cx,
    securables: &[Securable],
) -> Result<Vec<bool>> {
    let mut allowed = vec![false; securables.len()];
    let mut by_label: Vec<(ObjectLabel, Vec<usize>)> = Vec::new();
    for (i, securable) in securables.iter().enumerate() {
        let Some(label) = securable.label else {
            continue;
        };
        match by_label.iter_mut().find(|(l, _)| *l == label) {
            Some((_, indices)) => indices.push(i),
            None => by_label.push((label, vec![i])),
        }
    }
    for (label, indices) in by_label {
        let Some(op) = Operation::get_of(label) else {
            continue;
        };
        let batch: Vec<Securable> = indices.iter().map(|&i| securables[i].clone()).collect();
        let decisions = policy
            .authorize_many(op.into(), &batch, &Facts::default(), context)
            .await?;
        // `decisions[k]` answers `batch[k]`; a short answer denies the rest.
        for (k, &i) in indices.iter().enumerate() {
            allowed[i] = decisions.get(k) == Some(&Decision::Allow);
        }
    }
    Ok(allowed)
}

#[cfg(test)]
mod test {
    use super::*;
    use unitycatalog_common::models::{ResourceName, ResourceRef};

    /// Minimal resource carrying a catalog name, for exercising [`filter_authorized`].
    #[derive(Debug, Clone, PartialEq)]
    struct TestCatalog(&'static str);

    impl ResourceExt for TestCatalog {
        fn resource_name(&self) -> ResourceName {
            ResourceName::new([self.0])
        }
        fn resource_ref(&self) -> ResourceRef {
            ResourceRef::Name(self.resource_name())
        }
        fn resource_ident(&self) -> ResourceIdent {
            ResourceIdent::catalog(self.resource_name())
        }
    }

    /// Allows `GetCatalog` on the listed catalog names only.
    struct AllowListPolicy {
        allow: Vec<&'static str>,
    }

    #[async_trait::async_trait]
    impl Policy<()> for AllowListPolicy {
        async fn authorize(
            &self,
            action: ServerAction,
            securable: &Securable,
            _facts: &Facts,
            _context: &(),
        ) -> Result<Decision> {
            assert_eq!(action, Operation::GetCatalog.into());
            let name = securable.name.join(".");
            Ok(if self.allow.contains(&name.as_str()) {
                Decision::Allow
            } else {
                Decision::Deny
            })
        }
    }

    /// A non-uniform policy filters the right resources, in their original order.
    #[tokio::test]
    async fn filter_authorized_pairs_decision_with_correct_resource() {
        let policy = AllowListPolicy {
            allow: vec!["a", "c"],
        };
        let mut resources = vec![
            TestCatalog("a"),
            TestCatalog("b"),
            TestCatalog("c"),
            TestCatalog("d"),
        ];
        filter_authorized(&policy, &(), &mut resources)
            .await
            .unwrap();
        assert_eq!(resources, vec![TestCatalog("a"), TestCatalog("c")]);
    }

    #[test]
    fn securables_and_idents_round_trip() {
        let ident = ResourceIdent::table(ResourceName::new(["c", "s", "t"]));
        let securable = securable_of(&ident);
        assert_eq!(securable.label, Some(ObjectLabel::Table));
        assert_eq!(securable.name, ["c", "s", "t"]);
        assert_eq!(ident_of(&securable), Some(ident));
        assert_eq!(ident_of(&Securable::root()), None);
    }

    #[test]
    fn the_schema_is_every_fragment() {
        let schema = CEDAR_SCHEMA_FRAGMENTS.join("\n");
        for part in [
            "entity Metastore",
            "action \"GetTable\"",
            "action \"CommitTable\"",
            "action \"ReadData\"",
        ] {
            assert!(schema.contains(part), "{part} is missing");
        }
    }
}
