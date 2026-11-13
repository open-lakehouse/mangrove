//! The Delta v1 API over HTTP against the real server stack, in-process.
//!
//! Every test starts its own [`TestServer`]: the service `uc-server serve`
//! builds, over in-memory SQLite and a temporary `file://` managed root. Unlike
//! `crates/delta-api/tests` (the contract against an in-memory fake backend),
//! this covers the server's `DeltaBackend` adapter: catalog lookups, staging
//! reservations in the resource store, the commit coordinator and `file://`
//! credential vending.
//!
//! The wire assertions follow the docs-factory Delta API semantics script
//! (protocol negotiation, error shapes, staging, etag conflicts).

use reqwest::StatusCode;
use rstest::rstest;
use serde_json::{Value, json};
use unitycatalog_acceptance::JourneyContext;
use unitycatalog_acceptance::checks::managed_delta::create_commit_read;
use unitycatalog_acceptance::conformance::{Target, extended_checks, run};
use unitycatalog_acceptance::server::TestServer;

const CATALOG: &str = "unity";
const SCHEMA: &str = "default";

/// A started server with catalog `unity` and schema `default`.
async fn server() -> (TestServer, JourneyContext) {
    let server = TestServer::start().await.expect("start server");
    let ctx = server.context().expect("context");
    ctx.create_catalog(CATALOG).await.expect("create catalog");
    ctx.client()
        .create_schema(SCHEMA, CATALOG)
        .await
        .expect("create schema");
    (server, ctx)
}

fn delta_url(server: &TestServer, path: &str) -> String {
    format!("{}/api/2.1/unity-catalog/delta/v1{path}", server.url())
}

fn table_path(table: &str) -> String {
    format!("/catalogs/{CATALOG}/schemas/{SCHEMA}/tables/{table}")
}

async fn get(server: &TestServer, path: &str) -> (StatusCode, Value) {
    let response = reqwest::get(delta_url(server, path)).await.unwrap();
    let status = response.status();
    (status, response.json().await.unwrap_or(Value::Null))
}

async fn post(server: &TestServer, path: &str, body: &Value) -> (StatusCode, Value) {
    let response = reqwest::Client::new()
        .post(delta_url(server, path))
        .json(body)
        .send()
        .await
        .unwrap();
    let status = response.status();
    (status, response.json().await.unwrap_or(Value::Null))
}

/// The Delta error `type` of an error response body.
fn error_type(body: &Value) -> &str {
    body["error"]["type"].as_str().unwrap_or_default()
}

