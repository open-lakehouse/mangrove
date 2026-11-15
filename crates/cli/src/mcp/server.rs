//! The MCP wire: tool and resource listings, and [`Outcome`]s as tool results.
//!
//! The grant form follows the MCP Apps extension (`io.modelcontextprotocol/ui`):
//! `open_grant_form` names the app (`_meta.ui.resourceUri`), the host renders
//! `ui://uc/grants` beside its result, and the app's own tools carry
//! `_meta.ui.visibility: ["app"]` so the host keeps them from the model. Those
//! app tools are only listed, and only callable, for a client that declared
//! the extension: a host without MCP Apps can't render the form, and must not
//! hand `apply_grants` (a write) to the model.

use std::sync::Arc;

use rmcp::model::{
    Annotations, CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock,
    EmbeddedResource, Implementation, InitializeResult, JsonObject, ListResourcesResult,
    ListToolsResult, MetaObject, PaginatedRequestParams, ReadResourceRequestParams,
    ReadResourceResponse, ReadResourceResult, Resource, ResourceContents, Role, ServerCapabilities,
    Tool, ToolAnnotations,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler};
use serde_json::{Value, json};

use super::a2ui;
use super::tools::{Outcome, SECURABLE_TYPES, Tools};

/// The MCP Apps extension id, as clients declare it in their capabilities.
pub const UI_EXTENSION: &str = "io.modelcontextprotocol/ui";

/// The MIME type of an MCP App resource.
pub const MCP_APP_MIME_TYPE: &str = "text/html;profile=mcp-app";

/// The grant form app.
pub const GRANTS_APP_URI: &str = "ui://uc/grants";

/// The app: `node/uc-a2ui`, built into one HTML file (`just build mcp-app`).
const GRANTS_APP_HTML: &str = include_str!("../../assets/mcp/grants-app.html");

const INSTRUCTIONS: &str = "\
Unity Catalog, as the signed-in user. To grant privileges, call `open_grant_form` \
with the securable and whatever the user asked for (a principal hint, privileges); \
it shows the user a prefilled form and grants nothing itself. The user reviews and \
submits it, and the outcome comes back as context: never tell the user a grant \
happened before that. Use `get_grants` to see what is granted directly on a \
securable, and `search_principals` to look up users, groups and service principals. \
Securable types: metastore, catalog, schema, table, function, volume, \
registered_model, external_location, credential. Privileges use the underscore \
spelling (USE_CATALOG, SELECT, MODIFY, ...).";

/// The `uc mcp serve` server.
#[derive(Clone)]
pub struct UcMcpServer {
    tools: Arc<Tools>,
}

impl UcMcpServer {
    pub fn new(tools: Tools) -> Self {
        Self {
            tools: Arc::new(tools),
        }
    }
}

fn schema(value: Value) -> Arc<JsonObject> {
    match value {
        Value::Object(map) => Arc::new(map),
        _ => unreachable!("tool schemas are objects"),
    }
}

fn meta(value: Value) -> MetaObject {
    match value {
        Value::Object(map) => MetaObject(map),
        _ => unreachable!("_meta is an object"),
    }
}

fn securable_properties() -> Value {
    json!({
        "securable_type": {
            "type": "string",
            "enum": SECURABLE_TYPES,
            "description": "The kind of securable."
        },
        "full_name": {
            "type": "string",
            "description": "Its full name: `catalog`, `catalog.schema`, `catalog.schema.table`; a location's or credential's name."
        }
    })
}

fn with(mut base: Value, extra: Value) -> Value {
    if let (Value::Object(base), Value::Object(extra)) = (&mut base, extra) {
        base.extend(extra);
    }
    base
}

