//! Tests for the mounted, authenticated sharing surface: both protocol prefixes
//! behind the recipient bearer-token layer, response-format negotiation, and
//! temporary table credentials.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use open_sharing_api::auth::{Recipient, RecipientAuthenticator, require_recipient};
use open_sharing_api::testing::{FixtureObject, FixtureShare, in_memory_service};
use open_sharing_api::{Error, Result, SharingService, sharing_router};
use open_sharing_common::models::open_sharing::v1::SharingTemporaryCredentials;

const TOKEN: &str = "recipient-token";

struct OneToken;

#[async_trait]
impl RecipientAuthenticator for OneToken {
    async fn authenticate(&self, token: &str) -> Result<Recipient> {
        if token == TOKEN {
            Ok(Recipient {
                id: "r1".to_string(),
                name: "recipient one".to_string(),
            })
        } else {
            Err(Error::Unauthenticated)
        }
    }
}

fn app() -> axum::Router {
    let table = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/basic_partitioned")
        .canonicalize()
        .unwrap();
    let location = url::Url::from_directory_path(table).unwrap().to_string();
    let service: SharingService<Recipient> = in_memory_service(vec![FixtureShare {
        name: "share1".to_string(),
        objects: vec![FixtureObject::table("schema1", "letters", &location)],
        ..Default::default()
    }]);
    require_recipient(
        sharing_router::<SharingService<Recipient>, Recipient>(service),
        Arc::new(OneToken),
    )
}

fn request(method: &str, uri: &str, token: Option<&str>) -> axum::http::request::Builder {
    let builder = Request::builder().method(method).uri(uri);
    match token {
        Some(token) => builder.header("authorization", format!("Bearer {token}")),
        None => builder,
    }
}

async fn status(req: Request<Body>) -> StatusCode {
    app().oneshot(req).await.unwrap().status()
}

#[tokio::test]
async fn requires_a_valid_bearer_token() {
    for prefix in ["/api/v1/delta-sharing", "/api/1.0/opensharing"] {
        let uri = format!("{prefix}/shares");
        assert_eq!(
            status(request("GET", &uri, None).body(Body::empty()).unwrap()).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            status(
                request("GET", &uri, Some("wrong"))
                    .body(Body::empty())
                    .unwrap()
            )
            .await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            status(
                request("GET", &uri, Some(TOKEN))
                    .body(Body::empty())
                    .unwrap()
            )
            .await,
            StatusCode::OK,
            "{uri}"
        );
    }
}

#[tokio::test]
async fn negotiates_the_response_format() {
    let uri = "/api/v1/delta-sharing/shares/share1/schemas/schema1/tables/letters/query";
    let query = |capabilities: Option<&str>| {
        let mut builder =
            request("POST", uri, Some(TOKEN)).header("content-type", "application/json");
        if let Some(capabilities) = capabilities {
            builder = builder.header("delta-sharing-capabilities", capabilities);
        }
        builder.body(Body::from("{}")).unwrap()
    };
    assert_eq!(status(query(None)).await, StatusCode::OK);
    assert_eq!(
        status(query(Some("responseformat=delta,parquet"))).await,
        StatusCode::OK
    );
    assert_eq!(
        status(query(Some("responseformat=delta"))).await,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn vends_table_credentials_on_the_open_sharing_surface() {
    let path = "/shares/share1/schemas/schema1/tables/letters/temporary-table-credentials";
    let resp = app()
        .oneshot(
            request("POST", &format!("/api/1.0/opensharing{path}"), Some(TOKEN))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let credentials: SharingTemporaryCredentials = serde_json::from_slice(&bytes).unwrap();
    assert!(credentials.credentials.is_some());
    assert!(credentials.url.unwrap().ends_with("basic_partitioned/"));

    // Credential vending is an Open Sharing extension, absent from Delta Sharing.
    assert_eq!(
        status(
            request("POST", &format!("/api/v1/delta-sharing{path}"), Some(TOKEN))
                .body(Body::empty())
                .unwrap()
        )
        .await,
        StatusCode::NOT_FOUND
    );
}
