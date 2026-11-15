//! Authelia's users, from its file user database (`users.yml`).
//!
//! ```yaml
//! users:
//!   alice:
//!     displayname: Alice Liddell
//!     email: alice@example.com
//!     password: $argon2id$...
//!     disabled: false
//!     groups: [admins]
//! ```
//!
//! A user's id (the grant subject, and the `Remote-User` Envoy forwards) is its
//! key; its user name is its email where it has one. Passwords and groups are
//! ignored: groups come from the authorization store, like everywhere else.
//! The file is read again whenever its modification time changes.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::SystemTime;

use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::Mutex;

use super::{User, UserSource};
use crate::{Error, Result};

#[derive(Debug, Deserialize)]
struct UsersFile {
    #[serde(default)]
    users: BTreeMap<String, FileUser>,
}

#[derive(Debug, Deserialize)]
struct FileUser {
    #[serde(default)]
    displayname: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    disabled: bool,
}

/// The users of an Authelia file user database.
#[derive(Debug)]
pub struct AutheliaUsers {
    path: PathBuf,
    cache: Mutex<Option<(SystemTime, Vec<User>)>>,
}

impl AutheliaUsers {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            cache: Mutex::new(None),
        }
    }

    fn read_error(&self, e: impl std::fmt::Display) -> Error {
        Error::Generic(format!("authelia users file {}: {e}", self.path.display()))
    }
}

/// The users `yaml` (an Authelia users file) lists, by user name.
pub fn parse_users(yaml: &str) -> std::result::Result<Vec<User>, String> {
    let file: UsersFile = serde_yml::from_str(yaml).map_err(|e| e.to_string())?;
    Ok(file
        .users
        .into_iter()
        .map(|(name, user)| {
            let email = user.email.filter(|e| !e.is_empty());
            User {
                user_name: email.clone().unwrap_or_else(|| name.clone()),
                id: name,
                display_name: user.displayname.filter(|n| !n.is_empty()),
                emails: email.into_iter().collect(),
                active: !user.disabled,
            }
        })
        .collect())
}

#[async_trait]
impl UserSource for AutheliaUsers {
    async fn users(&self) -> Result<Vec<User>> {
        let modified = tokio::fs::metadata(&self.path)
            .await
            .and_then(|m| m.modified())
            .map_err(|e| self.read_error(e))?;
        let mut cache = self.cache.lock().await;
        if let Some((at, users)) = cache.as_ref()
            && *at == modified
        {
            return Ok(users.clone());
        }
        let yaml = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|e| self.read_error(e))?;
        let users = parse_users(&yaml).map_err(|e| self.read_error(e))?;
        *cache = Some((modified, users.clone()));
        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = r#"
users:
  alice:
    displayname: Alice Liddell
    email: alice@example.com
    password: "$argon2id$v=19$m=65536,t=3,p=4$x$y"
    groups: [admins]
  bob:
    disabled: true
    password: "$argon2id$v=19$m=65536,t=3,p=4$x$y"
"#;

    #[test]
    fn parses_users_by_key() {
        let users = parse_users(FILE).unwrap();
        assert_eq!(
            users,
            [
                User {
                    id: "alice".into(),
                    user_name: "alice@example.com".into(),
                    display_name: Some("Alice Liddell".into()),
                    emails: vec!["alice@example.com".into()],
                    active: true,
                },
                User {
                    id: "bob".into(),
                    user_name: "bob".into(),
                    display_name: None,
                    emails: vec![],
                    active: false,
                },
            ]
        );
    }

    #[test]
    fn rejects_a_malformed_file() {
        assert!(parse_users("users: [").is_err());
    }

    #[tokio::test]
    async fn rereads_the_file_when_it_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("users.yml");
        std::fs::write(&path, FILE).unwrap();
        let source = AutheliaUsers::new(&path);
        assert_eq!(source.users().await.unwrap().len(), 2);
        assert_eq!(
            source.user_by_name("alice").await.unwrap().unwrap().id,
            "alice"
        );

        std::fs::write(&path, "users:\n  carol:\n    email: c@example.com\n").unwrap();
        // Make sure the mtime moves even on coarse-grained file systems.
        let later = SystemTime::now() + std::time::Duration::from_secs(5);
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(later)
            .unwrap();
        let users = source.users().await.unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].user_name, "c@example.com");
    }

    #[tokio::test]
    async fn a_missing_file_is_an_error() {
        let source = AutheliaUsers::new("/nonexistent/users.yml");
        assert!(source.users().await.is_err());
    }
}
