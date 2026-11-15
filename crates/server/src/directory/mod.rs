//! The principal directory: who the users, groups and service accounts are.
//!
//! Users come from the deployment's identity provider, through a [`UserSource`]:
//! Neon Auth's `neon_auth."user"` table ([`postgres::NeonUsers`]), Authelia's
//! user file ([`authelia::AutheliaUsers`]), or a fixed list ([`StaticUsers`]).
//! Groups, their members and service accounts always come from the
//! authorization store (olai-auth's `authz` schema, [`postgres::PgAuthz`]),
//! through an [`AuthzDirectory`], whichever identity provider signs users in.
//! So grants, the SCIM API and `/Me` behave the same on every provider.
//!
//! This is read-only. Users register through their provider's own flow, and
//! groups and service accounts are managed in the authorization store.
//!
//! A [`Directory`] joins the two. It is also the grants API's
//! [`PrincipalDirectory`]: a principal named by email (or Authelia user name)
//! is a user, any other name a group, else a service account.

use std::sync::Arc;

use async_trait::async_trait;
use olai_auth::grants::{Subject, SubjectKind};
use serde::Deserialize;

use crate::grants::PrincipalDirectory;
use crate::{Error, Result};

#[cfg(feature = "scim")]
pub mod authelia;
#[cfg(feature = "olai-auth")]
pub mod postgres;

/// A user, as the identity provider knows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// The stable id: the grant subject id of the user. Neon Auth's user id, or
    /// Authelia's user name.
    pub id: String,
    /// The name principals are given by: the email address where there is one.
    pub user_name: String,
    /// A friendlier label (a full name), where the provider has one.
    pub display_name: Option<String>,
    /// The user's email addresses, primary first.
    pub emails: Vec<String>,
    /// Whether the user may sign in (not banned or disabled).
    pub active: bool,
}

/// A service account of the authorization store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServicePrincipal {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub active: bool,
}

/// A member of a [`Group`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Member {
    pub kind: SubjectKind,
    pub id: String,
}

/// A group of the authorization store, with its direct members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub members: Vec<Member>,
}

/// The users of an identity provider.
///
/// Only [`users`](Self::users) is required; the lookups default to scanning it,
/// and a source with an index overrides them.
#[async_trait]
pub trait UserSource: Send + Sync + 'static {
    /// Every user.
    async fn users(&self) -> Result<Vec<User>>;

    /// The user with id `id`.
    async fn user(&self, id: &str) -> Result<Option<User>> {
        Ok(self.users().await?.into_iter().find(|u| u.id == id))
    }

    /// The user `name` names: by user name or any email address (ignoring
    /// case), else by id.
    async fn user_by_name(&self, name: &str) -> Result<Option<User>> {
        let users = self.users().await?;
        let by_name = users.iter().position(|u| {
            u.user_name.eq_ignore_ascii_case(name)
                || u.emails.iter().any(|e| e.eq_ignore_ascii_case(name))
        });
        let index = by_name.or_else(|| users.iter().position(|u| u.id == name));
        Ok(index.map(|i| users[i].clone()))
    }
}

/// The groups and service accounts of the authorization store.
///
/// Only the two listings are required; the lookups default to scanning them.
#[async_trait]
pub trait AuthzDirectory: Send + Sync + 'static {
    /// Every group, with its direct members.
    async fn groups(&self) -> Result<Vec<Group>>;

    /// Every service account.
    async fn service_principals(&self) -> Result<Vec<ServicePrincipal>>;

    /// The group with id `id`.
    async fn group(&self, id: &str) -> Result<Option<Group>> {
        Ok(self.groups().await?.into_iter().find(|g| g.id == id))
    }

    /// The group named `name`.
    async fn group_by_name(&self, name: &str) -> Result<Option<Group>> {
        Ok(self.groups().await?.into_iter().find(|g| g.name == name))
    }

    /// The service account with id `id`.
    async fn service_principal(&self, id: &str) -> Result<Option<ServicePrincipal>> {
        Ok(self
            .service_principals()
            .await?
            .into_iter()
            .find(|s| s.id == id))
    }

    /// The service account named `name`.
    async fn service_principal_by_name(&self, name: &str) -> Result<Option<ServicePrincipal>> {
        Ok(self
            .service_principals()
            .await?
            .into_iter()
            .find(|s| s.name == name))
    }
}

