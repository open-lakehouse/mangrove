//! `uc mcp serve` end to end: an in-process Unity Catalog server (SQLite in
//! memory, a static SCIM directory) and an MCP client talking to
//! [`UcMcpServer`] over an in-memory pipe, as an MCP Apps host would; and
//! the same over Streamable HTTP (`uc mcp serve --http`).

use rmcp::model::{
    CallToolRequestParams, CallToolResult, ClientCapabilities, ClientConfig, ContentBlock,
    ExtensionCapabilities, Implementation, ReadResourceRequestParams, ResourceContents, Role,
};
use rmcp::service::{RoleClient, RunningService};
use rmcp::{ClientHandler, ServiceExt};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_server::config::{
    Backend, Config, ConfigValue, LocalStorageConfig, SqliteBackendConfig,
};
use url::Url;

use rmcp::transport::StreamableHttpClientTransport;

use super::a2ui::{A2UI_MIME_TYPE, GRANT_SURFACE_ID};
use super::http;
use super::scim::Scim;
use super::server::{GRANTS_APP_URI, MCP_APP_MIME_TYPE, UI_EXTENSION, UcMcpServer, tool_result};
use super::tools::{Outcome, Tools};

/// A server on a random loopback port; stops with the runtime.
struct Uc {
    url: Url,
    _dir: tempfile::TempDir,
}

async fn start(directory: bool) -> Uc {
    let dir = tempfile::tempdir().unwrap();
    let root = std::fs::canonicalize(dir.path()).unwrap();
    let root_str = root.to_str().unwrap().to_owned();
    let mut config = Config {
        backend: Backend::Sqlite(SqliteBackendConfig {
            path: ConfigValue::Value(":memory:".into()),
        }),
        local_storage: LocalStorageConfig {
            allowed_roots: vec![root_str],
        },
        managed_storage_root: Some(Url::from_directory_path(&root).unwrap().to_string()),
        ..Default::default()
    };
    config.ui.serve = false;
    if directory {
        config.directory = Some(
            serde_json::from_value(json!({
                "users": {
                    "source": "static",
                    "users": [
                        { "user-name": "alice@example.com", "display-name": "Alice Liddell" },
                        { "user-name": "bob@example.com", "display-name": "Bob Builder" },
                        { "user-name": "bobby@example.com" }
                    ]
                }
            }))
            .unwrap(),
        );
    }
    let router = unitycatalog_server::embed::start(&config)
        .await
        .unwrap()
        .router;
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await });
    Uc {
        url: Url::parse(&format!("http://{addr}")).unwrap(),
        _dir: dir,
    }
}

impl Uc {
    fn client(&self) -> UnityCatalogClient {
        let api = self.url.join("api/2.1/unity-catalog").unwrap();
        UnityCatalogClient::new(olai_http::CloudClient::new_unauthenticated(), api)
    }

    fn tools(&self) -> Tools {
        Tools::new(
            self.client(),
            Scim::new(olai_http::CloudClient::new_unauthenticated(), &self.url),
        )
    }

    async fn catalog(&self, name: &str) {
        self.client().create_catalog(name).await.unwrap();
    }
}

/// A client's capabilities, with the MCP Apps extension when `apps`.
fn capabilities(apps: bool) -> ClientCapabilities {
    if !apps {
        return ClientCapabilities::default();
    }
    let mut extensions = ExtensionCapabilities::new();
    extensions.insert(
        UI_EXTENSION.into(),
        serde_json::from_value(json!({ "mimeTypes": [MCP_APP_MIME_TYPE] })).unwrap(),
    );
    ClientCapabilities::builder()
        .enable_extensions_with(extensions)
        .build()
}

fn client_config(apps: bool) -> ClientConfig {
    ClientConfig::new(capabilities(apps), Implementation::new("test-host", "0"))
}

/// An MCP client, declaring the MCP Apps extension when `apps`.
async fn connect(uc: &Uc, apps: bool) -> RunningService<RoleClient, impl ClientHandler> {
    let (server_io, client_io) = tokio::io::duplex(1 << 20);
    let server = UcMcpServer::new(uc.tools());
    tokio::spawn(async move {
        if let Ok(running) = server.serve(server_io).await {
            let _ = running.waiting().await;
        }
    });
    client_config(apps).serve(client_io).await.unwrap()
}

