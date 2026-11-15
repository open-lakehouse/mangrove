//! Principal search over the server's read-only SCIM API
//! (`/api/2.0/preview/scim/v2`, `openapi/scim.yaml`).
//!
//! The Rust port of `node/unity-catalog-client/src/scim.ts`: users match on
//! user name or display name, groups and service principals on display name,
//! all three asked in parallel. The MCP App's iframe can't reach the server,
//! so `uc` makes these calls for it, as the user. A server built without its
//! `scim` feature answers 404; that is "no directory", not an error.

use olai_http::CloudClient;
use serde::Deserialize;
use url::Url;

use super::a2ui::PrincipalRef;
use crate::error::{Error, Result};

/// Where the server mounts the SCIM API, from its root.
pub const SCIM_BASE_PATH: &str = "/api/2.0/preview/scim/v2";

/// The principal kinds, as the host contract spells them.
pub const KINDS: [&str; 3] = ["user", "group", "service_account"];

/// A SCIM client for one server.
#[derive(Clone)]
pub struct Scim {
    client: CloudClient,
    base: Url,
}

#[derive(Deserialize)]
struct ListResponse {
    #[serde(rename = "Resources", default)]
    resources: Vec<Resource>,
}

#[derive(Deserialize)]
struct Resource {
    #[serde(rename = "userName")]
    user_name: Option<String>,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
}

/// `value` as a SCIM string literal.
fn literal(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The filter that finds `query` among principals of `kind`.
fn filter_for(kind: &str, query: &str) -> String {
    let q = literal(query);
    if kind == "user" {
        format!("userName co {q} or displayName co {q}")
    } else {
        format!("displayName co {q}")
    }
}

fn resource_path(kind: &str) -> &'static str {
    match kind {
        "user" => "Users",
        "group" => "Groups",
        _ => "ServicePrincipals",
    }
}

fn to_ref(kind: &str, resource: Resource) -> Option<PrincipalRef> {
    if kind == "user" {
        let name = resource.user_name.filter(|n| !n.is_empty())?;
        Some(PrincipalRef {
            kind: kind.into(),
            name,
            display_name: resource.display_name.filter(|n| !n.is_empty()),
        })
    } else {
        Some(PrincipalRef {
            kind: kind.into(),
            name: resource.display_name.filter(|n| !n.is_empty())?,
            display_name: None,
        })
    }
}

impl Scim {
    /// A client for the SCIM API of the server at `root` (its origin, with
    /// any path the server is mounted under).
    pub fn new(client: CloudClient, root: &Url) -> Self {
        let mut base = root.clone();
        let path = format!("{}{SCIM_BASE_PATH}", root.path().trim_end_matches('/'));
        base.set_path(&path);
        base.set_query(None);
        Self { client, base }
    }

    fn url(&self, resource: &str) -> Result<Url> {
        let mut url = self.base.clone();
        url.set_path(&format!("{}/{resource}", self.base.path()));
        Ok(url)
    }

    /// GET `url`; `None` when the server has no SCIM API (404).
    async fn get<T: for<'de> Deserialize<'de>>(&self, url: Url) -> Result<Option<T>> {
        let response = self
            .client
            .get(url)
            .header("accept", "application/scim+json, application/json")
            .send_unchecked()
            .await
            .map_err(unitycatalog_client::Error::from)?;
        let status = response.status();
        if status.as_u16() == 404 {
            return Ok(None);
        }
        let bytes = response
            .bytes()
            .await
            .map_err(unitycatalog_client::Error::from)?;
        if !status.is_success() {
            let detail = serde_json::from_slice::<serde_json::Value>(&bytes)
                .ok()
                .and_then(|v| v["detail"].as_str().map(str::to_owned))
                .unwrap_or_else(|| format!("SCIM request failed ({status})"));
            return Err(Error::Usage(detail));
        }
        Ok(Some(serde_json::from_slice(&bytes)?))
    }

