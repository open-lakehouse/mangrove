use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use olai_auth::grants::SubjectKind;
use serde_json::Value;
use tower::ServiceExt;

use super::*;
use crate::directory::{MemoryAuthz, StaticUsers};

fn user(id: &str, name: &str, display: Option<&str>) -> User {
    User {
        id: id.into(),
        user_name: name.into(),
        display_name: display.map(Into::into),
        emails: vec![name.into()],
        active: true,
    }
}

fn directory() -> Directory {
    Directory::new(
        Arc::new(StaticUsers::new(vec![
            user("u1", "alice@example.com", Some("Alice Liddell")),
            user("u2", "bob@example.com", None),
            User {
                active: false,
                ..user("u3", "carol@example.com", None)
            },
        ])),
        Arc::new(MemoryAuthz::new(
            vec![Group {
                id: "g1".into(),
                name: "analysts".into(),
                members: vec![
                    Member {
                        kind: SubjectKind::User,
                        id: "u1".into(),
                    },
                    Member {
                        kind: SubjectKind::ServiceAccount,
                        id: "s1".into(),
                    },
                    Member {
                        kind: SubjectKind::User,
                        id: "gone".into(),
                    },
                ],
            }],
            vec![ServicePrincipal {
                id: "s1".into(),
                name: "etl-nightly".into(),
                description: None,
                active: true,
            }],
        )),
    )
}

/// GET `path` (below the SCIM root) as `caller`.
async fn get_as(allow_anonymous: bool, caller: Option<&str>, path: &str) -> (StatusCode, Value) {
    let app = router(ScimState::new(directory(), allow_anonymous));
    let mut request = Request::get(format!("{SCIM_BASE_PATH}{path}"))
        .body(Body::empty())
        .unwrap();
    if let Some(name) = caller {
        request
            .extensions_mut()
            .insert(Principal::User(name.to_string()));
    }
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/scim+json"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&body).unwrap())
}

async fn get(path: &str) -> (StatusCode, Value) {
    get_as(false, Some("u1"), path).await
}

fn ids(list: &Value) -> Vec<&str> {
    list["Resources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect()
}

#[tokio::test]
async fn lists_users_with_their_groups() {
    let (status, list) = get("/Users").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["schemas"][0], LIST_SCHEMA);
    assert_eq!(list["totalResults"], 3);
    assert_eq!(ids(&list), ["u1", "u2", "u3"]);
    let alice = &list["Resources"][0];
    assert_eq!(alice["userName"], "alice@example.com");
    assert_eq!(alice["displayName"], "Alice Liddell");
    assert_eq!(alice["emails"][0]["value"], "alice@example.com");
    assert_eq!(alice["emails"][0]["primary"], true);
    assert_eq!(alice["groups"][0]["display"], "analysts");
    assert_eq!(alice["groups"][0]["$ref"], "Groups/g1");
    // No display name, no attribute.
    assert!(list["Resources"][1].get("displayName").is_none());
    assert_eq!(list["Resources"][2]["active"], false);
}

#[tokio::test]
async fn filters_users() {
    let (_, list) = get("/Users?filter=userName%20co%20%22ali%22").await;
    assert_eq!(ids(&list), ["u1"]);
    let (_, list) = get("/Users?filter=displayName%20sw%20%22alice%22").await;
    assert_eq!(ids(&list), ["u1"], "comparisons ignore case");
    let (_, list) = get("/Users?filter=active%20eq%20false").await;
    assert_eq!(ids(&list), ["u3"]);
    let (_, list) = get("/Users?filter=emails.value%20eq%20%22bob@example.com%22").await;
    assert_eq!(ids(&list), ["u2"]);
}

