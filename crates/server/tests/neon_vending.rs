//! Trusted vending for Neon object storage, against a fake Neon control plane.
//!
//! A Neon credential is an `aws_iam_role` whose `role_arn` is a Neon ARN. With
//! `neon-trusted-vending` and vending enabled, table and path vends mint branch
//! keys through the control plane; otherwise they are a `FAILED_PRECONDITION`,
//! while registering still works.
#![cfg(all(feature = "memory", feature = "axum"))]

use std::sync::Arc;

#[path = "support/audit_capture.rs"]
mod audit_capture;

use axum::response::IntoResponse;
use unitycatalog_common::models::catalogs::v1::CreateCatalogRequest;
use unitycatalog_common::models::credentials::v1::{
    AwsIamRoleConfig, CreateCredentialRequest, Purpose,
};
use unitycatalog_common::models::external_locations::v1::CreateExternalLocationRequest;
use unitycatalog_common::models::schemas::v1::CreateSchemaRequest;
use unitycatalog_common::models::tables::v1::{CreateTableRequest, DataSourceFormat, TableType};
use unitycatalog_common::models::temporary_credentials::v1::*;
use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};
use unitycatalog_server::api::{
    CatalogHandler, CredentialHandler, ExternalLocationHandler, RequestContext, SchemaHandler,
    TableHandler, TemporaryCredentialHandler,
};
use unitycatalog_server::memory::InMemoryResourceStore;
use unitycatalog_server::policy::{ConstantPolicy, Policy};
use unitycatalog_server::services::{NeonVending, ServerHandler};
use unitycatalog_storage_access::ProvidesStorageAccessStore;

fn ctx() -> RequestContext {
    RequestContext::anonymous()
}

fn handler(neon: NeonVending) -> ServerHandler<RequestContext> {
    let encryptor =
        EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
    let store = Arc::new(InMemoryResourceStore::new(encryptor));
    let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
    ServerHandler::try_new_tokio(policy, store)
        .unwrap()
        .with_neon_vending(neon)
}

async fn register_credential(
    h: &ServerHandler<RequestContext>,
    role_arn: &str,
) -> unitycatalog_server::Result<()> {
    h.create_credential(
        CreateCredentialRequest {
            name: "neon".into(),
            purpose: Purpose::Storage.into(),
            aws_iam_role: Some(AwsIamRoleConfig {
                role_arn: role_arn.into(),
                ..Default::default()
            })
            .into(),
            ..Default::default()
        },
        ctx(),
    )
    .await
    .map(drop)
}

