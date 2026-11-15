//! What the tools do, apart from the MCP wire: each takes JSON arguments and
//! answers an [`Outcome`] (a text summary for the model, structured content,
//! A2UI messages for the app), which `server.rs` turns into a tool result.

use serde::Deserialize;
use serde_json::{Value, json};
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::models::grants::v1::{GrantAssignment, GrantChange};

use super::a2ui::{self, Assignment, GrantForm, PrincipalRef, Status};
use super::scim::{self, Scim};
use crate::error::Error;

/// The securable types the grants API (`/permissions/{type}/{name}`) takes.
pub const SECURABLE_TYPES: [&str; 9] = [
    "metastore",
    "catalog",
    "schema",
    "table",
    "function",
    "volume",
    "registered_model",
    "external_location",
    "credential",
];

/// How many principals a search returns per kind, by default and at most.
const DEFAULT_SEARCH_LIMIT: usize = 8;
const MAX_SEARCH_LIMIT: usize = 50;

/// A tool's answer.
#[derive(Debug, Default)]
pub struct Outcome {
    /// For the model (and hosts without the app).
    pub text: String,
    /// `structuredContent`.
    pub structured: Option<Value>,
    /// A2UI messages for the app, sent as an embedded resource.
    pub a2ui: Vec<Value>,
    /// The call failed.
    pub is_error: bool,
}

impl Outcome {
    fn ok(text: impl Into<String>, structured: Value) -> Self {
        Self {
            text: text.into(),
            structured: Some(structured),
            ..Default::default()
        }
    }