async fn call(
    client: &RunningService<RoleClient, impl ClientHandler>,
    name: &'static str,
    args: Value,
) -> CallToolResult {
    let Value::Object(args) = args else {
        panic!("arguments are an object")
    };
    client
        .call_tool(CallToolRequestParams::new(name).with_arguments(args))
        .await
        .unwrap()
}

fn text(result: &CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(ContentBlock::as_text)
        .map(|t| t.text.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The A2UI messages in `result`, checking they're marked for the user only.
fn a2ui(result: &CallToolResult) -> Vec<Value> {
    let mut messages = Vec::new();
    for block in &result.content {
        let ContentBlock::Resource(embedded) = block else {
            continue;
        };
        let ResourceContents::TextResourceContents {
            mime_type, text, ..
        } = &embedded.resource
        else {
            continue;
        };
        assert_eq!(mime_type.as_deref(), Some(A2UI_MIME_TYPE));
        let audience = embedded
            .annotations
            .as_ref()
            .and_then(|a| a.audience.clone());
        assert_eq!(audience, Some(vec![Role::User]));
        let parsed: Vec<Value> = serde_json::from_str(text).unwrap();
        messages.extend(parsed);
    }
    messages
}

/// The data model `open_grant_form` opened the surface with.
fn data_model(messages: &[Value]) -> Value {
    messages
        .iter()
        .find_map(|m| {
            let update = m.get("updateDataModel")?;
            (update["path"] == "/").then(|| update["value"].clone())
        })
        .expect("a root data model")
}

fn structured(result: &CallToolResult) -> &Value {
    result
        .structured_content
        .as_ref()
        .expect("structured content")
}

#[tokio::test]
async fn app_tools_are_listed_only_for_mcp_apps_hosts() {
    let uc = start(false).await;
    let names = |tools: Vec<rmcp::model::Tool>| {
        let mut names: Vec<_> = tools.into_iter().map(|t| t.name.to_string()).collect();
        names.sort();
        names
    };

    let plain = connect(&uc, false).await;
    assert_eq!(
        names(plain.list_all_tools().await.unwrap()),
        ["get_grants", "open_grant_form", "search_principals"]
    );
    // And can't be called there either: apply_grants is a write.
    let refused = plain
        .call_tool(CallToolRequestParams::new("apply_grants"))
        .await;
    assert!(refused.is_err());

    let host = connect(&uc, true).await;
    let tools = host.list_all_tools().await.unwrap();
    let open = tools.iter().find(|t| t.name == "open_grant_form").unwrap();
    assert_eq!(
        open.meta.as_ref().unwrap().0["ui"]["resourceUri"],
        GRANTS_APP_URI
    );
    let apply = tools.iter().find(|t| t.name == "apply_grants").unwrap();
    assert_eq!(
        apply.meta.as_ref().unwrap().0["ui"]["visibility"],
        json!(["app"])
    );
    assert_eq!(
        names(tools),
        [
            "apply_grants",
            "get_grants",
            "open_grant_form",
            "search_principals",
            "whoami"
        ]
    );
}

#[tokio::test]
async fn the_app_is_served_as_an_mcp_app_resource() {
    let uc = start(false).await;
    let host = connect(&uc, true).await;
    let resources = host.list_all_resources().await.unwrap();
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].uri, GRANTS_APP_URI);
    assert_eq!(resources[0].mime_type.as_deref(), Some(MCP_APP_MIME_TYPE));

    let read = host
        .read_resource(ReadResourceRequestParams::new(GRANTS_APP_URI))
        .await
        .unwrap();
    let ResourceContents::TextResourceContents {
        mime_type, text, ..
    } = &read.contents[0]
    else {
        panic!("text contents")
    };
    assert_eq!(mime_type.as_deref(), Some(MCP_APP_MIME_TYPE));
    assert!(text.contains("<div id=\"root\">"));

    assert!(
        host.read_resource(ReadResourceRequestParams::new("ui://uc/nope"))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn open_the_form_submit_it_and_see_the_grant() {
    let uc = start(true).await;
    uc.catalog("main").await;
    let host = connect(&uc, true).await;

    // The agent asks for "bob": bob@example.com and bobby@example.com both
    // contain it and neither is an exact match, so the form offers both.
    let opened = call(
        &host,
        "open_grant_form",
        json!({
            "securable_type": "catalog",
            "full_name": "main",
            "principal": "bob",
            "privileges": ["use catalog", "SELECT", "SELECT"]
        }),
    )
    .await;
    assert_ne!(opened.is_error, Some(true), "{}", text(&opened));
    assert_eq!(
        opened.meta.as_ref().unwrap().0["ui"]["resourceUri"],
        GRANTS_APP_URI
    );
    assert!(text(&opened).contains("Nothing is granted until the user"));
    let messages = a2ui(&opened);
    assert_eq!(messages[0]["createSurface"]["surfaceId"], GRANT_SURFACE_ID);
    let model = data_model(&messages);
    assert_eq!(
        model["securable"],
        json!({ "type": "catalog", "fullName": "main" })
    );
    assert_eq!(model["principal"], "bob");
    assert_eq!(model["privileges"], json!(["USE_CATALOG", "SELECT"]));
    let candidates: Vec<_> = model["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(candidates, ["bob@example.com", "bobby@example.com"]);
    assert!(text(&opened).contains("matches several principals"));
    // Opening the form granted nothing.
    let before = call(
        &host,
        "get_grants",
        json!({ "securable_type": "catalog", "full_name": "main" }),
    )
    .await;
    assert_eq!(structured(&before)["privilege_assignments"], json!([]));

    // The user picks bob@example.com and submits.
    let applied = call(
        &host,
        "apply_grants",
        json!({
            "securable_type": "catalog",
            "full_name": "main",
            "changes": [{ "principal": "bob@example.com", "add": ["USE_CATALOG", "SELECT"] }]
        }),
    )
    .await;
    assert_ne!(applied.is_error, Some(true), "{}", text(&applied));
    assert_eq!(
        text(&applied),
        "The user granted USE CATALOG, SELECT to bob@example.com on catalog main."
    );
    let outcome = a2ui(&applied);
    assert_eq!(outcome[0]["updateDataModel"]["path"], "/status");
    assert_eq!(outcome[0]["updateDataModel"]["value"]["state"], "ok");
    assert_eq!(outcome[1]["updateDataModel"]["path"], "/assignments");

    let after = call(
        &host,
        "get_grants",
        json!({ "securable_type": "catalog", "full_name": "main" }),
    )
    .await;
    let assignments = &structured(&after)["privilege_assignments"];
    assert_eq!(assignments[0]["principal"], "bob@example.com");
    let mut privileges: Vec<_> = assignments[0]["privileges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap().to_owned())
        .collect();
    privileges.sort();
    assert_eq!(privileges, ["SELECT", "USE_CATALOG"]);
}

#[tokio::test]
async fn an_exact_or_only_match_prefills_the_principal() {
    let uc = start(true).await;
    uc.catalog("main").await;
    let tools = uc.tools();
    for (hint, want) in [
        ("alice", "alice@example.com"),
        ("Bob Builder", "bob@example.com"),
        ("BOBBY@example.com", "bobby@example.com"),
    ] {
        let outcome = tools
            .open_grant_form(json!({
                "securable_type": "catalog", "full_name": "main", "principal": hint
            }))
            .await;
        assert!(!outcome.is_error, "{}", outcome.text);
        let model = data_model(&outcome.a2ui);
        assert_eq!(model["principal"], want, "hint {hint}");
        assert_eq!(model["candidates"], json!([]));
    }
}

#[tokio::test]
async fn with_an_empty_directory_hints_are_kept_as_typed() {
    let uc = start(false).await;
    uc.catalog("main").await;
    let tools = uc.tools();

    let outcome = tools
        .open_grant_form(json!({
            "securable_type": "catalog", "full_name": "main", "principal": "bob"
        }))
        .await;
    assert!(!outcome.is_error, "{}", outcome.text);
    assert_eq!(data_model(&outcome.a2ui)["principal"], "bob");
    assert!(
        outcome
            .text
            .contains("No principal in the directory matches")
    );

    let search = tools.search_principals(json!({ "query": "bob" })).await;
    assert_eq!(
        search.structured.unwrap(),
        json!({ "available": true, "principals": [] })
    );
}

/// A server built without its `scim` feature: every SCIM path is a 404.
#[tokio::test]
async fn without_a_scim_api_search_is_unavailable() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, axum::Router::new()).await });
    let scim = Scim::new(
        olai_http::CloudClient::new_unauthenticated(),
        &Url::parse(&format!("http://{addr}")).unwrap(),
    );
    assert_eq!(
        scim.search("bob", &super::scim::KINDS, 5).await.unwrap(),
        None
    );
    assert_eq!(scim.me().await.unwrap(), None);

    let uc = start(false).await;
    let tools = Tools::new(uc.client(), scim);
    let search = tools.search_principals(json!({ "query": "bob" })).await;
    assert!(!search.is_error);
    assert_eq!(
        search.structured.unwrap(),
        json!({ "available": false, "principals": [] })
    );
    assert_eq!(tools.whoami().await.structured.unwrap()["available"], false);

    uc.catalog("main").await;
    let outcome = tools
        .open_grant_form(json!({
            "securable_type": "catalog", "full_name": "main", "principal": "bob"
        }))
        .await;
    assert!(!outcome.is_error, "{}", outcome.text);
    assert_eq!(data_model(&outcome.a2ui)["principal"], "bob");
}