/// Create a managed table through the staging flow, with one data commit.
async fn managed_table(ctx: &JourneyContext, table: &str) {
    let rows = create_commit_read(ctx.client(), CATALOG, SCHEMA, table)
        .await
        .expect("managed create + commit + read");
    assert_eq!(rows, 3);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_config_negotiates_the_protocol() {
    let (server, _ctx) = server().await;
    let (status, body) = get(
        &server,
        &format!("/config?catalog={CATALOG}&protocol-versions=1.0"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["protocol-version"], "1.0");
    let endpoints: Vec<&str> = body["endpoints"]
        .as_array()
        .expect("endpoints")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for endpoint in [
        "POST /v1/catalogs/{catalog}/schemas/{schema}/staging-tables",
        "POST /v1/catalogs/{catalog}/schemas/{schema}/tables",
        "GET /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}",
        "POST /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}",
    ] {
        assert!(
            endpoints.contains(&endpoint),
            "{endpoint} not in {endpoints:?}"
        );
    }
}

#[rstest]
#[case::missing_catalog("/config?protocol-versions=1.0", StatusCode::BAD_REQUEST, None)]
#[case::missing_protocol_versions("/config?catalog=unity", StatusCode::BAD_REQUEST, None)]
#[case::unsupported_version(
    "/config?catalog=unity&protocol-versions=9.9",
    StatusCode::BAD_REQUEST,
    None
)]
#[case::unknown_catalog(
    "/config?catalog=nope&protocol-versions=1.0",
    StatusCode::NOT_FOUND,
    Some("NoSuchCatalogException")
)]
#[tokio::test(flavor = "multi_thread")]
async fn get_config_rejects(
    #[case] path: &str,
    #[case] status: StatusCode,
    #[case] kind: Option<&str>,
) {
    let (server, _ctx) = server().await;
    let (actual, body) = get(&server, path).await;
    assert_eq!(actual, status, "{body}");
    if let Some(kind) = kind {
        assert_eq!(error_type(&body), kind, "{body}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn staging_reserves_a_managed_location_under_the_root() {
    let (server, _ctx) = server().await;
    let (status, body) = post(
        &server,
        &format!("/catalogs/{CATALOG}/schemas/{SCHEMA}/staging-tables"),
        &json!({ "name": "staged" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let table_id = body["table-id"].as_str().expect("table-id");
    let location = body["location"].as_str().expect("location");
    assert!(
        location.starts_with(&server.storage_root()),
        "{location} is not under {}",
        server.storage_root()
    );
    assert!(
        location.ends_with(table_id),
        "{location} does not end in {table_id}"
    );
    assert!(!location.ends_with('/'), "{location}");
    assert!(
        body["required-protocol"]["min-reader-version"].is_number(),
        "{body}"
    );
    assert!(body["required-properties"].is_object(), "{body}");

    // Staging alone doesn't create a table.
    let (status, _) = get(&server, &table_path("staged")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test(flavor = "multi_thread")]
async fn managed_create_requires_the_catalog_managed_contract() {
    let (server, _ctx) = server().await;
    let (_, staging) = post(
        &server,
        &format!("/catalogs/{CATALOG}/schemas/{SCHEMA}/staging-tables"),
        &json!({ "name": "bare" }),
    )
    .await;

    // A create without the required protocol features and properties.
    let (status, body) = post(
        &server,
        &format!("/catalogs/{CATALOG}/schemas/{SCHEMA}/tables"),
        &json!({
            "name": "bare",
            "location": staging["location"],
            "table-type": "MANAGED",
            "columns": { "type": "struct", "fields": [
                { "name": "id", "type": "long", "nullable": true, "metadata": {} }
            ] },
            "protocol": { "min-reader-version": 1, "min-writer-version": 2 },
            "properties": { "io.unitycatalog.tableId": staging["table-id"] },
            "last-commit-timestamp-ms": 0,
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    let (status, _) = get(&server, &table_path("bare")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test(flavor = "multi_thread")]
async fn managed_lifecycle_serves_the_ratified_commits() {
    let (server, ctx) = server().await;
    managed_table(&ctx, "events").await;

    let (status, body) = get(&server, &table_path("events")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["metadata"]["table-type"], "MANAGED");
    assert_eq!(body["latest-table-version"], 1, "{body}");
    let commits = body["commits"].as_array().expect("commits");
    assert_eq!(commits.len(), 1, "{body}");
    assert_eq!(commits[0]["version"], 1);

    // The seeder echoes the location back with a trailing slash; the table keeps
    // the reservation's spelling, which engines join `/_delta_log` onto.
    let location = body["metadata"]["location"].as_str().expect("location");
    assert!(!location.ends_with('/'), "{location}");
    assert!(location.starts_with(&server.storage_root()), "{location}");

    // tableExists answers 204, not 200 (see openapi/delta.yaml).
    let status = reqwest::Client::new()
        .head(delta_url(&server, &table_path("events")))
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_with_a_stale_etag_conflicts() {
    let (server, ctx) = server().await;
    managed_table(&ctx, "events").await;
    let (_, loaded) = get(&server, &table_path("events")).await;
    let uuid = loaded["metadata"]["table-uuid"].clone();
    let etag = loaded["metadata"]["etag"].clone();
    assert!(etag.is_string(), "{loaded}");

    let set_comment = |etag: Value, comment: &str| {
        json!({
            "requirements": [
                { "type": "assert-table-uuid", "uuid": uuid },
                { "type": "assert-etag", "etag": etag },
            ],
            "updates": [ { "action": "set-table-comment", "comment": comment } ],
        })
    };

    // The current etag wins and moves the etag on ...
    let (status, body) = post(
        &server,
        &table_path("events"),
        &set_comment(etag.clone(), "one"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_ne!(body["metadata"]["etag"], etag, "{body}");

    // ... so a writer still holding the old one conflicts.
    let (status, body) = post(&server, &table_path("events"), &set_comment(etag, "two")).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(
        error_type(&body),
        "UpdateRequirementConflictException",
        "{body}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn local_tables_vend_no_secrets() {
    let (server, ctx) = server().await;
    managed_table(&ctx, "events").await;
    for operation in ["READ", "READ_WRITE"] {
        let (status, body) = get(
            &server,
            &format!("{}/credentials?operation={operation}", table_path("events")),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{operation}: {body}");
        let creds = body["storage-credentials"]
            .as_array()
            .expect("storage-credentials");
        for cred in creds {
            assert!(
                cred["config"].as_object().is_none_or(|c| c.is_empty()),
                "{operation}: file:// vended secrets: {body}"
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn unknown_tables_are_not_found() {
    let (server, _ctx) = server().await;
    let (status, body) = get(&server, &table_path("missing")).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(error_type(&body), "NoSuchTableException", "{body}");
}

/// The conformance battery `integration_oss_rust` runs in CI, here in-process,
/// so it also runs locally without booting a server.
#[tokio::test(flavor = "multi_thread")]
async fn conformance_in_process() {
    let server = TestServer::start().await.expect("start server");
    let ctx = server.context().expect("context");
    let report = run(Target::OssRust, &ctx, extended_checks()).await;
    assert!(
        report.no_unexpected_failures(),
        "unexpected conformance failures: {:?}",
        report
            .unexpected_failures()
            .iter()
            .map(|c| c.name)
            .collect::<Vec<_>>()
    );
}