/// The Neon credential (ARN `role_arn`) governing `s3://bucket`, and the external
/// table `cat.sch.t` at `s3://bucket/cat/sch/t`. Returns the table id.
async fn seed(h: &ServerHandler<RequestContext>, role_arn: &str) -> String {
    register_credential(h, role_arn).await.unwrap();
    h.create_external_location(
        CreateExternalLocationRequest {
            name: "branch".into(),
            url: "s3://bucket".into(),
            credential_name: "neon".into(),
            ..Default::default()
        },
        ctx(),
    )
    .await
    .unwrap();
    h.create_catalog(
        CreateCatalogRequest {
            name: "cat".into(),
            storage_root: Some("s3://bucket/cat".into()),
            ..Default::default()
        },
        ctx(),
    )
    .await
    .unwrap();
    h.create_schema(
        CreateSchemaRequest {
            name: "sch".into(),
            catalog_name: "cat".into(),
            ..Default::default()
        },
        ctx(),
    )
    .await
    .unwrap();
    let table = h
        .create_table(
            CreateTableRequest {
                name: "t".into(),
                catalog_name: "cat".into(),
                schema_name: "sch".into(),
                table_type: TableType::External.into(),
                data_source_format: DataSourceFormat::Delta.into(),
                storage_location: Some("s3://bucket/cat/sch/t".into()),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
    table.table_id.unwrap()
}

fn table_vend(table_id: &str, read_write: bool) -> GenerateTemporaryTableCredentialsRequest {
    use generate_temporary_table_credentials_request::Operation;
    GenerateTemporaryTableCredentialsRequest {
        table_id: table_id.into(),
        operation: if read_write {
            Operation::ReadWrite
        } else {
            Operation::Read
        }
        .into(),
        ..Default::default()
    }
}

/// Asserts `err` is a 400 `FAILED_PRECONDITION` whose message contains `needle`.
async fn assert_failed_precondition(err: unitycatalog_server::Error, needle: &str) {
    assert_eq!(err.error_code(), "FAILED_PRECONDITION", "{err:?}");
    let response = err.into_response();
    assert_eq!(response.status(), http::StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["errorCode"], "FAILED_PRECONDITION");
    let message = body["message"].as_str().unwrap();
    assert!(message.contains(needle), "{message}");
}

/// Registering a Neon credential works whether or not vending is on, and the
/// stored ARN is completed from the server's defaults.
#[tokio::test]
async fn registration_completes_the_arn_from_the_defaults() {
    let h = handler(NeonVending::disabled(
        Some("proj".into()),
        Some("br-1".into()),
    ));
    register_credential(&h, "arn:neon:storage::").await.unwrap();
    let stored = h
        .storage_access()
        .credential_with_secrets("neon")
        .await
        .unwrap();
    assert_eq!(
        stored.aws_iam_role.as_option().unwrap().role_arn,
        "arn:neon:storage::proj:branch/br-1"
    );

    // Ids in the ARN win over the defaults.
    let h = handler(NeonVending::disabled(Some("proj".into()), None));
    register_credential(&h, "arn:neon:storage::own:branch/mine")
        .await
        .unwrap();
    let stored = h
        .storage_access()
        .credential_with_secrets("neon")
        .await
        .unwrap();
    assert_eq!(
        stored.aws_iam_role.as_option().unwrap().role_arn,
        "arn:neon:storage::own:branch/mine"
    );
}

#[tokio::test]
async fn registration_refuses_incomplete_or_keyed_neon_credentials() {
    let h = handler(NeonVending::disabled(Some("proj".into()), None));
    let err = register_credential(&h, "arn:neon:storage::")
        .await
        .unwrap_err();
    assert_eq!(err.error_code(), "INVALID_PARAMETER_VALUE", "{err:?}");
    assert!(
        format!("{err:?}").contains("vending.neon.branch-id"),
        "{err:?}"
    );

    let err = register_credential(&h, "arn:neon:storage::p:branch/b/x")
        .await
        .unwrap_err();
    assert_eq!(err.error_code(), "INVALID_PARAMETER_VALUE", "{err:?}");

    let err = h
        .create_credential(
            CreateCredentialRequest {
                name: "keyed".into(),
                purpose: Purpose::Storage.into(),
                aws_iam_role: Some(AwsIamRoleConfig {
                    role_arn: "arn:neon:storage::p:branch/b".into(),
                    access_key_id: Some("AKIA".into()),
                    secret_access_key: Some("secret".into()),
                    ..Default::default()
                })
                .into(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap_err();
    assert_eq!(err.error_code(), "INVALID_PARAMETER_VALUE", "{err:?}");
}

/// Disabled (the default), a vend against a Neon credential is a clear
/// `FAILED_PRECONDITION`, on UC REST and the Delta API.
#[tokio::test]
async fn disabled_vending_is_a_failed_precondition() {
    use unitycatalog_delta_api::backend::{CredentialAccess, DeltaBackend};

    let h = handler(NeonVending::default());
    let table_id = seed(&h, "arn:neon:storage::proj:branch/br-1").await;
    let needle = if cfg!(feature = "neon-trusted-vending") {
        "vending.neon.enabled: false"
    } else {
        "neon-trusted-vending"
    };

    let (captured, guard) = audit_capture::capture();
    let err = h
        .generate_temporary_table_credentials(table_vend(&table_id, false), ctx())
        .await
        .unwrap_err();
    drop(guard);
    assert_failed_precondition(err, needle).await;
    // A refused vend is audited too.
    let [event] = captured.events().try_into().unwrap();
    assert_eq!(event["outcome"], "error");
    assert_eq!(event["error_code"], "FAILED_PRECONDITION");
    assert_eq!(event["credential_kind"], "neon_storage");

    let err = h
        .generate_temporary_path_credentials(
            GenerateTemporaryPathCredentialsRequest {
                url: "s3://bucket/other".into(),
                operation: generate_temporary_path_credentials_request::Operation::PathRead.into(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap_err();
    assert_failed_precondition(err, needle).await;

    let err = DeltaBackend::vend_table_credential(&h, &table_id, CredentialAccess::Read, &ctx())
        .await
        .unwrap_err();
    assert!(
        matches!(
            &err,
            unitycatalog_delta_api::DeltaBackendError::BadRequest(m) if m.contains(needle)
        ),
        "{err:?}"
    );
}

#[cfg(feature = "neon-trusted-vending")]
mod enabled {
    use mockito::{Matcher, Mock, Server, ServerGuard};
    use olai_credential_vending::neon::NeonMinterOptions;
    use serde_json::json;
    use unitycatalog_delta_api::DeltaApiHandler;
    use unitycatalog_delta_api::backend::TableRef;
    use unitycatalog_delta_api::models::DeltaCredentialOperation;

    use super::*;

    const BRANCH: &str = "/projects/proj/branches/br-1";
    const ENDPOINT: &str = "https://br-1.storage.c-7.eu-central-1.aws.neon.tech";
    const EXPIRES: &str = "2030-01-01T00:00:00Z";
    const EXPIRES_MS: i64 = 1_893_456_000_000;

    fn enabled(server: &ServerGuard) -> NeonVending {
        let api_url = format!("{}/", server.url()).parse().unwrap();
        NeonVending::disabled(Some("proj".into()), Some("br-1".into())).enabled(
            "NEON-API-KEY",
            NeonMinterOptions::default().with_api_url(api_url),
        )
    }

    async fn storage(server: &mut ServerGuard) -> Mock {
        server
            .mock("GET", format!("{BRANCH}/storage").as_str())
            .match_header("authorization", "Bearer NEON-API-KEY")
            .with_body(
                json!({
                    "enabled": true,
                    "s3_endpoint": ENDPOINT,
                    "region": "eu-central-1",
                    "force_path_style": true,
                })
                .to_string(),
            )
            .create_async()
            .await
    }

    /// Creating a key with exactly `scopes`, answered with key `token_id`.
    async fn create(server: &mut ServerGuard, scopes: &[&str], token_id: &str) -> Mock {
        server
            .mock("POST", format!("{BRANCH}/credentials").as_str())
            .match_header("authorization", "Bearer NEON-API-KEY")
            .match_body(Matcher::PartialJson(json!({ "scopes": scopes })))
            .with_status(201)
            .with_body(
                json!({
                    "token_id": token_id,
                    "s3_secret_access_key": format!("SECRET-{token_id}"),
                    "scopes": scopes,
                    "branch_id": "br-1",
                    "expires_at": EXPIRES,
                })
                .to_string(),
            )
            .expect(1)
            .create_async()
            .await
    }

    /// Garbage collection after a mint lists the branch's keys.
    async fn list(server: &mut ServerGuard) -> Mock {
        server
            .mock("GET", format!("{BRANCH}/credentials").as_str())
            .with_body(json!({ "credentials": [] }).to_string())
            .create_async()
            .await
    }

    fn aws(credential: &TemporaryCredential) -> &AwsTemporaryCredentials {
        match credential.credentials.as_ref() {
            Some(temporary_credential::Credentials::AwsTempCredentials(aws)) => aws,
            other => panic!("expected AWS credentials, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn table_and_path_vends_mint_expiring_branch_keys() {
        let mut server = Server::new_async().await;
        let _storage = storage(&mut server).await;
        let _list = list(&mut server).await;
        let read = create(&mut server, &["storage:read"], "nak_read").await;
        let read_write = create(&mut server, &["storage:read", "storage:write"], "nak_write").await;

        // The ARN is completed from the defaults (`proj` / `br-1`).
        let h = handler(enabled(&server));
        let table_id = seed(&h, "arn:neon:storage::").await;

        let credential = h
            .generate_temporary_table_credentials(table_vend(&table_id, false), ctx())
            .await
            .unwrap();
        let key = aws(&credential);
        assert_eq!(key.access_key_id, "nak_read");
        assert_eq!(key.secret_access_key, "SECRET-nak_read");
        assert!(key.session_token.is_empty());
        assert_eq!(credential.expiration_time, EXPIRES_MS);

        let credential = h
            .generate_temporary_path_credentials(
                GenerateTemporaryPathCredentialsRequest {
                    url: "s3://bucket/landing".into(),
                    operation:
                        generate_temporary_path_credentials_request::Operation::PathReadWrite
                            .into(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(aws(&credential).access_key_id, "nak_write");
        assert_eq!(credential.url, "s3://bucket/landing");

        // A second read reuses the cached key: one create per access level.
        h.generate_temporary_table_credentials(table_vend(&table_id, false), ctx())
            .await
            .unwrap();
        read.assert_async().await;
        read_write.assert_async().await;
    }

    #[tokio::test]
    async fn delta_credentials_carry_the_branch_endpoint() {
        let mut server = Server::new_async().await;
        let _storage = storage(&mut server).await;
        let _list = list(&mut server).await;
        let _read = create(&mut server, &["storage:read"], "nak_read").await;

        let h = handler(enabled(&server));
        seed(&h, "arn:neon:storage::proj:branch/br-1").await;

        let response = DeltaApiHandler::get_table_credentials(
            &h,
            TableRef {
                catalog: "cat".into(),
                schema: "sch".into(),
                table: "t".into(),
            },
            DeltaCredentialOperation::Read,
            ctx(),
        )
        .await
        .unwrap();
        let [credential] = response.storage_credentials.as_slice() else {
            panic!("{response:?}");
        };
        let config = &credential.config;
        assert_eq!(config["s3.access-key-id"], "nak_read");
        assert_eq!(config["s3.endpoint"], ENDPOINT);
        assert_eq!(config["s3.region"], "eu-central-1");
        assert_eq!(config["s3.path-style-access"], "true");
        assert_eq!(credential.expiration_time_ms, Some(EXPIRES_MS));
    }

    /// Every vend emits exactly one audit event, without the key's secret.
    #[tokio::test]
    async fn every_vend_emits_one_audit_event_without_secrets() {
        let mut server = Server::new_async().await;
        let _storage = storage(&mut server).await;
        let _list = list(&mut server).await;
        let _read = create(&mut server, &["storage:read"], "nak_read_0123456789").await;
        let _write = create(
            &mut server,
            &["storage:read", "storage:write"],
            "nak_write_0123456789",
        )
        .await;
        let h = handler(enabled(&server));
        let table_id = seed(&h, "arn:neon:storage::proj:branch/br-1").await;

        let (captured, _guard) = audit_capture::capture();
        h.generate_temporary_table_credentials(table_vend(&table_id, false), ctx())
            .await
            .unwrap();
        h.generate_temporary_path_credentials(
            GenerateTemporaryPathCredentialsRequest {
                url: "s3://bucket/landing".into(),
                operation: generate_temporary_path_credentials_request::Operation::PathReadWrite
                    .into(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        DeltaApiHandler::get_table_credentials(
            &h,
            TableRef {
                catalog: "cat".into(),
                schema: "sch".into(),
                table: "t".into(),
            },
            DeltaCredentialOperation::Read,
            ctx(),
        )
        .await
        .unwrap();

        let events = captured.events();
        assert_eq!(events.len(), 3, "{events:#?}");
        let expect = |event: &audit_capture::AuditEvent, pairs: &[(&str, &str)]| {
            for (field, value) in pairs {
                assert_eq!(
                    event.get(*field).map(String::as_str),
                    Some(*value),
                    "{field}: {event:#?}"
                );
            }
        };
        let common = [
            ("event", "credential_vend"),
            ("outcome", "ok"),
            ("principal", "anonymous"),
            ("credential_kind", "neon_storage"),
            ("expires_at", "2030-01-01T00:00:00+00:00"),
        ];
        for event in &events {
            expect(event, &common);
            for value in event.values() {
                assert!(!value.contains("SECRET-"), "a secret leaked: {event:#?}");
            }
        }
        expect(
            &events[0],
            &[
                ("surface", "uc_rest"),
                ("securable", "cat.sch.t"),
                ("access", "read"),
                ("credential_id", "nak_read"),
            ],
        );
        expect(
            &events[1],
            &[
                ("surface", "uc_rest"),
                ("securable", "s3://bucket/landing"),
                ("access", "read_write"),
                ("credential_id", "nak_writ"),
            ],
        );
        expect(
            &events[2],
            &[
                ("surface", "delta_api"),
                ("securable", "cat.sch.t"),
                ("access", "read"),
            ],
        );
    }

    /// A failing control plane is a server error, never a credential.
    #[tokio::test]
    async fn control_plane_failures_do_not_vend() {
        let mut server = Server::new_async().await;
        let _storage = storage(&mut server).await;
        let _fail = server
            .mock("POST", format!("{BRANCH}/credentials").as_str())
            .with_status(500)
            .with_body(json!({ "code": "", "message": "boom" }).to_string())
            .create_async()
            .await;

        let h = handler(enabled(&server));
        let table_id = seed(&h, "arn:neon:storage::proj:branch/br-1").await;
        let err = h
            .generate_temporary_table_credentials(table_vend(&table_id, false), ctx())
            .await
            .unwrap_err();
        assert_eq!(err.error_code(), "INTERNAL_ERROR", "{err:?}");
    }
}