#[tokio::test]
async fn search_finds_users_by_name_or_display_name() {
    let uc = start(true).await;
    let tools = uc.tools();
    let found = tools
        .search_principals(json!({ "query": "liddell", "kinds": ["user"] }))
        .await;
    assert!(!found.is_error, "{}", found.text);
    assert_eq!(
        found.structured.unwrap(),
        json!({
            "available": true,
            "principals": [
                { "kind": "user", "name": "alice@example.com", "displayName": "Alice Liddell" }
            ]
        })
    );

    let bad = tools
        .search_principals(json!({ "query": "x", "kinds": ["robot"] }))
        .await;
    assert!(bad.is_error);
    assert_eq!(bad.structured.unwrap()["kind"], "usage");
}

#[tokio::test]
async fn errors_come_back_as_tool_errors() {
    let uc = start(false).await;
    let tools = uc.tools();

    let unknown_type = tools
        .open_grant_form(json!({ "securable_type": "share", "full_name": "s" }))
        .await;
    assert!(unknown_type.is_error);
    assert_eq!(unknown_type.structured.unwrap()["kind"], "usage");

    let missing = tools
        .open_grant_form(json!({ "securable_type": "catalog", "full_name": "nope" }))
        .await;
    assert!(missing.is_error, "{}", missing.text);
    assert_eq!(missing.structured.unwrap()["kind"], "not_found");
    assert!(missing.a2ui.is_empty());

    // A failed submit also tells the form.
    let failed = tools
        .apply_grants(json!({
            "securable_type": "catalog",
            "full_name": "nope",
            "changes": [{ "principal": "bob@example.com", "add": ["USE_CATALOG"] }]
        }))
        .await;
    assert!(failed.is_error);
    assert_eq!(failed.a2ui[0]["updateDataModel"]["value"]["state"], "error");

    let empty = tools
        .apply_grants(json!({
            "securable_type": "catalog", "full_name": "main", "changes": [{ "principal": "bob" }]
        }))
        .await;
    assert!(empty.is_error);
    assert_eq!(empty.structured.unwrap()["kind"], "usage");
    assert_eq!(empty.a2ui[0]["updateDataModel"]["value"]["state"], "error");
}

