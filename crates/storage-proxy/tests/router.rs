//! End-to-end tests for the storage-proxy router + streaming handler over an
//! in-memory backend, driven via `tower::ServiceExt::oneshot`.
//!
//! These exercise the full wire contract: ranged/full `GET`, `HEAD`,
//! conditional/unconditional `PUT`, the `If-Match` → 412 relay, and the
//! confused-deputy / read-only-scope rejections — all through real HTTP requests.

#![allow(clippy::result_large_err)]

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use bytes::Bytes;
use object_store::{ObjectStoreExt, PutPayload, memory::InMemory};
use tower::ServiceExt; // oneshot

use unitycatalog_storage_proxy::testing::InMemoryStorageProxyBackend;
use unitycatalog_storage_proxy::{ContextExtractor, router_with_context_at};

type Cx = ();

fn unit_extractor() -> ContextExtractor<Cx> {
    Arc::new(|_parts| Box::pin(async { Ok(()) }))
}

/// Build an app from a backend, mounting the proxy under `/storage-proxy`.
fn app(backend: InMemoryStorageProxyBackend) -> Router {
    let proxy: Router = router_with_context_at("", Arc::new(backend), unit_extractor());
    Router::new().nest("/storage-proxy", proxy)
}

/// Seed an object directly in a backend's store.
async fn seed(store: &InMemory, key: &str, bytes: &[u8]) -> String {
    let res = store
        .put(
            &object_store::path::Path::from(key),
            PutPayload::from(Bytes::copy_from_slice(bytes)),
        )
        .await
        .unwrap();
    res.e_tag.unwrap_or_default()
}

const SEC: &str = "table:main.default.events";

#[tokio::test]
async fn get_full_returns_200_with_length_and_etag() {
    let backend = InMemoryStorageProxyBackend::table("main.default.events");
    let store = backend.store();
    seed(&store, "data.bin", b"hello world").await;

    let resp = app(backend)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/storage-proxy/{SEC}/data.bin"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get(header::CONTENT_LENGTH).unwrap(), "11");
    assert_eq!(resp.headers().get(header::ACCEPT_RANGES).unwrap(), "bytes");
    assert!(resp.headers().get(header::ETAG).is_some());
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"hello world");
}

#[tokio::test]
async fn get_range_returns_206_with_content_range() {
    let backend = InMemoryStorageProxyBackend::table("main.default.events");
    let store = backend.store();
    seed(&store, "data.bin", b"0123456789").await;

    // bytes=2-5 -> 4 bytes "2345"
    let resp = app(backend)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/storage-proxy/{SEC}/data.bin"))
                .header(header::RANGE, "bytes=2-5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        resp.headers().get(header::CONTENT_RANGE).unwrap(),
        "bytes 2-5/10"
    );
    assert_eq!(resp.headers().get(header::CONTENT_LENGTH).unwrap(), "4");
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"2345");
}

#[tokio::test]
async fn get_suffix_range() {
    let backend = InMemoryStorageProxyBackend::table("main.default.events");
    seed(&backend.store(), "data.bin", b"0123456789").await;

    let resp = app(backend)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/storage-proxy/{SEC}/data.bin"))
                .header(header::RANGE, "bytes=-3")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        resp.headers().get(header::CONTENT_RANGE).unwrap(),
        "bytes 7-9/10"
    );
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"789");
}

#[tokio::test]
async fn head_returns_metadata_no_body() {
    let backend = InMemoryStorageProxyBackend::table("main.default.events");
    seed(&backend.store(), "data.bin", b"hello world").await;

    let resp = app(backend)
        .oneshot(
            Request::builder()
                .method("HEAD")
                .uri(format!("/storage-proxy/{SEC}/data.bin"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get(header::CONTENT_LENGTH).unwrap(), "11");
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert!(body.is_empty());
}

#[tokio::test]
async fn put_writes_and_relays_etag() {
    let backend = InMemoryStorageProxyBackend::table("main.default.events");
    let store = backend.store();

    let resp = app(backend)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/storage-proxy/{SEC}/out.bin"))
                .body(Body::from("payload"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().get(header::ETAG).is_some());
    // Round-trip: the object is now in the store.
    let got = store
        .get(&object_store::path::Path::from("out.bin"))
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    assert_eq!(&got[..], b"payload");
}

#[tokio::test]
async fn put_if_match_stale_returns_412_current_succeeds() {
    let backend = InMemoryStorageProxyBackend::table("main.default.events");
    let store = backend.store();
    let etag = seed(&store, "commit.json", b"v1").await;

    // Stale etag -> 412.
    let resp = app(backend.clone())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/storage-proxy/{SEC}/commit.json"))
                .header(header::IF_MATCH, "\"definitely-stale\"")
                .body(Body::from("v2"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED);

    // Current etag -> success.
    let resp = app(backend)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/storage-proxy/{SEC}/commit.json"))
                .header(header::IF_MATCH, etag)
                .body(Body::from("v2"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let got = store
        .get(&object_store::path::Path::from("commit.json"))
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    assert_eq!(&got[..], b"v2");
}

#[tokio::test]
async fn put_if_match_star_is_rejected_400() {
    let backend = InMemoryStorageProxyBackend::table("main.default.events");
    let resp = app(backend)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/storage-proxy/{SEC}/x.json"))
                .header(header::IF_MATCH, "*")
                .body(Body::from("x"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn out_of_scope_key_rejected_for_get_and_put() {
    // Backend restricts authorized keys to the `_delta_log/` prefix.
    let backend =
        InMemoryStorageProxyBackend::table("main.default.events").with_prefix("_delta_log/");

    // GET outside the prefix -> 403.
    let resp = app(backend.clone())
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/storage-proxy/{SEC}/data/secret.parquet"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // PUT outside the prefix -> 403.
    let resp = app(backend)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/storage-proxy/{SEC}/data/secret.parquet"))
                .body(Body::from("x"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn traversal_key_rejected_403() {
    let backend = InMemoryStorageProxyBackend::table("main.default.events");
    let resp = app(backend)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/storage-proxy/{SEC}/a/../../etc/passwd"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn read_only_scope_rejects_put_403() {
    let backend = InMemoryStorageProxyBackend::table("main.default.events").read_only();
    let resp = app(backend)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/storage-proxy/{SEC}/x.bin"))
                .body(Body::from("x"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn wrong_securable_rejected_403() {
    let backend = InMemoryStorageProxyBackend::table("main.default.events");
    let resp = app(backend)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/storage-proxy/table:main.default.other/x.bin")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn bad_securable_rejected_400() {
    let backend = InMemoryStorageProxyBackend::table("main.default.events");
    let resp = app(backend)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/storage-proxy/bogus:x/y.bin")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
