//! Grants: the privileges principals hold on securables, and ownership.
//!
//! Grants live in an [`olai_auth::grants::GrantStore`], as `(subject, privilege,
//! object)` tuples over the Cedar entities of the securables
//! (`UnityCatalog::Table::"main.sales.orders"`), which is what a decision
//! point's entity loader reads to decide. The grants API
//! ([`GrantHandler`](crate::api::GrantHandler)) writes them; the handlers of the
//! securables keep them in step: the creator of a securable owns it (an
//! [`OWNER`] tuple), an owner update moves that tuple, a delete drops the
//! securable's grants and its descendants', and a rename re-keys them.
//!
//! The grants API names principals the way Unity Catalog does (a user's email, a
//! group's name, a service account's name); a [`PrincipalDirectory`] maps those
//! to the store's [`Subject`]s and back.
//!
//! Privileges are the canonical names of the grants API's
//! [`Privilege`] enum
//! (`USE_CATALOG`); the API accepts the OSS spelling with spaces too
//! (`USE CATALOG`) and answers with it.

use std::sync::Arc;

use async_trait::async_trait;
use buffa::Enumeration;
use olai_auth::grants::{GrantChange, GrantError, GrantStore, ObjectRef, Subject};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::grants::v1::Privilege;

use crate::api::RequestContext;
use crate::policy::{CEDAR_NAMESPACE, METASTORE_ID, Securable, entity_kind};
use crate::{Error, Result};

/// The privilege of the owner of a securable. Not grantable through the grants
/// API: ownership changes by updating the securable's `owner`.
pub const OWNER: &str = "OWNER";

/// Maps the principals of the grants API (a user's email, a group's or service
/// account's name) to grant subjects, and back.
#[async_trait]
pub trait PrincipalDirectory: Send + Sync + 'static {
    /// The subject `principal` names, or [`Error::InvalidArgument`] when it names
    /// none.
    async fn resolve(&self, principal: &str) -> Result<Subject>;

    /// The principal name of `subject`, as the grants API shows it.
    async fn name(&self, subject: &Subject) -> Result<String>;
}

/// A directory without a user database: a principal is the user of that name,
/// and a subject is shown by its id. Fits the in-memory and reverse-proxy
/// deployments, where the caller's name is its identity.
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentityDirectory;

#[async_trait]
impl PrincipalDirectory for IdentityDirectory {
    async fn resolve(&self, principal: &str) -> Result<Subject> {
        if principal.is_empty() {
            return Err(Error::invalid_argument("principal must not be empty"));
        }
        Ok(Subject::user(principal))
    }

    async fn name(&self, subject: &Subject) -> Result<String> {
        Ok(subject.id.clone())
    }
}

/// The grant store and principal directory a handler keeps grants in.
#[derive(Clone)]
pub struct Grants {
    pub(crate) store: Arc<dyn GrantStore>,
    pub(crate) principals: Arc<dyn PrincipalDirectory>,
}

impl Grants {
    /// Grants in `store`, with principals resolved through `principals`.
    pub fn new(store: Arc<dyn GrantStore>, principals: Arc<dyn PrincipalDirectory>) -> Self {
        Self { store, principals }
    }

    /// Grants in memory, principals by name ([`IdentityDirectory`]).
    pub fn in_memory() -> Self {
        Self::new(
            Arc::new(olai_auth::grants::MemoryGrantStore::new()),
            Arc::new(IdentityDirectory),
        )
    }

    /// The store.
    pub fn store(&self) -> &Arc<dyn GrantStore> {
        &self.store
    }

    /// The directory.
    pub fn principals(&self) -> &Arc<dyn PrincipalDirectory> {
        &self.principals
    }

    /// Make the caller of `context` the owner of `securable`, as its creator.
    ///
    /// An anonymous caller owns nothing. Runs after the securable is created: a
    /// failure here leaves a securable without an owner (the store and the grant
    /// store are separate), so it fails the request, and an admin can assign an
    /// owner by updating the securable.
    pub async fn record_owner(
        &self,
        securable: &Securable,
        context: &RequestContext,
    ) -> Result<()> {
        let Some(owner) = caller(context) else {
            return Ok(());
        };
        let object = object_of(securable)?;
        let change = GrantChange {
            subject: owner,
            add: vec![OWNER.to_string()],
            remove: vec![],
        };
        self.store
            .apply(&object, &[change])
            .await
            .map(|_| ())
            .map_err(|e| logged(e, "recording the owner of", &object))
    }