// Streamable HTTP (`uc mcp serve --http`).

/// `uc mcp serve --http` on a random loopback port, accepting `allowed_hosts`
/// (rmcp's loopback default when empty). Returns its base URL.
async fn serve_http(uc: &Uc, allowed_hosts: &[&str]) -> Url {
    let allowed: Vec<String> = allowed_hosts.iter().map(|h| h.to_string()).collect();
    let router = http::router(UcMcpServer::new(uc.tools()), "/mcp", http::config(&allowed));
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await });
    Url::parse(&format!("http://{addr}")).unwrap()
}

/// An MCP client over Streamable HTTP, declaring the MCP Apps extension when `apps`.
async fn connect_http(base: &Url, apps: bool) -> RunningService<RoleClient, impl ClientHandler> {
    let transport = StreamableHttpClientTransport::from_uri(base.join("mcp").unwrap().as_str());
    client_config(apps).serve(transport).await.unwrap()
}

fn tool_names(tools: Vec<rmcp::model::Tool>) -> Vec<String> {
    let mut names: Vec<_> = tools.into_iter().map(|t| t.name.to_string()).collect();
    names.sort();
    names
}

const PLAIN_TOOLS: [&str; 3] = ["get_grants", "open_grant_form", "search_principals"];
const APP_TOOLS: [&str; 5] = [
    "apply_grants",
    "get_grants",
    "open_grant_form",
    "search_principals",
    "whoami",
];