/// A fixed list of users, from config or a test.
#[derive(Debug, Clone, Default)]
pub struct StaticUsers(Vec<User>);

impl StaticUsers {
    pub fn new(users: Vec<User>) -> Self {
        Self(users)
    }
}

#[async_trait]
impl UserSource for StaticUsers {
    async fn users(&self) -> Result<Vec<User>> {
        Ok(self.0.clone())
    }
}

/// Groups and service accounts held in memory: none by default.
#[derive(Debug, Clone, Default)]
pub struct MemoryAuthz {
    groups: Vec<Group>,
    service_principals: Vec<ServicePrincipal>,
}

impl MemoryAuthz {
    pub fn new(groups: Vec<Group>, service_principals: Vec<ServicePrincipal>) -> Self {
        Self {
            groups,
            service_principals,
        }
    }
}

#[async_trait]
impl AuthzDirectory for MemoryAuthz {
    async fn groups(&self) -> Result<Vec<Group>> {
        Ok(self.groups.clone())
    }

    async fn service_principals(&self) -> Result<Vec<ServicePrincipal>> {
        Ok(self.service_principals.clone())
    }
}

/// A user of [`StaticUsers`], as configured.
#[derive(Debug, Clone, Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct StaticUser {
    /// The grant subject id. Defaults to the user name.
    #[serde(default)]
    pub id: Option<String>,
    /// The user's name, usually an email address.
    pub user_name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    /// Defaults to the user name when it is an email address.
    #[serde(default)]
    pub emails: Vec<String>,
    #[serde(default = "default_true")]
    pub active: bool,
}

fn default_true() -> bool {
    true
}

impl From<StaticUser> for User {
    fn from(user: StaticUser) -> Self {
        let emails = if user.emails.is_empty() && user.user_name.contains('@') {
            vec![user.user_name.clone()]
        } else {
            user.emails
        };
        Self {
            id: user.id.unwrap_or_else(|| user.user_name.clone()),
            user_name: user.user_name,
            display_name: user.display_name,
            emails,
            active: user.active,
        }
    }
}

/// The users of an identity provider and the groups and service accounts of
/// the authorization store, together.
#[derive(Clone)]
pub struct Directory {
    users: Arc<dyn UserSource>,
    authz: Arc<dyn AuthzDirectory>,
}

impl std::fmt::Debug for Directory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Directory").finish_non_exhaustive()
    }
}

impl Directory {
    pub fn new(users: Arc<dyn UserSource>, authz: Arc<dyn AuthzDirectory>) -> Self {
        Self { users, authz }
    }

    /// No users, groups or service accounts.
    pub fn empty() -> Self {
        Self::new(
            Arc::new(StaticUsers::default()),
            Arc::new(MemoryAuthz::default()),
        )
    }

    /// The identity provider's users.
    pub fn users(&self) -> &Arc<dyn UserSource> {
        &self.users
    }

    /// The authorization store's groups and service accounts.
    pub fn authz(&self) -> &Arc<dyn AuthzDirectory> {
        &self.authz
    }

    /// The direct groups of `member`.
    pub async fn groups_of(&self, member: &Member) -> Result<Vec<Group>> {
        Ok(self
            .authz
            .groups()
            .await?
            .into_iter()
            .filter(|g| g.members.contains(member))
            .collect())
    }

    /// The name `subject` is shown by, if it still exists.
    pub async fn subject_name(&self, subject: &Subject) -> Result<Option<String>> {
        Ok(match subject.kind {
            SubjectKind::User => self.users.user(&subject.id).await?.map(|u| u.user_name),
            SubjectKind::Group => self.authz.group(&subject.id).await?.map(|g| g.name),
            SubjectKind::ServiceAccount => self
                .authz
                .service_principal(&subject.id)
                .await?
                .map(|s| s.name),
        })
    }
}