    /// Make `owner` (a principal name) the owner of `securable`, instead of
    /// whoever owned it.
    pub async fn transfer_owner(&self, securable: &Securable, owner: &str) -> Result<()> {
        let object = object_of(securable)?;
        let owner = self.principals.resolve(owner).await?;
        let mut changes: Vec<GrantChange> = self
            .store
            .list(&object)
            .await
            .map_err(grant_error)?
            .into_iter()
            .filter(|g| g.privilege == OWNER && g.subject != owner)
            .map(|g| GrantChange {
                subject: g.subject,
                add: vec![],
                remove: vec![OWNER.to_string()],
            })
            .collect();
        changes.push(GrantChange {
            subject: owner,
            add: vec![OWNER.to_string()],
            remove: vec![],
        });
        self.store
            .apply(&object, &changes)
            .await
            .map(|_| ())
            .map_err(|e| logged(e, "transferring the ownership of", &object))
    }

    /// Drop the grants on `securable` and on everything in it, as it is deleted.
    pub async fn forget(&self, securable: &Securable) -> Result<()> {
        let object = object_of(securable)?;
        let types = descendant_types(securable.label.unwrap_or(ObjectLabel::Catalog));
        let types: Vec<&str> = types.iter().map(String::as_str).collect();
        self.store
            .delete_object(&object, &types)
            .await
            .map(|_| ())
            .map_err(|e| logged(e, "dropping the grants of", &object))
    }

    /// Move the grants on `securable` and on everything in it to its new name
    /// (`new_name`, its full name after the rename).
    pub async fn rename(&self, securable: &Securable, new_name: &[String]) -> Result<()> {
        let object = object_of(securable)?;
        let new_id = new_name.join(".");
        if new_id == object.id {
            return Ok(());
        }
        let types = descendant_types(securable.label.unwrap_or(ObjectLabel::Catalog));
        let types: Vec<&str> = types.iter().map(String::as_str).collect();
        self.store
            .rename(&object, &new_id, &types)
            .await
            .map(|_| ())
            .map_err(|e| logged(e, "re-keying the grants of", &object))
    }
}

impl crate::services::ServerHandler<RequestContext> {
    /// Make the caller the owner of `securable`, which it just created.
    pub(crate) async fn own(&self, securable: &Securable, context: &RequestContext) -> Result<()> {
        self.grants().record_owner(securable, context).await
    }

    /// Keep the grants of `securable` in step with an update that renamed it to
    /// `new_name` (its new last name part) and/or gave it to `owner`.
    pub(crate) async fn regrant(
        &self,
        securable: &Securable,
        new_name: Option<&str>,
        owner: Option<&str>,
    ) -> Result<()> {
        let mut current = securable.clone();
        if let Some(new_name) = new_name.filter(|n| !n.is_empty()) {
            let mut name = securable.name.clone();
            if let Some(last) = name.last_mut() {
                *last = new_name.to_string();
            }
            self.grants().rename(securable, &name).await?;
            current.name = name;
        }
        if let Some(owner) = owner.filter(|o| !o.is_empty()) {
            self.grants().transfer_owner(&current, owner).await?;
        }
        Ok(())
    }

    /// Drop the grants of `securable`, which was just deleted, and of
    /// everything in it.
    pub(crate) async fn ungrant(&self, securable: &Securable) -> Result<()> {
        self.grants().forget(securable).await
    }
}

/// The grant subject of the caller of `context`: the verified principal, else
/// the named recipient; none for an anonymous caller.
pub fn caller(context: &RequestContext) -> Option<Subject> {
    #[cfg(feature = "olai-auth")]
    if let Some(principal) = context.verified_principal() {
        return Some(match &principal.kind {
            olai_auth::PrincipalKind::User { sub, .. } => Subject::user(sub.clone()),
            olai_auth::PrincipalKind::ServiceAccount { id, .. } => {
                Subject::service_account(id.clone())
            }
        });
    }
    match context.recipient() {
        crate::policy::Principal::User(name) => Some(Subject::user(name.clone())),
        crate::policy::Principal::Anonymous => None,
    }
}

