//! The directory over Postgres: Neon Auth's users and olai-auth's `authz`
//! groups and service accounts.

use async_trait::async_trait;
use olai_auth::grants::SubjectKind;
use sqlx::PgPool;

use super::{AuthzDirectory, Group, Member, ServicePrincipal, User, UserSource};
use crate::{Error, Result};

fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

fn db_error(e: sqlx::Error) -> Error {
    Error::Generic(format!("directory: {e}"))
}

/// Neon Auth's users, from the `user` table of its schema (`neon_auth`).
///
/// A user is active unless banned with no expiry or one still ahead. The
/// connecting role needs `SELECT` on `id`, `email`, `name`, `banned` and
/// `"banExpires"`.
#[derive(Debug, Clone)]
pub struct NeonUsers {
    pool: PgPool,
    table: String,
}

impl NeonUsers {
    pub fn new(pool: PgPool, schema: &str) -> Self {
        Self {
            pool,
            table: format!("{}.\"user\"", quote_ident(schema)),
        }
    }

    async fn fetch(&self, filter: &str, value: Option<&str>) -> Result<Vec<User>> {
        let query = format!(
            "SELECT id::text, email, name, \
               NOT (coalesce(banned, false) AND (\"banExpires\" IS NULL OR \"banExpires\" > now())) \
             FROM {} {filter} ORDER BY lower(email)",
            self.table
        );
        let mut query = sqlx::query_as::<_, (String, String, Option<String>, bool)>(&query);
        if let Some(value) = value {
            query = query.bind(value);
        }
        let rows = query.fetch_all(&self.pool).await.map_err(db_error)?;
        Ok(rows
            .into_iter()
            .map(|(id, email, name, active)| User {
                id,
                user_name: email.clone(),
                display_name: name.filter(|n| !n.is_empty()),
                emails: vec![email],
                active,
            })
            .collect())
    }
}

#[async_trait]
impl UserSource for NeonUsers {
    async fn users(&self) -> Result<Vec<User>> {
        self.fetch("", None).await
    }

    async fn user(&self, id: &str) -> Result<Option<User>> {
        Ok(self
            .fetch("WHERE id::text = $1", Some(id))
            .await?
            .into_iter()
            .next())
    }

    async fn user_by_name(&self, name: &str) -> Result<Option<User>> {
        Ok(self
            .fetch("WHERE lower(email) = lower($1)", Some(name))
            .await?
            .into_iter()
            .next())
    }
}

/// The groups and service accounts of olai-auth's `authz` schema.
///
/// The connecting role needs `SELECT` on `groups`, `memberships` and
/// `service_accounts`.
#[derive(Debug, Clone)]
pub struct PgAuthz {
    pool: PgPool,
    schema: String,
}

impl PgAuthz {
    pub fn new(pool: PgPool, schema: &str) -> Self {
        Self {
            pool,
            schema: quote_ident(schema),
        }
    }

    async fn fetch_groups(&self, filter: &str, value: Option<&str>) -> Result<Vec<Group>> {
        let s = &self.schema;
        let query = format!(
            "SELECT g.id::text, g.name, m.principal_type, m.principal_id \
             FROM {s}.groups g LEFT JOIN {s}.memberships m ON m.group_id = g.id \
             {filter} ORDER BY g.name, m.created_at"
        );
        let mut query =
            sqlx::query_as::<_, (String, String, Option<String>, Option<String>)>(&query);
        if let Some(value) = value {
            query = query.bind(value);
        }
        let rows = query.fetch_all(&self.pool).await.map_err(db_error)?;
        let mut groups: Vec<Group> = Vec::new();
        for (id, name, kind, member) in rows {
            if groups.last().is_none_or(|g| g.id != id) {
                groups.push(Group {
                    id,
                    name,
                    members: vec![],
                });
            }
            let (Some(kind), Some(member)) = (kind, member) else {
                continue;
            };
            let kind = match kind.as_str() {
                "user" => SubjectKind::User,
                "group" => SubjectKind::Group,
                "service_account" => SubjectKind::ServiceAccount,
                _ => continue,
            };
            if let Some(group) = groups.last_mut() {
                group.members.push(Member { kind, id: member });
            }
        }
        Ok(groups)
    }

    async fn fetch_accounts(
        &self,
        filter: &str,
        value: Option<&str>,
    ) -> Result<Vec<ServicePrincipal>> {
        let query = format!(
            "SELECT id::text, name, description, disabled_at IS NULL \
             FROM {}.service_accounts {filter} ORDER BY name",
            self.schema
        );
        let mut query = sqlx::query_as::<_, (String, String, Option<String>, bool)>(&query);
        if let Some(value) = value {
            query = query.bind(value);
        }
        let rows = query.fetch_all(&self.pool).await.map_err(db_error)?;
        Ok(rows
            .into_iter()
            .map(|(id, name, description, active)| ServicePrincipal {
                id,
                name,
                description,
                active,
            })
            .collect())
    }
}

#[async_trait]
impl AuthzDirectory for PgAuthz {
    async fn groups(&self) -> Result<Vec<Group>> {
        self.fetch_groups("", None).await
    }

    async fn service_principals(&self) -> Result<Vec<ServicePrincipal>> {
        self.fetch_accounts("", None).await
    }

    async fn group(&self, id: &str) -> Result<Option<Group>> {
        Ok(self
            .fetch_groups("WHERE g.id::text = $1", Some(id))
            .await?
            .into_iter()
            .next())
    }

    async fn group_by_name(&self, name: &str) -> Result<Option<Group>> {
        Ok(self
            .fetch_groups("WHERE g.name = $1", Some(name))
            .await?
            .into_iter()
            .next())
    }

    async fn service_principal(&self, id: &str) -> Result<Option<ServicePrincipal>> {
        Ok(self
            .fetch_accounts("WHERE id::text = $1", Some(id))
            .await?
            .into_iter()
            .next())
    }

    async fn service_principal_by_name(&self, name: &str) -> Result<Option<ServicePrincipal>> {
        Ok(self
            .fetch_accounts("WHERE name = $1", Some(name))
            .await?
            .into_iter()
            .next())
    }
}