    /// Principals of `kinds` whose names contain `query`, at most `limit` of
    /// each kind. `None` when the server has no directory.
    pub async fn search(
        &self,
        query: &str,
        kinds: &[&str],
        limit: usize,
    ) -> Result<Option<Vec<PrincipalRef>>> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(Some(Vec::new()));
        }
        let pages = futures::future::try_join_all(kinds.iter().map(|kind| async move {
            let mut url = self.url(resource_path(kind))?;
            url.query_pairs_mut()
                .append_pair("filter", &filter_for(kind, query))
                .append_pair("count", &limit.to_string());
            let page: Option<ListResponse> = self.get(url).await?;
            Ok::<_, Error>(page.map(|page| {
                page.resources
                    .into_iter()
                    .filter_map(|r| to_ref(kind, r))
                    .take(limit)
                    .collect::<Vec<_>>()
            }))
        }))
        .await?;
        // Any kind answering 404 means the API isn't there at all.
        if pages.iter().any(Option::is_none) {
            return Ok(None);
        }
        Ok(Some(pages.into_iter().flatten().flatten().collect()))
    }

    /// The signed-in principal (`Me`). `None` when the server has no
    /// directory.
    pub async fn me(&self) -> Result<Option<PrincipalRef>> {
        let me: Option<Resource> = self.get(self.url("Me")?).await?;
        Ok(me.and_then(|r| {
            if r.user_name.as_deref().is_some_and(|n| !n.is_empty()) {
                to_ref("user", r)
            } else {
                to_ref("service_account", r)
            }
        }))
    }
}

/// The candidate a principal hint resolves to: an exact match on name or
/// display name (ignoring case), else the only match.
pub fn resolve_hint<'a>(hint: &str, found: &'a [PrincipalRef]) -> Option<&'a PrincipalRef> {
    let hint = hint.trim();
    let exact = |p: &&PrincipalRef| {
        p.name.eq_ignore_ascii_case(hint)
            || p.display_name
                .as_deref()
                .is_some_and(|d| d.eq_ignore_ascii_case(hint))
    };
    let mut exact_matches = found.iter().filter(exact);
    match (exact_matches.next(), exact_matches.next()) {
        (Some(only), None) => Some(only),
        (Some(_), Some(_)) => None,
        (None, _) if found.len() == 1 => found.first(),
        (None, _) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literals_escape_quotes_and_backslashes() {
        assert_eq!(literal(r#"a"b\c"#), r#""a\"b\\c""#);
    }

    #[test]
    fn users_match_on_user_name_or_display_name() {
        assert_eq!(
            filter_for("user", "bo"),
            r#"userName co "bo" or displayName co "bo""#
        );
        assert_eq!(filter_for("group", "eng"), r#"displayName co "eng""#);
        assert_eq!(
            filter_for("service_account", "ci"),
            r#"displayName co "ci""#
        );
    }

    #[test]
    fn the_scim_base_keeps_the_server_mount_path() {
        let scim = Scim::new(
            CloudClient::new_unauthenticated(),
            &Url::parse("http://h:8080/uc/?x=1").unwrap(),
        );
        assert_eq!(
            scim.url("Users").unwrap().as_str(),
            "http://h:8080/uc/api/2.0/preview/scim/v2/Users"
        );
    }

    fn user(name: &str, display: Option<&str>) -> PrincipalRef {
        PrincipalRef {
            kind: "user".into(),
            name: name.into(),
            display_name: display.map(Into::into),
        }
    }

    #[test]
    fn a_hint_resolves_to_an_exact_or_only_match() {
        let bob = user("bob@example.com", Some("Bob"));
        let bobby = user("bobby@example.com", None);
        let both = [bob.clone(), bobby.clone()];
        // Display name, any case.
        assert_eq!(resolve_hint("bob", &both), Some(&bob));
        assert_eq!(resolve_hint("BOBBY@example.com", &both), Some(&bobby));
        // Ambiguous: two substring matches, no exact one.
        assert_eq!(resolve_hint("bo", &both), None);
        // A single match needn't be exact.
        assert_eq!(resolve_hint("bo", std::slice::from_ref(&bob)), Some(&bob));
        assert_eq!(resolve_hint("x", &[]), None);
    }
}