/// The grant object of `securable`: its Cedar entity, by full name. The
/// service root is the metastore.
pub fn object_of(securable: &Securable) -> Result<ObjectRef> {
    let invalid = || Error::invalid_argument(format!("not a securable: {:?}", securable.name));
    let Some(label) = securable.label else {
        return securable
            .name
            .is_empty()
            .then(|| ObjectRef::new(format!("{CEDAR_NAMESPACE}::Metastore"), METASTORE_ID))
            .ok_or_else(invalid);
    };
    let (kind, containers) = entity_kind(label);
    if securable.name.len() != containers.len() + 1 || securable.name.iter().any(String::is_empty) {
        return Err(invalid());
    }
    Ok(ObjectRef::new(
        format!("{CEDAR_NAMESPACE}::{kind}"),
        securable.name.join("."),
    ))
}

/// The entity types of the securables inside one of `label`, whose grants go
/// with it on a delete or rename.
fn descendant_types(label: ObjectLabel) -> Vec<String> {
    let kinds: &[&str] = match label {
        ObjectLabel::Catalog => &[
            "Schema",
            "Table",
            "Volume",
            "Function",
            "RegisteredModel",
            "ModelVersion",
            "Agent",
            "AgentSkill",
            "StagingTable",
            "Column",
        ],
        ObjectLabel::Schema => &[
            "Table",
            "Volume",
            "Function",
            "RegisteredModel",
            "ModelVersion",
            "Agent",
            "AgentSkill",
            "StagingTable",
            "Column",
        ],
        ObjectLabel::Table => &["Column"],
        ObjectLabel::RegisteredModel => &["ModelVersion"],
        _ => &[],
    };
    kinds
        .iter()
        .map(|kind| format!("{CEDAR_NAMESPACE}::{kind}"))
        .collect()
}

/// The securable of a grants API `securable_type` (the OSS names, lower case)
/// and full name.
pub fn grants_securable(securable_type: &str, full_name: &str) -> Result<Securable> {
    let label = match securable_type.to_ascii_lowercase().as_str() {
        "metastore" => return Ok(Securable::root()),
        "catalog" => ObjectLabel::Catalog,
        "schema" => ObjectLabel::Schema,
        "table" => ObjectLabel::Table,
        "function" => ObjectLabel::Function,
        "volume" => ObjectLabel::Volume,
        "registered_model" => ObjectLabel::RegisteredModel,
        "external_location" => ObjectLabel::ExternalLocation,
        "credential" => ObjectLabel::Credential,
        other => {
            return Err(Error::invalid_argument(format!(
                "unsupported securable_type '{other}'"
            )));
        }
    };
    let securable = Securable::new(label, full_name.split('.').map(str::to_string).collect());
    object_of(&securable)?;
    Ok(securable)
}

/// The canonical name of a grantable privilege, in either spelling (`USE
/// CATALOG`, `USE_CATALOG`, any case). [`OWNER`] and unknown names are refused.
pub fn parse_privilege(raw: &str) -> Result<String> {
    let name = raw.trim().to_ascii_uppercase().replace(' ', "_");
    match Privilege::from_proto_name(&name) {
        Some(Privilege::PRIVILEGE_UNSPECIFIED) | None => Err(Error::invalid_argument(format!(
            "unknown privilege '{raw}'"
        ))),
        Some(privilege) => Ok(privilege.proto_name().to_string()),
    }
}

/// The OSS spelling of the canonical privilege `name`: `USE_CATALOG` is `USE
/// CATALOG`.
pub fn oss_privilege(name: &str) -> String {
    name.replace('_', " ")
}

pub(crate) fn grant_error(e: GrantError) -> Error {
    match e {
        GrantError::InvalidPrivilege(_)
        | GrantError::InvalidObject(_)
        | GrantError::InvalidRename { .. } => Error::invalid_argument(e.to_string()),
        other => Error::Generic(other.to_string()),
    }
}

fn logged(e: GrantError, what: &str, object: &ObjectRef) -> Error {
    tracing::error!(%object, error = %e, "{what} {object} failed");
    grant_error(e)
}

