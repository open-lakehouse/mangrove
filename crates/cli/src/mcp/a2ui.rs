//! The A2UI messages `uc mcp serve` sends: the grant form surface and its
//! updates.
//!
//! A2UI (<https://a2ui.org>) describes a UI as JSON: `createSurface` opens a
//! surface over a component catalog, `updateComponents` lays out components
//! whose properties bind to paths in the surface's data model, and
//! `updateDataModel` fills that model. The grant form is one component from
//! the Unity Catalog catalog, `UcGrantForm`, defined (with the MCP App that
//! renders it) in `node/uc-a2ui`. These constants mirror its
//! `src/protocol.ts`, and `node/uc-a2ui/fixtures/open_grant_form.json` is the
//! payload [`open_grant_form`] builds for a fixed input: a test here fails if
//! the fixture is stale, and a test there validates it against the catalogs.

use serde::Serialize;
use serde_json::{Value, json};

/// The A2UI protocol version the React renderer reads.
pub const A2UI_VERSION: &str = "v0.9";

/// The Unity Catalog component catalog.
pub const UC_CATALOG_ID: &str = "https://open-lakehouse.dev/a2ui/uc/v1/catalog.json";

/// The surface `open_grant_form` creates.
pub const GRANT_SURFACE_ID: &str = "uc-grant-form";

/// The MIME type of an embedded resource holding A2UI messages.
pub const A2UI_MIME_TYPE: &str = "application/a2ui+json";

/// The URI of the embedded resource carrying the grant form's messages.
pub const GRANT_FORM_RESOURCE_URI: &str = "a2ui://uc/grant-form";

/// The action the form's submit button fires; the app answers it by calling
/// the `apply_grants` tool.
pub const APPLY_GRANTS_EVENT: &str = "apply_grants";

/// A principal and the privileges granted to it directly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Assignment {
    pub principal: String,
    pub privileges: Vec<String>,
}

/// A principal from the directory: the host contract's `PrincipalRef`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct PrincipalRef {
    /// `user`, `group` or `service_account`.
    pub kind: String,
    /// The name services know it by: a user's email, a group's name.
    pub name: String,
    #[serde(
        rename = "displayName",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub display_name: Option<String>,
}

/// `/status` in the grant form's data model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Status {
    /// `idle`, `pending`, `ok` or `error`.
    pub state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl Status {
    pub fn idle() -> Self {
        Self {
            state: "idle",
            message: None,
        }
    }

    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            state: "ok",
            message: Some(message.into()),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            state: "error",
            message: Some(message.into()),
        }
    }
}

/// What the grant form opens with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantForm {
    pub securable_type: String,
    pub full_name: String,
    /// The principal to prefill (possibly just the agent's hint).
    pub principal: String,
    /// The privileges to prefill, canonical spelling.
    pub privileges: Vec<String>,
    /// The current direct grants.
    pub assignments: Vec<Assignment>,
    /// The principals an ambiguous hint matched.
    pub candidates: Vec<PrincipalRef>,
}

fn message(body: (&str, Value)) -> Value {
    let (kind, payload) = body;
    json!({ "version": A2UI_VERSION, kind: payload })
}

fn update_data_model(path: &str, value: Value) -> Value {
    message((
        "updateDataModel",
        json!({ "surfaceId": GRANT_SURFACE_ID, "path": path, "value": value }),
    ))
}