    /// A failure, with the CLI's `{error, kind}` contract as structured content.
    fn error(err: &Error) -> Self {
        let message = error_message(err);
        Self {
            structured: Some(json!({ "error": message, "kind": err.kind().as_str() })),
            text: message,
            is_error: true,
            ..Default::default()
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::error(&Error::Usage(message.into()))
    }
}

/// An error's message without the variant prefix (`Client error: …`).
fn error_message(err: &Error) -> String {
    match err {
        Error::Client(e) => e.to_string(),
        Error::Usage(m) => m.clone(),
        other => other.to_string(),
    }
}

/// The canonical (underscore, upper-case) name of a privilege in either
/// spelling the server accepts (`USE CATALOG`, `use_catalog`).
pub fn normalize_privilege(raw: &str) -> String {
    raw.split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .to_ascii_uppercase()
}

/// A privilege for people: `USE_CATALOG` is `USE CATALOG`.
fn display_privilege(raw: &str) -> String {
    normalize_privilege(raw).replace('_', " ")
}

fn assignments(raw: Vec<GrantAssignment>) -> Vec<Assignment> {
    let mut out: Vec<_> = raw
        .into_iter()
        .map(|a| {
            let mut privileges: Vec<_> = a
                .privileges
                .iter()
                .map(|p| normalize_privilege(p))
                .collect();
            privileges.sort();
            Assignment {
                principal: a.principal,
                privileges,
            }
        })
        .filter(|a| !a.privileges.is_empty())
        .collect();
    out.sort_by(|a, b| a.principal.cmp(&b.principal));
    out
}

fn describe_assignments(assignments: &[Assignment]) -> String {
    if assignments.is_empty() {
        return "No privileges are granted directly.".into();
    }
    assignments
        .iter()
        .map(|a| {
            let privileges: Vec<_> = a.privileges.iter().map(|p| display_privilege(p)).collect();
            format!("- {}: {}", a.principal, privileges.join(", "))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The arguments naming one securable.
#[derive(Debug, Deserialize)]
struct Securable {
    securable_type: String,
    full_name: String,
}

impl Securable {
    fn check(&self) -> Result<(), Box<Outcome>> {
        if !SECURABLE_TYPES.contains(&self.securable_type.as_str()) {
            return Err(Box::new(Outcome::usage(format!(
                "unknown securable_type `{}`; expected one of: {}",
                self.securable_type,
                SECURABLE_TYPES.join(", ")
            ))));
        }
        if self.full_name.trim().is_empty() {
            return Err(Box::new(Outcome::usage("full_name is empty")));
        }
        Ok(())
    }
}

fn parse<T: for<'de> Deserialize<'de>>(args: Value) -> Result<T, Box<Outcome>> {
    serde_json::from_value(args)
        .map_err(|e| Box::new(Outcome::usage(format!("invalid arguments: {e}"))))
}

/// The tools, over one server, as the user.
#[derive(Clone)]
pub struct Tools {
    uc: UnityCatalogClient,
    scim: Scim,
}

impl Tools {
    pub fn new(uc: UnityCatalogClient, scim: Scim) -> Self {
        Self { uc, scim }
    }

    async fn current_grants(&self, s: &Securable) -> Result<Vec<Assignment>, Box<Outcome>> {
        self.uc
            .get_grants(&s.securable_type, &s.full_name)
            .await
            .map(|r| assignments(r.privilege_assignments))
            .map_err(|e| Box::new(Outcome::error(&e.into())))
    }

    /// `get_grants`: the privileges granted directly on a securable.
    pub async fn get_grants(&self, args: Value) -> Outcome {
        let run = async {
            let s: Securable = parse(args)?;
            s.check()?;
            let assignments = self.current_grants(&s).await?;
            Ok::<_, Box<Outcome>>(Outcome::ok(
                format!(
                    "Grants on {} {}:\n{}",
                    s.securable_type,
                    s.full_name,
                    describe_assignments(&assignments)
                ),
                json!({ "privilege_assignments": assignments }),
            ))
        };
        run.await.unwrap_or_else(|o| *o)
    }

    /// `open_grant_form`: the grant form, prefilled. Changes nothing.
    pub async fn open_grant_form(&self, args: Value) -> Outcome {
        #[derive(Deserialize)]
        struct Args {
            #[serde(flatten)]
            securable: Securable,
            #[serde(default)]
            principal: Option<String>,
            #[serde(default)]
            privileges: Vec<String>,
        }
        let run = async {
            let args: Args = parse(args)?;
            let s = &args.securable;
            s.check()?;
            let assignments = self.current_grants(s).await?;
            let mut privileges: Vec<String> = Vec::new();
            for p in args.privileges.iter().map(|p| normalize_privilege(p)) {
                if !p.is_empty() && !privileges.contains(&p) {
                    privileges.push(p);
                }
            }

            let hint = args.principal.unwrap_or_default().trim().to_owned();
            let mut principal = hint.clone();
            let mut candidates = Vec::new();
            let mut note = String::new();
            if !hint.is_empty() {
                // No directory, or it failed: take the hint as typed.
                if let Ok(Some(found)) = self
                    .scim
                    .search(&hint, &scim::KINDS, DEFAULT_SEARCH_LIMIT)
                    .await
                {
                    match scim::resolve_hint(&hint, &found) {
                        Some(one) => principal = one.name.clone(),
                        None if found.is_empty() => {
                            note = format!(
                                " No principal in the directory matches `{hint}`; the server checks the name on submit."
                            );
                        }
                        None => {
                            let names: Vec<_> = found.iter().map(|p| p.name.as_str()).collect();
                            note = format!(
                                " `{hint}` matches several principals ({}); the form asks the user to pick one.",
                                names.join(", ")
                            );
                            candidates = found;
                        }
                    }
                }
            }

            let form = GrantForm {
                securable_type: s.securable_type.clone(),
                full_name: s.full_name.clone(),
                principal: principal.clone(),
                privileges: privileges.clone(),
                assignments: assignments.clone(),
                candidates: candidates.clone(),
            };
            let who = if principal.is_empty() {
                "a principal the user picks".to_owned()
            } else {
                principal.clone()
            };
            let what = if privileges.is_empty() {
                "privileges the user picks".to_owned()
            } else {
                privileges
                    .iter()
                    .map(|p| display_privilege(p))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            Ok::<_, Box<Outcome>>(Outcome {
                text: format!(
                    "Opened a grant form for {what} on {} {} to {who}.{note} Nothing is granted until the user reviews the form and submits it; the outcome is reported back.",
                    s.securable_type, s.full_name
                ),
                structured: Some(json!({
                    "securable_type": s.securable_type,
                    "full_name": s.full_name,
                    "principal": principal,
                    "privileges": privileges,
                    "candidates": candidates,
                    "privilege_assignments": assignments,
                    "status": "awaiting_user",
                })),
                a2ui: a2ui::open_grant_form(&form),
                is_error: false,
            })
        };
        run.await.unwrap_or_else(|o| *o)
    }

    /// `apply_grants`: the grant the user submitted from the form.
    pub async fn apply_grants(&self, args: Value) -> Outcome {
        #[derive(Deserialize)]
        struct Change {
            principal: String,
            #[serde(default)]
            add: Vec<String>,
            #[serde(default)]
            remove: Vec<String>,
        }
        #[derive(Deserialize)]
        struct Args {
            #[serde(flatten)]
            securable: Securable,
            changes: Vec<Change>,
        }
        let args: Args = match parse(args) {
            Ok(args) => args,
            Err(outcome) => return with_status(*outcome),
        };
        let s = &args.securable;
        if let Err(outcome) = s.check() {
            return with_status(*outcome);
        }
        let changes: Vec<GrantChange> = args
            .changes
            .into_iter()
            .map(|c| GrantChange {
                principal: c.principal.trim().to_owned(),
                add: c.add.iter().map(|p| normalize_privilege(p)).collect(),
                remove: c.remove.iter().map(|p| normalize_privilege(p)).collect(),
                ..Default::default()
            })
            .filter(|c| !c.add.is_empty() || !c.remove.is_empty())
            .collect();
        if changes.is_empty() || changes.iter().any(|c| c.principal.is_empty()) {
            return with_status(Outcome::usage(
                "changes must name a principal and at least one privilege",
            ));
        }
        let summary = changes
            .iter()
            .map(|c| {
                let mut parts = Vec::new();
                if !c.add.is_empty() {
                    let added: Vec<_> = c.add.iter().map(|p| display_privilege(p)).collect();
                    parts.push(format!("granted {} to {}", added.join(", "), c.principal));
                }
                if !c.remove.is_empty() {
                    let removed: Vec<_> = c.remove.iter().map(|p| display_privilege(p)).collect();
                    parts.push(format!(
                        "revoked {} from {}",
                        removed.join(", "),
                        c.principal
                    ));
                }
                parts.join(", ")
            })
            .collect::<Vec<_>>()
            .join("; ");

        match self
            .uc
            .update_grants(&s.securable_type, &s.full_name)
            .with_changes(changes)
            .await
        {
            Ok(response) => {
                let assignments = assignments(response.privilege_assignments);
                let message = format!(
                    "The user {summary} on {} {}.",
                    s.securable_type, s.full_name
                );
                Outcome {
                    text: message.clone(),
                    structured: Some(json!({ "privilege_assignments": assignments })),
                    a2ui: a2ui::grant_outcome(Status::ok(capitalize(&summary)), Some(&assignments)),
                    is_error: false,
                }
            }
            Err(e) => with_status(Outcome::error(&e.into())),
        }
    }

    /// `search_principals`: users, groups and service principals by name.
    pub async fn search_principals(&self, args: Value) -> Outcome {
        #[derive(Deserialize)]
        struct Args {
            query: String,
            #[serde(default)]
            kinds: Option<Vec<String>>,
            #[serde(default)]
            limit: Option<usize>,
        }
        let args: Args = match parse(args) {
            Ok(args) => args,
            Err(outcome) => return *outcome,
        };
        let kinds: Vec<&str> = match &args.kinds {
            None => scim::KINDS.to_vec(),
            Some(kinds) => {
                if let Some(bad) = kinds.iter().find(|k| !scim::KINDS.contains(&k.as_str())) {
                    return Outcome::usage(format!(
                        "unknown principal kind `{bad}`; expected one of: {}",
                        scim::KINDS.join(", ")
                    ));
                }
                kinds.iter().map(String::as_str).collect()
            }
        };
        let limit = args
            .limit
            .unwrap_or(DEFAULT_SEARCH_LIMIT)
            .clamp(1, MAX_SEARCH_LIMIT);
        match self.scim.search(&args.query, &kinds, limit).await {
            Ok(Some(found)) => Outcome::ok(
                describe_principals(&found),
                json!({
                    "available": true,
                    "principals": found,
                }),
            ),
            Ok(None) => Outcome::ok(
                "The server has no principal directory (SCIM); principals are checked by name when granting.",
                json!({ "available": false, "principals": [] }),
            ),
            Err(e) => Outcome::error(&e),
        }
    }

    /// `whoami`: the signed-in principal.
    pub async fn whoami(&self) -> Outcome {
        match self.scim.me().await {
            Ok(Some(me)) => Outcome::ok(
                format!("Signed in as {}", me.name),
                json!({ "available": true, "principal": me }),
            ),
            Ok(None) => Outcome::ok(
                "The server has no principal directory (SCIM).",
                json!({ "available": false, "principal": null }),
            ),
            Err(e) => Outcome::error(&e),
        }
    }
}

/// A failed `apply_grants` also tells the form, through `/status`.
fn with_status(mut outcome: Outcome) -> Outcome {
    outcome.a2ui = a2ui::grant_outcome(Status::error(outcome.text.clone()), None);
    outcome
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn describe_principals(found: &[PrincipalRef]) -> String {
    if found.is_empty() {
        return "No principals match.".into();
    }
    found
        .iter()
        .map(|p| match &p.display_name {
            Some(d) => format!("- {} {} ({d})", p.kind, p.name),
            None => format!("- {} {}", p.kind, p.name),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privileges_normalize_from_either_spelling() {
        assert_eq!(normalize_privilege(" use  catalog "), "USE_CATALOG");
        assert_eq!(normalize_privilege("SELECT"), "SELECT");
        assert_eq!(display_privilege("use_catalog"), "USE CATALOG");
    }

    #[test]
    fn assignments_are_canonical_sorted_and_non_empty() {
        let raw = vec![
            GrantAssignment {
                principal: "zed".into(),
                privileges: vec!["SELECT".into(), "USE SCHEMA".into()],
                ..Default::default()
            },
            GrantAssignment {
                principal: "empty".into(),
                ..Default::default()
            },
            GrantAssignment {
                principal: "amy".into(),
                privileges: vec!["MODIFY".into()],
                ..Default::default()
            },
        ];
        assert_eq!(
            assignments(raw),
            vec![
                Assignment {
                    principal: "amy".into(),
                    privileges: vec!["MODIFY".into()],
                },
                Assignment {
                    principal: "zed".into(),
                    privileges: vec!["SELECT".into(), "USE_SCHEMA".into()],
                },
            ]
        );
    }

    #[test]
    fn a_bad_securable_is_a_usage_error() {
        let bad = Securable {
            securable_type: "share".into(),
            full_name: "s".into(),
        };
        let outcome = bad.check().unwrap_err();
        assert!(outcome.is_error);
        assert_eq!(outcome.structured.unwrap()["kind"], "usage");
        let empty = Securable {
            securable_type: "table".into(),
            full_name: " ".into(),
        };
        assert!(empty.check().is_err());
    }
}