/// POSTs one JSON-RPC message to `/mcp`, optionally with a `Host` override.
/// Returns the status and the JSON-RPC response (from a JSON or an SSE body).
async fn post_mcp(
    base: &Url,
    host: Option<&str>,
    headers: &[(&str, &str)],
    body: Value,
) -> (reqwest::StatusCode, Option<Value>) {
    let mut request = reqwest::Client::new()
        .post(base.join("mcp").unwrap())
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body(body.to_string());
    if let Some(host) = host {
        request = request.header("Host", host);
    }
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let response = request.send().await.unwrap();
    let status = response.status();
    let text = response.text().await.unwrap();
    let message = serde_json::from_str(&text).ok().or_else(|| {
        // An SSE body: the response is the `data:` event carrying an `id`.
        text.lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .filter_map(|data| serde_json::from_str::<Value>(data.trim()).ok())
            .find(|message| message.get("id").is_some())
    });
    (status, message)
}

/// A stateless (2026-07-28) `tools/list`, with the client's capabilities in `_meta`.
async fn stateless_tool_names(base: &Url, apps: bool) -> Vec<String> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {
            "_meta": {
                "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                "io.modelcontextprotocol/clientInfo": { "name": "stateless-host", "version": "0" },
                "io.modelcontextprotocol/clientCapabilities": capabilities(apps)
            }
        }
    });
    let headers = [
        ("MCP-Protocol-Version", "2026-07-28"),
        ("Mcp-Method", "tools/list"),
    ];
    let (status, message) = post_mcp(base, None, &headers, body).await;
    assert_eq!(status, 200, "{message:?}");
    let message = message.expect("a JSON-RPC response");
    let mut names: Vec<String> = message["result"]["tools"]
        .as_array()
        .unwrap_or_else(|| panic!("tools in {message}"))
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_owned())
        .collect();
    names.sort();
    names
}

