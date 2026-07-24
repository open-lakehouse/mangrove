//! End-to-end integration test for the storage byte-proxy **local arm** against a
//! live Azurite (Azure Blob emulator).
//!
//! Drives the proxy router over HTTP (`tower::ServiceExt::oneshot`) exactly as the
//! server mounts it, exercising the full local-arm path: authorize a `path:`
//! securable against a registered external location → vend a scoped SAS →
//! `store_from_vended_credential` → stream bytes. Proves ranged GET → 206, HEAD,
//! whole-object PUT round-trip, and the `If-Match` conditional-write relay, all
//! through real blob I/O.
//!
//! Gated behind `integration-storage-proxy-azurite` + `#[ignore]` so it never runs
//! in a normal `cargo test`. Needs a running Azurite on `localhost:10000`:
//!
//! ```sh
//! just integration-storage-proxy-azurite
//! ```
#![cfg(feature = "integration-storage-proxy-azurite")]

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use tower::ServiceExt;
use unitycatalog_common::models::credentials::v1::{
    AzureStorageKey, CreateCredentialRequest, Purpose,
};
use unitycatalog_common::models::external_locations::v1::CreateExternalLocationRequest;
use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};
use unitycatalog_server::api::{CredentialHandler, ExternalLocationHandler, RequestContext};
use unitycatalog_server::memory::InMemoryResourceStore;
use unitycatalog_server::policy::{ConstantPolicy, Policy, Principal};
use unitycatalog_server::rest::create_storage_proxy_router;
use unitycatalog_server::services::ServerHandler;

const ACCOUNT: &str = "devstoreaccount1";
const ACCOUNT_KEY: &str =
    "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==";

fn container() -> String {
    std::env::var("UC_AZURITE_CONTAINER").unwrap_or_else(|_| "lakehouse".to_string())
}

fn ctx() -> RequestContext {
    RequestContext {
        recipient: Principal::anonymous(),
    }
}

async fn seeded_handler() -> ServerHandler<RequestContext> {
    let encryptor =
        EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
    let store = Arc::new(InMemoryResourceStore::new(encryptor));
    let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
    let h = ServerHandler::try_new_tokio(policy, store).unwrap();

    h.create_credential(
        CreateCredentialRequest {
            name: "azurite_key".to_string(),
            purpose: Purpose::Storage.into(),
            azure_storage_key: Some(AzureStorageKey {
                account_name: ACCOUNT.to_string(),
                account_key: ACCOUNT_KEY.to_string(),
                ..Default::default()
            })
            .into(),
            skip_validation: Some(true),
            ..Default::default()
        },
        ctx(),
    )
    .await
    .unwrap();
    h.create_external_location(
        CreateExternalLocationRequest {
            name: "azurite_loc".to_string(),
            url: format!("azurite://{}", container()),
            credential_name: "azurite_key".to_string(),
            ..Default::default()
        },
        ctx(),
    )
    .await
    .unwrap();
    h
}

/// Mount the proxy router at root (the same way the server does).
fn app(h: ServerHandler<RequestContext>) -> Router {
    create_storage_proxy_router(h)
}

/// The `{securable}` segment for a `path:` securable pointing at the container
/// root, percent-encoded so the whole cloud URL stays in one path segment.
fn path_securable() -> String {
    let raw = format!("azurite://{}", container());
    // Encode the `:` and `/` in the URL so it does not split the path segment.
    let enc: String = raw
        .chars()
        .map(|c| match c {
            ':' => "%3A".to_string(),
            '/' => "%2F".to_string(),
            other => other.to_string(),
        })
        .collect();
    format!("path:{enc}")
}

#[tokio::test]
#[ignore = "requires a running Azurite emulator (just integration-storage-proxy-azurite)"]
async fn put_get_range_and_if_match_through_proxy() {
    let sec = path_securable();
    let key = "proxy/e2e/object.bin";
    let body = b"the quick brown fox jumps over the lazy dog";

    // PUT the object through the proxy.
    let resp = app(seeded_handler().await)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/storage-proxy/{sec}/{key}"))
                .body(Body::from(body.to_vec()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "PUT should succeed");
    let etag = resp
        .headers()
        .get(header::ETAG)
        .map(|v| v.to_str().unwrap().to_string());

    // Full GET returns the whole body.
    let resp = app(seeded_handler().await)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/storage-proxy/{sec}/{key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_LENGTH).unwrap(),
        &body.len().to_string()
    );
    let got = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&got[..], body);

    // Ranged GET returns a 206 with the requested slice.
    let resp = app(seeded_handler().await)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/storage-proxy/{sec}/{key}"))
                .header(header::RANGE, "bytes=4-8")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        resp.headers().get(header::CONTENT_RANGE).unwrap(),
        &format!("bytes 4-8/{}", body.len())
    );
    let slice = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&slice[..], &body[4..9]);

    // HEAD returns metadata, empty body.
    let resp = app(seeded_handler().await)
        .oneshot(
            Request::builder()
                .method("HEAD")
                .uri(format!("/storage-proxy/{sec}/{key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap()
            .is_empty()
    );

    // A conditional PUT with a stale If-Match is rejected 412 (if the store
    // returned an ETag to be stale against).
    if etag.is_some() {
        let resp = app(seeded_handler().await)
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/storage-proxy/{sec}/{key}"))
                    .header(header::IF_MATCH, "\"definitely-stale-etag\"")
                    .body(Body::from("updated"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::PRECONDITION_FAILED,
            "stale If-Match must be rejected 412"
        );
    }
}

/// An out-of-scope path securable (no covering external location) is rejected.
#[tokio::test]
#[ignore = "requires a running Azurite emulator (just integration-storage-proxy-azurite)"]
async fn unregistered_path_is_rejected() {
    // A different container that has no external location registered.
    let raw = "azurite://not-registered";
    let enc: String = raw
        .chars()
        .map(|c| match c {
            ':' => "%3A".to_string(),
            '/' => "%2F".to_string(),
            other => other.to_string(),
        })
        .collect();
    let resp = app(seeded_handler().await)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/storage-proxy/path:{enc}/some/key.bin"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::FORBIDDEN,
        "unregistered path must be denied, got {}",
        resp.status()
    );
}