/// A [`PrincipalDirectory`] over Neon Auth's users and the `authz` schema's
/// groups and service accounts.
///
/// A principal with an `@` is the user with that email (`neon_auth."user"`);
/// any other name is a group, else a service account, of that name. The
/// connecting role needs `SELECT` on `id` and `email` of the user table, and on
/// `id` and `name` of `groups` and `service_accounts`.
#[cfg(feature = "olai-auth")]
#[derive(Debug, Clone)]
pub struct PgPrincipalDirectory {
    pool: sqlx::PgPool,
    users: String,
    authz: String,
}

#[cfg(feature = "olai-auth")]
impl PgPrincipalDirectory {
    /// Users from the `user` table of `users_schema`, groups and service
    /// accounts from `authz_schema`.
    pub fn new(pool: sqlx::PgPool, users_schema: &str, authz_schema: &str) -> Self {
        Self {
            pool,
            users: quote_ident(users_schema),
            authz: quote_ident(authz_schema),
        }
    }

    async fn id(&self, query: &str, value: &str) -> Result<Option<String>> {
        sqlx::query_scalar::<_, String>(query)
            .bind(value)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::Generic(format!("principal directory: {e}")))
    }
}

#[cfg(feature = "olai-auth")]
fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

#[cfg(feature = "olai-auth")]
#[async_trait]
impl PrincipalDirectory for PgPrincipalDirectory {
    async fn resolve(&self, principal: &str) -> Result<Subject> {
        let unknown = || Error::invalid_argument(format!("unknown principal '{principal}'"));
        if principal.contains('@') {
            let query = format!(
                "SELECT id::text FROM {}.\"user\" WHERE lower(email) = lower($1)",
                self.users
            );
            return self
                .id(&query, principal)
                .await?
                .map(Subject::user)
                .ok_or_else(unknown);
        }
        let group = format!("SELECT id::text FROM {}.groups WHERE name = $1", self.authz);
        if let Some(id) = self.id(&group, principal).await? {
            return Ok(Subject::group(id));
        }
        let account = format!(
            "SELECT id::text FROM {}.service_accounts WHERE name = $1",
            self.authz
        );
        self.id(&account, principal)
            .await?
            .map(Subject::service_account)
            .ok_or_else(unknown)
    }

    async fn name(&self, subject: &Subject) -> Result<String> {
        use olai_auth::grants::SubjectKind;
        let query = match subject.kind {
            SubjectKind::User => format!(
                "SELECT email FROM {}.\"user\" WHERE id::text = $1",
                self.users
            ),
            SubjectKind::Group => {
                format!("SELECT name FROM {}.groups WHERE id::text = $1", self.authz)
            }
            SubjectKind::ServiceAccount => format!(
                "SELECT name FROM {}.service_accounts WHERE id::text = $1",
                self.authz
            ),
        };
        // A subject that is gone (a deleted user) is shown by its id.
        Ok(self
            .id(&query, &subject.id)
            .await?
            .unwrap_or_else(|| subject.id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privileges_parse_in_both_spellings() {
        assert_eq!(parse_privilege("USE CATALOG").unwrap(), "USE_CATALOG");
        assert_eq!(parse_privilege("use_catalog").unwrap(), "USE_CATALOG");
        assert_eq!(
            parse_privilege("EXTERNAL USE SCHEMA").unwrap(),
            "EXTERNAL_USE_SCHEMA"
        );
        assert_eq!(oss_privilege("EXTERNAL_USE_SCHEMA"), "EXTERNAL USE SCHEMA");
        for refused in ["OWNER", "PRIVILEGE_UNSPECIFIED", "FLY", ""] {
            assert!(parse_privilege(refused).is_err(), "{refused}");
        }
    }

    #[test]
    fn securables_name_their_grant_objects() {
        let table = grants_securable("table", "main.sales.orders").unwrap();
        assert_eq!(
            object_of(&table).unwrap(),
            ObjectRef::new("UnityCatalog::Table", "main.sales.orders")
        );
        assert_eq!(
            object_of(&grants_securable("METASTORE", "").unwrap()).unwrap(),
            ObjectRef::new("UnityCatalog::Metastore", "metastore")
        );
        assert!(grants_securable("table", "main.sales").is_err());
        assert!(grants_securable("share", "s").is_err());
        assert!(grants_securable("catalog", "").is_err());
    }
}