#[tokio::test]
async fn over_http_app_tools_are_gated_per_connection() {
    let uc = start(false).await;
    let base = serve_http(&uc, &[]).await;

    // Two clients on one server: each sees the tools its capabilities allow.
    let host = connect_http(&base, true).await;
    let agent = connect_http(&base, false).await;
    assert_eq!(tool_names(host.list_all_tools().await.unwrap()), APP_TOOLS);
    assert_eq!(
        tool_names(agent.list_all_tools().await.unwrap()),
        PLAIN_TOOLS
    );

    // apply_grants (a write) is refused to the client without the extension.
    let refused = agent
        .call_tool(CallToolRequestParams::new("apply_grants"))
        .await
        .unwrap_err();
    let rmcp::ServiceError::McpError(error) = refused else {
        panic!("an MCP error, got {refused:?}")
    };
    assert_eq!(error.code, rmcp::model::ErrorCode::INVALID_PARAMS);

    // The host may call the app tools on the same server (the anonymous test
    // server has no identity, so whoami answers with a tool error, not a refusal).
    assert!(
        host.call_tool(CallToolRequestParams::new("whoami"))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn over_http_a_stateless_request_is_gated_on_its_meta() {
    let uc = start(false).await;
    let base = serve_http(&uc, &[]).await;
    assert_eq!(stateless_tool_names(&base, true).await, APP_TOOLS);
    assert_eq!(stateless_tool_names(&base, false).await, PLAIN_TOOLS);
}

#[tokio::test]
async fn over_http_the_grant_form_keeps_its_structure() {
    let uc = start(true).await;
    uc.catalog("main").await;
    let base = serve_http(&uc, &[]).await;
    let host = connect_http(&base, true).await;

    let opened = call(
        &host,
        "open_grant_form",
        json!({
            "securable_type": "catalog",
            "full_name": "main",
            "principal": "alice",
            "privileges": ["USE_CATALOG"]
        }),
    )
    .await;
    assert!(!opened.is_error.unwrap_or(false), "{}", text(&opened));
    assert_eq!(
        opened.meta.as_ref().unwrap().0["ui"]["resourceUri"],
        GRANTS_APP_URI
    );
    assert!(structured(&opened).is_object());
    let messages = a2ui(&opened);
    assert!(!messages.is_empty(), "the embedded A2UI resource");
    assert!(messages.iter().any(|m| {
        m.get("createSurface")
            .is_some_and(|surface| surface["surfaceId"] == GRANT_SURFACE_ID)
    }));
}

// The A2UI resource goes only to MCP Apps clients (mangrove#371).

/// `open_grant_form` on catalog `main` for `bob`, who matches two principals.
async fn open_form(client: &RunningService<RoleClient, impl ClientHandler>) -> CallToolResult {
    let opened = call(
        client,
        "open_grant_form",
        json!({
            "securable_type": "catalog",
            "full_name": "main",
            "principal": "bob",
            "privileges": ["SELECT"]
        }),
    )
    .await;
    assert_ne!(opened.is_error, Some(true), "{}", text(&opened));
    opened
}

fn resources(result: &CallToolResult) -> usize {
    result
        .content
        .iter()
        .filter(|block| matches!(block, ContentBlock::Resource(_)))
        .count()
}

/// The host's result has the A2UI resource and `_meta.ui`, the agent's has
/// neither, and both carry the same text and structured content.
fn assert_gated(host: &CallToolResult, agent: &CallToolResult) {
    assert_eq!(resources(host), 1);
    assert!(!a2ui(host).is_empty());
    assert_eq!(
        host.meta.as_ref().unwrap().0["ui"]["resourceUri"],
        GRANTS_APP_URI
    );

    assert_eq!(resources(agent), 0, "{:?}", agent.content);
    assert_eq!(agent.content.len(), 1, "only the text");
    assert!(agent.meta.is_none(), "{:?}", agent.meta);

    assert_eq!(host.structured_content, agent.structured_content);
    let structured = structured(agent);
    assert_eq!(structured["status"], "awaiting_user");
    assert_eq!(
        structured["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["bob@example.com", "bobby@example.com"]
    );
    assert_eq!(text(host), text(agent));
    let text = text(agent);
    assert!(text.contains("Opened a grant form"), "{text}");
    assert!(text.contains("Nothing is granted until the user"), "{text}");
    assert!(
        text.contains("bob@example.com, bobby@example.com"),
        "{text}"
    );
}

#[tokio::test]
async fn open_grant_form_sends_a2ui_only_to_mcp_apps_hosts() {
    let uc = start(true).await;
    uc.catalog("main").await;
    let host = open_form(&connect(&uc, true).await).await;
    let agent = open_form(&connect(&uc, false).await).await;
    assert_gated(&host, &agent);
}

#[tokio::test]
async fn over_http_open_grant_form_sends_a2ui_only_to_mcp_apps_hosts() {
    let uc = start(true).await;
    uc.catalog("main").await;
    let base = serve_http(&uc, &[]).await;
    // One server, both clients.
    let host = open_form(&connect_http(&base, true).await).await;
    let agent = open_form(&connect_http(&base, false).await).await;
    assert_gated(&host, &agent);
}

/// A stateless (2026-07-28) `tools/call` of `open_grant_form`, with the
/// client's capabilities in `_meta`. Returns the JSON-RPC result.
async fn stateless_open_form(base: &Url, apps: bool) -> Value {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "open_grant_form",
            "arguments": { "securable_type": "catalog", "full_name": "main", "principal": "bob" },
            "_meta": {
                "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                "io.modelcontextprotocol/clientInfo": { "name": "stateless-host", "version": "0" },
                "io.modelcontextprotocol/clientCapabilities": capabilities(apps)
            }
        }
    });
    let headers = [
        ("MCP-Protocol-Version", "2026-07-28"),
        ("Mcp-Method", "tools/call"),
        ("Mcp-Name", "open_grant_form"),
    ];
    let (status, message) = post_mcp(base, None, &headers, body).await;
    assert_eq!(status, 200, "{message:?}");
    let message = message.expect("a JSON-RPC response");
    message
        .get("result")
        .unwrap_or_else(|| panic!("a result in {message}"))
        .clone()
}

