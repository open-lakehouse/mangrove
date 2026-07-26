//! The client arm forwards the per-request end-user identity (the request
//! context) to the upstream Unity Catalog credential vend.
//!
//! A standalone `storage-proxy` sits behind the same reverse proxy as the
//! upstream UC. When a request carries a validated forwarded identity, the vend
//! must be attributed to that user rather than to the proxy's own service
//! principal — so the identity rides upstream under the configured
//! forwarded-user header. An anonymous request forwards nothing (and keeps the
//! proxy's own auth). This drives the real port method (`StorageProxyBackend::open`)
//! through a mock UC and asserts the header on the wire.

#![allow(clippy::result_large_err)]

use unitycatalog_storage_proxy::backend::{ProxyReq, ProxyVerb, Securable, StorageProxyBackend};
use unitycatalog_storage_proxy::{ForwardedIdentity, UnityFactoryProxyBackend};

/// A `temporary-path-credentials` response with a minimal, well-formed Azurite
/// SAS credential — enough for the client arm to build an emulator store with no
/// real cloud I/O.
fn azurite_credential_body() -> String {
    let expiration = 32_503_680_000_000_i64; // year 3000, in epoch millis.
    format!(
        r#"{{"expiration_time":{expiration},"url":"http://127.0.0.1:10000/devstoreaccount1/mycontainer/prefix/","azure_user_delegation_sas":{{"sas_token":"sv=2021-08-06&ss=b&srt=co&sp=rl&se=2999-01-01T00:00:00Z&sig=AAAA"}}}}"#
    )
}

/// A read request for a raw cloud path (goes straight to the vend, no metadata
/// lookup).
fn get_path_req() -> ProxyReq {
    ProxyReq {
        verb: ProxyVerb::Get,
        securable: Securable::Path {
            url: url::Url::parse("abfss://mycontainer@devstoreaccount1/prefix/").unwrap(),
        },
        key: "part-0.parquet".to_string(),
        range: None,
        if_match: None,
    }
}

#[tokio::test]
async fn open_forwards_identity_to_upstream_vend() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/api/2.1/unity-catalog/temporary-path-credentials")
        .match_header("x-forwarded-user", "alice")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(azurite_credential_body())
        .expect(1)
        .create_async()
        .await;

    let backend = UnityFactoryProxyBackend::connect_with_forwarded_header(
        format!("{}/api/2.1/unity-catalog/", server.url()),
        Some("proxy-token".to_string()),
        "x-forwarded-user",
    )
    .await
    .unwrap();

    let cx = ForwardedIdentity::user("alice");
    StorageProxyBackend::open(&backend, &get_path_req(), &cx)
        .await
        .expect("open should vend + build a store against the mock");

    mock.assert_async().await;
}

#[tokio::test]
async fn open_anonymous_forwards_no_identity() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/api/2.1/unity-catalog/temporary-path-credentials")
        .match_header("x-forwarded-user", mockito::Matcher::Missing)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(azurite_credential_body())
        .expect(1)
        .create_async()
        .await;

    let backend = UnityFactoryProxyBackend::connect_with_forwarded_header(
        format!("{}/api/2.1/unity-catalog/", server.url()),
        Some("proxy-token".to_string()),
        "x-forwarded-user",
    )
    .await
    .unwrap();

    let cx = ForwardedIdentity::anonymous();
    StorageProxyBackend::open(&backend, &get_path_req(), &cx)
        .await
        .expect("open should vend + build a store against the mock");

    mock.assert_async().await;
}