#[tokio::test]
async fn pages_users() {
    let (_, list) = get("/Users?startIndex=2&count=1").await;
    assert_eq!(list["totalResults"], 3);
    assert_eq!(list["startIndex"], 2);
    assert_eq!(list["itemsPerPage"], 1);
    assert_eq!(ids(&list), ["u2"]);
    // An index below 1 is 1.
    let (_, list) = get("/Users?startIndex=0&count=1").await;
    assert_eq!(ids(&list), ["u1"]);
}

#[tokio::test]
async fn an_invalid_filter_is_a_scim_error() {
    let (status, error) = get("/Users?filter=userName%20co").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error["schemas"][0], ERROR_SCHEMA);
    assert_eq!(error["scimType"], "invalidFilter");
    assert_eq!(error["status"], "400");
}

#[tokio::test]
async fn gets_one_user_or_404() {
    let (status, alice) = get("/Users/u1").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(alice["userName"], "alice@example.com");
    let (status, error) = get("/Users/nobody").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error["status"], "404");
}

#[tokio::test]
async fn lists_groups_with_member_names() {
    let (status, list) = get("/Groups?filter=displayName%20eq%20%22analysts%22").await;
    assert_eq!(status, StatusCode::OK);
    let members = &list["Resources"][0]["members"];
    assert_eq!(members[0]["display"], "alice@example.com");
    assert_eq!(members[0]["$ref"], "Users/u1");
    assert_eq!(members[1]["display"], "etl-nightly");
    assert_eq!(members[1]["$ref"], "ServicePrincipals/s1");
    // A member that is gone is shown by its id.
    assert_eq!(members[2]["display"], "gone");

    let (status, _) = get("/Groups/missing").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, group) = get("/Groups/g1").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(group["displayName"], "analysts");
}

#[tokio::test]
async fn lists_service_principals() {
    let (_, list) = get("/ServicePrincipals").await;
    let etl = &list["Resources"][0];
    assert_eq!(etl["schemas"][0], SERVICE_PRINCIPAL_SCHEMA);
    assert_eq!(etl["displayName"], "etl-nightly");
    assert_eq!(etl["applicationId"], "s1");
    assert_eq!(etl["groups"][0]["value"], "g1");
    let (status, _) = get("/ServicePrincipals/s2").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn me_is_the_caller() {
    // By id (Authelia's Remote-User), by name, and unknown.
    let (_, me) = get_as(false, Some("u1"), "/Me").await;
    assert_eq!(me["userName"], "alice@example.com");
    assert_eq!(me["groups"][0]["display"], "analysts");
    let (_, me) = get_as(false, Some("bob@example.com"), "/Me").await;
    assert_eq!(me["id"], "u2");
    let (status, me) = get_as(false, Some("dave@example.com"), "/Me").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["id"], "dave@example.com");
    assert_eq!(me["emails"][0]["value"], "dave@example.com");
}

#[tokio::test]
async fn anonymous_callers_are_turned_away() {
    let (status, _) = get_as(false, None, "/Users").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    // Without authentication anyone may read, but nobody is signed in.
    let (status, _) = get_as(true, None, "/Users").await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = get_as(true, None, "/Me").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[cfg(feature = "olai-auth")]
#[tokio::test]
async fn me_is_the_verified_caller() {
    let app = router(ScimState::new(directory(), false));
    let principal = olai_auth::Principal {
        kind: olai_auth::PrincipalKind::ServiceAccount {
            id: "s1".into(),
            name: "etl-nightly".into(),
        },
        issuer: olai_auth::API_KEY_ISSUER.into(),
        scopes: vec![],
        client_id: None,
        auth_method: olai_auth::AuthMethod::ApiKey,
        admin: false,
        issued_at: None,
        actor: None,
    };
    let mut request = Request::get(format!("{SCIM_BASE_PATH}/Me"))
        .body(Body::empty())
        .unwrap();
    request.extensions_mut().insert(principal);
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let me: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(me["schemas"][0], SERVICE_PRINCIPAL_SCHEMA);
    assert_eq!(me["groups"][0]["display"], "analysts");
}