/// The tools, for a client with (`apps`) or without the MCP Apps extension.
pub fn tools(apps: bool) -> Vec<Tool> {
    let securable = securable_properties();
    let read_only = ToolAnnotations::new().read_only(true).destructive(false);
    let mut tools = vec![
        Tool::new(
            "get_grants",
            "The privileges granted directly on a securable (inherited ones are not listed).",
            schema(json!({
                "type": "object",
                "properties": securable,
                "required": ["securable_type", "full_name"],
                "additionalProperties": false
            })),
        )
        .with_title("Get grants")
        .with_annotations(read_only.clone()),
        Tool::new(
            "open_grant_form",
            "Show the user a form to grant privileges on a securable, prefilled with what they asked \
             for. Grants nothing itself: the user reviews and submits it. A principal hint (`bob`, \
             `data eng`) is looked up in the directory.",
            schema(json!({
                "type": "object",
                "properties": with(securable.clone(), json!({
                    "principal": {
                        "type": "string",
                        "description": "Who to grant to: a user's email or name, a group, a service principal. A partial name is looked up."
                    },
                    "privileges": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "The privileges to prefill, e.g. [\"USE_CATALOG\", \"SELECT\"]."
                    }
                })),
                "required": ["securable_type", "full_name"],
                "additionalProperties": false
            })),
        )
        .with_title("Grant privileges")
        // Opening the form changes nothing.
        .with_annotations(read_only.clone())
        .with_meta(meta(json!({
            "ui": { "resourceUri": GRANTS_APP_URI, "visibility": ["model", "app"] }
        }))),
        Tool::new(
            "search_principals",
            "Users, groups and service principals whose name contains `query`.",
            schema(json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "kinds": {
                        "type": "array",
                        "items": { "type": "string", "enum": super::scim::KINDS }
                    },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 50 }
                },
                "required": ["query"],
                "additionalProperties": false
            })),
        )
        .with_title("Search principals")
        .with_annotations(read_only.clone()),
    ];
    if apps {
        tools.push(
            Tool::new(
                "apply_grants",
                "Apply the grant the user submitted from the grant form. Called by the form only.",
                schema(json!({
                    "type": "object",
                    "properties": with(securable.clone(), json!({
                        "changes": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "principal": { "type": "string" },
                                    "add": { "type": "array", "items": { "type": "string" } },
                                    "remove": { "type": "array", "items": { "type": "string" } }
                                },
                                "required": ["principal"]
                            }
                        }
                    })),
                    "required": ["securable_type", "full_name", "changes"],
                    "additionalProperties": false
                })),
            )
            .with_title("Apply grants")
            .with_annotations(ToolAnnotations::new().read_only(false).destructive(false))
            .with_meta(meta(json!({ "ui": { "visibility": ["app"] } }))),
        );
        tools.push(
            Tool::new(
                "whoami",
                "The signed-in principal. Called by the form only.",
                schema(
                    json!({ "type": "object", "properties": {}, "additionalProperties": false }),
                ),
            )
            .with_title("Who am I")
            .with_annotations(read_only)
            .with_meta(meta(json!({ "ui": { "visibility": ["app"] } }))),
        );
    }
    tools
}

/// `outcome` as a tool result. A2UI messages ride as one embedded resource,
/// for the user's eyes (the app) only, so their JSON stays out of the model's
/// context.
pub fn tool_result(outcome: Outcome, ui: bool) -> CallToolResult {
    let mut content = vec![ContentBlock::text(outcome.text)];
    if !outcome.a2ui.is_empty() {
        let payload = Value::Array(outcome.a2ui).to_string();
        content.push(ContentBlock::Resource(
            EmbeddedResource::new(ResourceContents::TextResourceContents {
                uri: a2ui::GRANT_FORM_RESOURCE_URI.into(),
                mime_type: Some(a2ui::A2UI_MIME_TYPE.into()),
                text: payload,
                meta: None,
            })
            .with_annotations(Annotations::default().with_audience(vec![Role::User])),
        ));
    }
    let mut result = if outcome.is_error {
        CallToolResult::error(content)
    } else {
        CallToolResult::success(content)
    };
    result.structured_content = outcome.structured;
    if ui {
        result.meta = Some(meta(json!({ "ui": { "resourceUri": GRANTS_APP_URI } })));
    }
    result
}

/// Whether the client declared the MCP Apps extension.
fn supports_apps(context: &RequestContext<RoleServer>) -> bool {
    context.peer.peer_info().is_some_and(|info| {
        info.capabilities
            .extensions
            .as_ref()
            .is_some_and(|extensions| extensions.contains_key(UI_EXTENSION))
    })
}

fn args(request: &CallToolRequestParams) -> Value {
    request
        .arguments
        .clone()
        .map(Value::Object)
        .unwrap_or_else(|| json!({}))
}

impl ServerHandler for UcMcpServer {
    fn get_info(&self) -> InitializeResult {
        InitializeResult::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(
            Implementation::new("uc", env!("CARGO_PKG_VERSION")).with_title("Unity Catalog"),
        )
        .with_instructions(INSTRUCTIONS)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(tools(supports_apps(
            &context,
        ))))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let apps = supports_apps(&context);
        let tools = &self.tools;
        let outcome = match request.name.as_ref() {
            "get_grants" => tools.get_grants(args(&request)).await,
            "open_grant_form" => {
                return Ok(tool_result(tools.open_grant_form(args(&request)).await, true).into());
            }
            "search_principals" => tools.search_principals(args(&request)).await,
            "apply_grants" if apps => tools.apply_grants(args(&request)).await,
            "whoami" if apps => tools.whoami().await,
            other => {
                return Err(McpError::invalid_params(
                    format!("unknown tool `{other}`"),
                    None,
                ));
            }
        };
        Ok(tool_result(outcome, false).into())
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult::with_all_items(vec![
            Resource::new(GRANTS_APP_URI, "grants")
                .with_title("Grant privileges")
                .with_description("The grant form, an A2UI surface (MCP App).")
                .with_mime_type(MCP_APP_MIME_TYPE),
        ]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        if request.uri != GRANTS_APP_URI {
            return Err(McpError::resource_not_found(
                format!("no resource `{}`", request.uri),
                None,
            ));
        }
        Ok(
            ReadResourceResult::new(vec![ResourceContents::TextResourceContents {
                uri: GRANTS_APP_URI.into(),
                mime_type: Some(MCP_APP_MIME_TYPE.into()),
                text: GRANTS_APP_HTML.into(),
                meta: None,
            }])
            .into(),
        )
    }
}