#[async_trait]
impl PrincipalDirectory for Directory {
    async fn resolve(&self, principal: &str) -> Result<Subject> {
        if principal.is_empty() {
            return Err(Error::invalid_argument("principal must not be empty"));
        }
        // An email names a user only; a bare name is a group, else a service
        // account, else (Authelia) a user name.
        if principal.contains('@') {
            if let Some(user) = self.users.user_by_name(principal).await? {
                return Ok(Subject::user(user.id));
            }
        } else if let Some(group) = self.authz.group_by_name(principal).await? {
            return Ok(Subject::group(group.id));
        } else if let Some(account) = self.authz.service_principal_by_name(principal).await? {
            return Ok(Subject::service_account(account.id));
        } else if let Some(user) = self.users.user_by_name(principal).await? {
            return Ok(Subject::user(user.id));
        }
        Err(Error::invalid_argument(format!(
            "unknown principal '{principal}'"
        )))
    }

    async fn name(&self, subject: &Subject) -> Result<String> {
        // A subject that is gone (a deleted user) is shown by its id.
        Ok(self
            .subject_name(subject)
            .await?
            .unwrap_or_else(|| subject.id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(id: &str, name: &str) -> User {
        User {
            id: id.into(),
            user_name: name.into(),
            display_name: None,
            emails: vec![name.into()],
            active: true,
        }
    }

    fn directory() -> Directory {
        Directory::new(
            Arc::new(StaticUsers::new(vec![
                user("u1", "alice@example.com"),
                User {
                    emails: vec![],
                    ..user("bob", "bob")
                },
            ])),
            Arc::new(MemoryAuthz::new(
                vec![Group {
                    id: "g1".into(),
                    name: "analysts".into(),
                    members: vec![Member {
                        kind: SubjectKind::User,
                        id: "u1".into(),
                    }],
                }],
                vec![ServicePrincipal {
                    id: "s1".into(),
                    name: "etl".into(),
                    description: None,
                    active: true,
                }],
            )),
        )
    }

    #[tokio::test]
    async fn resolves_users_groups_and_service_accounts() {
        let d = directory();
        assert_eq!(
            d.resolve("Alice@Example.com").await.unwrap(),
            Subject::user("u1")
        );
        assert_eq!(d.resolve("analysts").await.unwrap(), Subject::group("g1"));
        assert_eq!(
            d.resolve("etl").await.unwrap(),
            Subject::service_account("s1")
        );
        // A bare name that is no group or account is a user name (Authelia).
        assert_eq!(d.resolve("bob").await.unwrap(), Subject::user("bob"));
    }

    #[tokio::test]
    async fn unknown_and_empty_principals_are_invalid() {
        let d = directory();
        for name in ["", "nobody", "nobody@example.com"] {
            let err = d.resolve(name).await.unwrap_err();
            assert!(matches!(err, Error::InvalidArgument(_)), "{name}: {err:?}");
        }
    }

    #[tokio::test]
    async fn names_subjects_and_falls_back_to_the_id() {
        let d = directory();
        assert_eq!(
            d.name(&Subject::user("u1")).await.unwrap(),
            "alice@example.com"
        );
        assert_eq!(d.name(&Subject::group("g1")).await.unwrap(), "analysts");
        assert_eq!(
            d.name(&Subject::service_account("s1")).await.unwrap(),
            "etl"
        );
        assert_eq!(d.name(&Subject::user("gone")).await.unwrap(), "gone");
    }

    #[tokio::test]
    async fn groups_of_a_member() {
        let d = directory();
        let member = Member {
            kind: SubjectKind::User,
            id: "u1".into(),
        };
        let groups = d.groups_of(&member).await.unwrap();
        assert_eq!(
            groups.iter().map(|g| g.name.as_str()).collect::<Vec<_>>(),
            ["analysts"]
        );
    }

    #[test]
    fn static_users_default_their_id_and_email() {
        let user: User = StaticUser {
            id: None,
            user_name: "a@example.com".into(),
            display_name: None,
            emails: vec![],
            active: true,
        }
        .into();
        assert_eq!(user.id, "a@example.com");
        assert_eq!(user.emails, ["a@example.com"]);
    }
}