#[tokio::test]
async fn over_http_a_stateless_call_is_gated_on_its_meta() {
    let uc = start(true).await;
    uc.catalog("main").await;
    let base = serve_http(&uc, &[]).await;
    let host = stateless_open_form(&base, true).await;
    let agent = stateless_open_form(&base, false).await;

    let kinds = |result: &Value| -> Vec<String> {
        result["content"]
            .as_array()
            .unwrap()
            .iter()
            .map(|block| block["type"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(kinds(&host), ["text", "resource"]);
    assert_eq!(host["_meta"]["ui"]["resourceUri"], GRANTS_APP_URI);
    assert_eq!(kinds(&agent), ["text"]);
    assert!(agent.get("_meta").is_none(), "{agent}");
    assert_eq!(host["structuredContent"], agent["structuredContent"]);
    assert_eq!(agent["structuredContent"]["status"], "awaiting_user");
    assert_eq!(host["content"][0], agent["content"][0]);
}

/// The rule for every tool, `apply_grants`' outcome included: A2UI messages
/// only with the extension, and `_meta.ui` only for the tool that names the app.
#[test]
fn tool_results_carry_a2ui_only_for_mcp_apps_clients() {
    let outcome = || Outcome {
        text: "The user granted SELECT to bob on catalog main.".into(),
        structured: Some(json!({ "applied": true })),
        a2ui: vec![json!({ "updateDataModel": { "path": "/status", "value": { "state": "ok" } } })],
        is_error: false,
    };

    let agent = tool_result(outcome(), false, false);
    assert_eq!(resources(&agent), 0);
    assert!(agent.meta.is_none());
    assert_eq!(text(&agent), outcome().text);
    assert_eq!(agent.structured_content, outcome().structured);
    // Not even for the tool that names the app.
    let agent = tool_result(outcome(), false, true);
    assert_eq!(resources(&agent), 0);
    assert!(agent.meta.is_none());

    let app = tool_result(outcome(), true, false);
    assert_eq!(a2ui(&app), outcome().a2ui);
    assert!(app.meta.is_none());
    let app = tool_result(outcome(), true, true);
    assert_eq!(resources(&app), 1);
    assert_eq!(
        app.meta.as_ref().unwrap().0["ui"]["resourceUri"],
        GRANTS_APP_URI
    );
}

#[tokio::test]
async fn over_http_health_is_ok() {
    let uc = start(false).await;
    let base = serve_http(&uc, &[]).await;
    let response = reqwest::get(base.join("health").unwrap()).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "OK");
}

#[tokio::test]
async fn over_http_a_foreign_host_is_rejected_unless_allowed() {
    let uc = start(false).await;
    let initialize = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "0" }
        }
    });

    let loopback = serve_http(&uc, &[]).await;
    let (status, _) = post_mcp(&loopback, Some("evil.example"), &[], initialize.clone()).await;
    assert_eq!(status, 403);
    let (status, _) = post_mcp(&loopback, None, &[], initialize.clone()).await;
    assert_eq!(status, 200);

    let allowed = serve_http(&uc, &["uc-mcp"]).await;
    let (status, _) = post_mcp(&allowed, Some("uc-mcp:8090"), &[], initialize.clone()).await;
    assert_eq!(status, 200);
    let (status, _) = post_mcp(&allowed, Some("evil.example"), &[], initialize).await;
    assert_eq!(status, 403);
}
