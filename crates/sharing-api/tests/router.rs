//! Router integration tests: serve the Open Sharing router over the in-memory
//! service and check discovery JSON + that the gap routes return 501.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use open_sharing_api::testing::{FixtureObject, FixtureShare, in_memory_service};
use open_sharing_api::{SharingService, open_sharing_router};
use tower::ServiceExt; // for `oneshot`

fn router() -> axum::Router {
    let service = in_memory_service(vec![FixtureShare {
        name: "share1".to_string(),
        id: Some("share1-id".to_string()),
        objects: vec![FixtureObject::table(
            "schema1",
            "table1",
            "s3://bucket/table1",
        )],
        ..Default::default()
    }]);
    open_sharing_router::<SharingService<()>, ()>(service)
}

async fn body_string(resp: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn list_shares_returns_json() {
    let resp = router()
        .oneshot(
            Request::builder()
                .uri("/shares")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    assert!(body.contains("share1"), "body was: {body}");
}

#[tokio::test]
async fn get_share_returns_json() {
    let resp = router()
        .oneshot(
            Request::builder()
                .uri("/shares/share1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    assert!(body.contains("share1"), "body was: {body}");
}

#[tokio::test]
async fn changes_route_is_not_implemented() {
    // The change-data-feed route exists (not 404) but returns 501.
    let resp = router()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/shares/share1/schemas/schema1/tables/table1/changes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn query_poll_route_is_not_implemented() {
    // The async-query poll route exists (not 404) but returns 501.
    let resp = router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/queries/q1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
}
