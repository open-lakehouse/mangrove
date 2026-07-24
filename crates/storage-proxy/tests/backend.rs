//! Backend-facing tests: the streaming paths (large ranged GET, multi-part PUT)
//! and the conditional-write relay, driven through the router over the in-memory
//! backend.
//!
//! The bounded-memory guarantee is *structural* — the handler streams `GET` via
//! `object_store`'s lazy `into_stream()` and pumps `PUT` through `WriteMultipart`,
//! so the whole object is never materialized. These tests exercise those paths at
//! sizes that span many chunks / multiple parts, proving they stream correctly end
//! to end (a size-independent code path, not a peak-RSS probe).

#![allow(clippy::result_large_err)]

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use bytes::Bytes;
use object_store::{ObjectStoreExt, PutPayload};
use tower::ServiceExt;

use unitycatalog_storage_proxy::testing::InMemoryStorageProxyBackend;
use unitycatalog_storage_proxy::{ContextExtractor, router_with_context_at};

type Cx = ();

fn app(backend: InMemoryStorageProxyBackend) -> Router {
    let extract_cx: ContextExtractor<Cx> = Arc::new(|_parts| Box::pin(async { Ok(()) }));
    let proxy: Router = router_with_context_at("", Arc::new(backend), extract_cx);
    Router::new().nest("/storage-proxy", proxy)
}

const SEC: &str = "vol:main.default.landing";

/// A deterministic byte pattern so we can verify slices without holding a second
/// copy in the test's expectations.
fn pattern(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 251) as u8).collect()
}

#[tokio::test]
async fn large_object_get_streams_full_and_ranged() {
    // 16 MiB spans many object_store stream chunks.
    let data = pattern(16 * 1024 * 1024);
    let backend = InMemoryStorageProxyBackend::volume("main.default.landing");
    backend
        .store()
        .put(
            &object_store::path::Path::from("big.bin"),
            PutPayload::from(Bytes::from(data.clone())),
        )
        .await
        .unwrap();

    // Full GET round-trips every byte.
    let resp = app(backend.clone())
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/storage-proxy/{SEC}/big.bin"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_LENGTH).unwrap(),
        &data.len().to_string()
    );
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.len(), data.len());
    assert_eq!(&body[..], &data[..]);

    // A mid-object range returns exactly that slice with a 206.
    let resp = app(backend)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/storage-proxy/{SEC}/big.bin"))
                .header(header::RANGE, "bytes=1048576-2097151")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::PARTIAL_CONTENT);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.len(), 1024 * 1024);
    assert_eq!(&body[..], &data[1048576..2097152]);
}

#[tokio::test]
async fn large_unconditional_put_streams_via_multipart() {
    // 12 MiB > the 5 MiB part size, so the multipart path flushes multiple parts.
    let data = pattern(12 * 1024 * 1024);
    let backend = InMemoryStorageProxyBackend::volume("main.default.landing");
    let store = backend.store();

    let resp = app(backend)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/storage-proxy/{SEC}/upload.bin"))
                .body(Body::from(data.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let got = store
        .get(&object_store::path::Path::from("upload.bin"))
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    assert_eq!(got.len(), data.len());
    assert_eq!(&got[..], &data[..]);
}

#[tokio::test]
async fn conditional_put_overwrites_existing() {
    let backend = InMemoryStorageProxyBackend::volume("main.default.landing");
    let store = backend.store();
    let put = store
        .put(
            &object_store::path::Path::from("commit.json"),
            PutPayload::from(Bytes::from_static(b"v1")),
        )
        .await
        .unwrap();
    let etag = put.e_tag.unwrap();

    let resp = app(backend)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/storage-proxy/{SEC}/commit.json"))
                .header(header::IF_MATCH, etag)
                .body(Body::from("v2-longer"))
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
    assert_eq!(&got[..], b"v2-longer");
}