/// The messages that open the grant form: the surface, the `UcGrantForm`
/// bound to the data model, and the data model.
pub fn open_grant_form(form: &GrantForm) -> Vec<Value> {
    let bind = |path: &str| json!({ "path": path });
    vec![
        message((
            "createSurface",
            json!({ "surfaceId": GRANT_SURFACE_ID, "catalogId": UC_CATALOG_ID }),
        )),
        message((
            "updateComponents",
            json!({
                "surfaceId": GRANT_SURFACE_ID,
                "components": [{
                    "id": "root",
                    "component": "UcGrantForm",
                    "securableType": bind("/securable/type"),
                    "fullName": bind("/securable/fullName"),
                    "principal": bind("/principal"),
                    "privileges": bind("/privileges"),
                    "assignments": bind("/assignments"),
                    "candidates": bind("/candidates"),
                    "status": bind("/status"),
                    "submit": {
                        "event": {
                            "name": APPLY_GRANTS_EVENT,
                            "context": {
                                "securable_type": bind("/securable/type"),
                                "full_name": bind("/securable/fullName"),
                                "principal": bind("/principal"),
                                "privileges": bind("/privileges"),
                            }
                        }
                    }
                }]
            }),
        )),
        update_data_model(
            "/",
            json!({
                "securable": { "type": form.securable_type, "fullName": form.full_name },
                "principal": form.principal,
                "privileges": form.privileges,
                "assignments": form.assignments,
                "candidates": form.candidates,
                "status": Status::idle(),
            }),
        ),
    ]
}

/// The messages that report a grant's outcome: the status, and the grants
/// after it when the server answered with them.
pub fn grant_outcome(status: Status, assignments: Option<&[Assignment]>) -> Vec<Value> {
    let mut messages = vec![update_data_model("/status", json!(status))];
    if let Some(assignments) = assignments {
        messages.push(update_data_model("/assignments", json!(assignments)));
    }
    messages
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The input `node/uc-a2ui/fixtures/open_grant_form.json` was built from.
    fn fixture_form() -> GrantForm {
        GrantForm {
            securable_type: "table".into(),
            full_name: "main.sales.orders".into(),
            principal: "bob".into(),
            privileges: vec!["SELECT".into()],
            assignments: vec![Assignment {
                principal: "data-eng".into(),
                privileges: vec!["MODIFY".into(), "SELECT".into()],
            }],
            candidates: vec![
                PrincipalRef {
                    kind: "user".into(),
                    name: "bob@example.com".into(),
                    display_name: Some("Bob Builder".into()),
                },
                PrincipalRef {
                    kind: "user".into(),
                    name: "bobby@example.com".into(),
                    display_name: None,
                },
            ],
        }
    }

    const FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../node/uc-a2ui/fixtures/open_grant_form.json"
    );

    /// The app's catalog test validates the fixture; this keeps it what the
    /// server sends. `UPDATE_A2UI_FIXTURE=1` rewrites it.
    #[test]
    fn open_grant_form_matches_the_app_fixture() {
        let built = Value::Array(open_grant_form(&fixture_form()));
        if std::env::var_os("UPDATE_A2UI_FIXTURE").is_some() {
            let pretty = serde_json::to_string_pretty(&built).unwrap();
            std::fs::write(FIXTURE, format!("{pretty}\n")).unwrap();
        }
        let fixture: Value = serde_json::from_str(&std::fs::read_to_string(FIXTURE).unwrap())
            .expect("fixture is JSON");
        assert_eq!(
            built, fixture,
            "node/uc-a2ui/fixtures/open_grant_form.json is stale; rerun with UPDATE_A2UI_FIXTURE=1"
        );
    }

    #[test]
    fn every_message_names_the_version_and_surface() {
        let mut messages = open_grant_form(&fixture_form());
        messages.extend(grant_outcome(Status::ok("done"), Some(&[])));
        for m in &messages {
            assert_eq!(m["version"], A2UI_VERSION);
            let body = m.as_object().unwrap().iter().find(|(k, _)| *k != "version");
            let (_, body) = body.expect("one message body");
            assert_eq!(body["surfaceId"], GRANT_SURFACE_ID);
        }
    }

    #[test]
    fn grant_outcome_reports_status_then_assignments() {
        let assignments = [Assignment {
            principal: "bob".into(),
            privileges: vec!["SELECT".into()],
        }];
        let messages = grant_outcome(Status::ok("Granted"), Some(&assignments));
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["updateDataModel"]["path"], "/status");
        assert_eq!(
            messages[0]["updateDataModel"]["value"],
            json!({ "state": "ok", "message": "Granted" })
        );
        assert_eq!(messages[1]["updateDataModel"]["path"], "/assignments");

        let failed = grant_outcome(Status::error("nope"), None);
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0]["updateDataModel"]["value"]["state"], "error");
    }
}
