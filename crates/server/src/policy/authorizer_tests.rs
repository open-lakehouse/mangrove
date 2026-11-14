//! `AuthorizerPolicy` in front of the server handlers, deciding with the
//! published Cedar fragment (`CedarAuthorizer`) or a remote decision point
//! (`HttpAuthorizer`).

use std::sync::{Arc, Mutex};
use std::time::Duration;

use olai_auth::{
    AuthMethod, AuthorizeRequest, AuthorizeResponse, Authorizer, BoxError, CedarAuthorizer,
    EntityLoader, EntityRef, PrincipalKind, async_trait,
};
use serde_json::{Value, json};
use unitycatalog_common::models::catalogs::v1::{CreateCatalogRequest, ListCatalogsRequest};
use unitycatalog_common::models::credentials::v1::{
    AwsIamRoleConfig, CreateCredentialRequest, Purpose,
};
use unitycatalog_common::models::external_locations::v1::CreateExternalLocationRequest;
use unitycatalog_common::models::schemas::v1::CreateSchemaRequest;
use unitycatalog_common::models::tables::v1::{
    CreateTableRequest, DataSourceFormat, GetTableRequest, ListTablesRequest, TableType,
};
use unitycatalog_common::models::temporary_credentials::v1::*;
use unitycatalog_common::models::volumes::v1::{CreateVolumeRequest, VolumeType};
use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};
use unitycatalog_delta_api::backend::{CredentialAccess, SchemaRef, TableRef};
use unitycatalog_delta_api::{DeltaAction, DeltaBackend, DeltaBackendError};

use super::*;
use crate::Error;
use crate::api::{
    CatalogHandler, CredentialHandler, ExternalLocationHandler, SchemaHandler, TableHandler,
    TemporaryCredentialHandler, VolumeHandler,
};
use crate::memory::InMemoryResourceStore;
use crate::services::ServerHandler;

/// The host's principal types, which the fragment refers to.
const LAKEHOUSE: &str = "namespace Lakehouse { entity User; entity ServiceAccount; }";

/// `admin` may do anything. `reader` may read, write and create in `cat.sch`
/// and read the external location, but holds no EXTERNAL USE. `vendor` holds
/// EXTERNAL USE on both as well.
const POLICIES: &str = r#"
@id("admin")
permit (principal == Lakehouse::User::"admin", action, resource);

@id("reader-schema")
permit (
  principal == Lakehouse::User::"reader",
  action in [
    UnityCatalog::Action::"read", UnityCatalog::Action::"write",
    UnityCatalog::Action::"create", UnityCatalog::Action::"use"
  ],
  resource in UnityCatalog::Schema::"cat.sch"
);

@id("reader-location")
permit (
  principal == Lakehouse::User::"reader",
  action == UnityCatalog::Action::"read",
  resource == UnityCatalog::ExternalLocation::"ext"
);

@id("vendor-schema")
permit (
  principal == Lakehouse::User::"vendor",
  action in [
    UnityCatalog::Action::"read", UnityCatalog::Action::"write",
    UnityCatalog::Action::"create", UnityCatalog::Action::"use"
  ],
  resource in UnityCatalog::Schema::"cat.sch"
);

@id("vendor-location")
permit (
  principal == Lakehouse::User::"vendor",
  action == UnityCatalog::Action::"read",
  resource == UnityCatalog::ExternalLocation::"ext"
);

@id("vendor-schema-external-use")
permit (
  principal == Lakehouse::User::"vendor",
  action == UnityCatalog::Action::"external_use",
  resource in UnityCatalog::Schema::"cat.sch"
);

@id("vendor-location-external-use")
permit (
  principal == Lakehouse::User::"vendor",
  action == UnityCatalog::Action::"external_use",
  resource == UnityCatalog::ExternalLocation::"ext"
);
"#;

/// Only the principal: everything about the resource comes with the request.
struct Principals;

#[async_trait]
impl EntityLoader for Principals {
    async fn load(&self, request: &AuthorizeRequest) -> Result<Value, BoxError> {
        Ok(json!([{ "uid": request.principal, "attrs": {}, "parents": [] }]))
    }
}

/// Records each `(action, resource)` it is asked about.
struct Recording {
    inner: Box<dyn Authorizer>,
    seen: Mutex<Vec<(String, EntityRef)>>,
}

impl Recording {
    fn new(inner: impl Authorizer) -> Arc<Self> {
        Arc::new(Self {
            inner: Box::new(inner),
            seen: Mutex::default(),
        })
    }

    fn seen(&self) -> Vec<(String, EntityRef)> {
        self.seen.lock().unwrap().clone()
    }

    fn asked(&self, action: &str, kind: &str, id: &str) -> bool {
        self.seen().iter().any(|(a, r)| {
            a == action && r.entity_type == format!("UnityCatalog::{kind}") && r.id == id
        })
    }
}

#[async_trait]
impl Authorizer for Recording {
    async fn authorize(&self, request: &AuthorizeRequest) -> AuthorizeResponse {
        self.seen
            .lock()
            .unwrap()
            .push((request.action.id.clone(), request.resource.clone()));
        self.inner.authorize(request).await
    }
}

/// Fails every decision, like an unreachable decision point.
struct Unreachable;

#[async_trait]
impl Authorizer for Unreachable {
    async fn authorize(&self, _: &AuthorizeRequest) -> AuthorizeResponse {
        AuthorizeResponse::deny("decision point unreachable")
    }
}

fn cedar() -> CedarAuthorizer {
    let schema = format!("{LAKEHOUSE}\n{CEDAR_SCHEMA}");
    CedarAuthorizer::new(&schema, POLICIES, Principals).unwrap()
}

/// The local storage the seeded table and volume live in: local locations vend
/// without minting, so an authorized vend succeeds without any cloud.
fn local_root() -> std::path::PathBuf {
    let root = std::env::temp_dir().join("uc-authorizer-policy-tests");
    std::fs::create_dir_all(&root).unwrap();
    root.canonicalize().unwrap()
}

fn handler(authorizer: impl Authorizer) -> ServerHandler<RequestContext> {
    let encryptor =
        EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
    let store = Arc::new(InMemoryResourceStore::new(encryptor));
    let policy = AuthorizerPolicy::new(authorizer, store.clone());
    ServerHandler::try_new_tokio(Arc::new(policy), store)
        .unwrap()
        .with_local_storage_policy(
            crate::services::location_policy::LocalStoragePolicy::new([local_root()]).unwrap(),
        )
}

fn local(path: &str) -> String {
    format!("file://{}/{path}", local_root().display())
}

fn user(sub: &str) -> RequestContext {
    RequestContext::from_verified(olai_auth::Principal {
        kind: PrincipalKind::User {
            sub: sub.into(),
            email: None,
            email_verified: false,
        },
        issuer: "https://auth.example.com".into(),
        scopes: vec![],
        client_id: None,
        auth_method: AuthMethod::Jwt,
        admin: false,
        issued_at: None,
    })
}

/// `cat.sch.t` (an external table) and `cat.sch.v` (an external volume) under
/// external location `ext`, created by `admin`. Returns the table and volume ids.
async fn seed(h: &ServerHandler<RequestContext>) -> (String, String) {
    let admin = || user("admin");
    h.create_credential(
        CreateCredentialRequest {
            name: "cred".into(),
            purpose: Purpose::Storage.into(),
            aws_iam_role: Some(AwsIamRoleConfig {
                role_arn: "arn:aws:iam::123456789012:role/test".into(),
                ..Default::default()
            })
            .into(),
            ..Default::default()
        },
        admin(),
    )
    .await
    .unwrap();
    h.create_external_location(
        CreateExternalLocationRequest {
            name: "ext".into(),
            url: "s3://bucket/ext".into(),
            credential_name: "cred".into(),
            ..Default::default()
        },
        admin(),
    )
    .await
    .unwrap();
    // Governs the local table and volume.
    h.create_external_location(
        CreateExternalLocationRequest {
            name: "local".into(),
            url: format!("file://{}", local_root().display()),
            credential_name: "cred".into(),
            ..Default::default()
        },
        admin(),
    )
    .await
    .unwrap();
    h.create_catalog(
        CreateCatalogRequest {
            name: "cat".into(),
            storage_root: Some("s3://bucket/ext/cat".into()),
            ..Default::default()
        },
        admin(),
    )
    .await
    .unwrap();
    h.create_schema(
        CreateSchemaRequest {
            name: "sch".into(),
            catalog_name: "cat".into(),
            ..Default::default()
        },
        admin(),
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
                storage_location: Some(local("cat/sch/t")),
                ..Default::default()
            },
            admin(),
        )
        .await
        .unwrap();
    let volume = h
        .create_volume(
            CreateVolumeRequest {
                name: "v".into(),
                catalog_name: "cat".into(),
                schema_name: "sch".into(),
                volume_type: VolumeType::External.into(),
                storage_location: Some(local("cat/sch/v")),
                ..Default::default()
            },
            admin(),
        )
        .await
        .unwrap();
    (table.table_id.unwrap(), volume.volume_id)
}

fn table_vend(table_id: &str) -> GenerateTemporaryTableCredentialsRequest {
    GenerateTemporaryTableCredentialsRequest {
        table_id: table_id.into(),
        operation: generate_temporary_table_credentials_request::Operation::READ.into(),
        ..Default::default()
    }
}

fn authorized<T: std::fmt::Debug>(result: crate::Result<T>) {
    assert!(result.is_ok(), "{result:?}");
}

fn denied<T: std::fmt::Debug>(result: crate::Result<T>) {
    assert!(matches!(result, Err(Error::NotAllowed)), "{result:?}");
}

#[tokio::test]
async fn the_fragment_validates_with_the_host_principals() {
    let schema = format!("{LAKEHOUSE}\n{CEDAR_SCHEMA}");
    CedarAuthorizer::new(&schema, POLICIES, Principals).expect("the schema and policies validate");
}

#[tokio::test]
async fn reading_a_table_does_not_vend_its_credentials() {
    let recording = Recording::new(cedar());
    let h = handler(recording.clone());
    let (table_id, _) = seed(&h).await;

    let table = h
        .get_table(
            GetTableRequest {
                full_name: "cat.sch.t".into(),
                ..Default::default()
            },
            user("reader"),
        )
        .await;
    assert!(table.is_ok(), "{table:?}");

    denied(
        h.generate_temporary_table_credentials(table_vend(&table_id), user("reader"))
            .await,
    );
    // The table, addressed by id, was resolved to its name for the decision.
    assert!(recording.asked("read", "Table", "cat.sch.t"));
    assert!(recording.asked("external_use", "Schema", "cat.sch"));
}

#[tokio::test]
async fn external_use_on_the_schema_vends_table_volume_and_model_credentials() {
    let h = handler(cedar());
    let (table_id, volume_id) = seed(&h).await;

    authorized(
        h.generate_temporary_table_credentials(table_vend(&table_id), user("vendor"))
            .await,
    );

    let volume = || GenerateTemporaryVolumeCredentialsRequest {
        volume_id: volume_id.clone(),
        operation: generate_temporary_volume_credentials_request::Operation::READ_VOLUME.into(),
        ..Default::default()
    };
    denied(
        h.generate_temporary_volume_credentials(volume(), user("reader"))
            .await,
    );
    authorized(
        h.generate_temporary_volume_credentials(volume(), user("vendor"))
            .await,
    );

    let model_version = || GenerateTemporaryModelVersionCredentialsRequest {
        catalog_name: "cat".into(),
        schema_name: "sch".into(),
        model_name: "m".into(),
        version: 1,
        ..Default::default()
    };
    denied(
        h.generate_temporary_model_version_credentials(model_version(), user("reader"))
            .await,
    );
    // Authorized, and then there is no such model version.
    let result = h
        .generate_temporary_model_version_credentials(model_version(), user("vendor"))
        .await;
    assert!(
        result
            .as_ref()
            .is_err_and(|e| !matches!(e, Error::NotAllowed)),
        "{result:?}"
    );
}

#[tokio::test]
async fn path_credentials_need_external_use_on_the_external_location() {
    let recording = Recording::new(cedar());
    let h = handler(recording.clone());
    seed(&h).await;
    let path = || GenerateTemporaryPathCredentialsRequest {
        url: "s3://bucket/ext/landing".into(),
        operation: generate_temporary_path_credentials_request::Operation::PATH_READ.into(),
        ..Default::default()
    };

    denied(
        h.generate_temporary_path_credentials(path(), user("reader"))
            .await,
    );
    assert!(recording.asked("read", "ExternalLocation", "ext"));
    assert!(recording.asked("external_use", "ExternalLocation", "ext"));
    // Allowed; minting the credential needs a cloud, so stop at the decision.
    let url =
        crate::services::location::StorageLocationUrl::parse("s3://bucket/ext/landing").unwrap();
    let vendor = user("vendor");
    crate::api::temporary_credentials::authorize_path_vend(
        &h,
        &url,
        olai_storage_credentials::AccessLevel::Read,
        &vendor,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn delta_vends_need_external_use() {
    let h = handler(cedar());
    seed(&h).await;
    let table = TableRef {
        catalog: "cat".into(),
        schema: "sch".into(),
        table: "t".into(),
    };
    let vend = || DeltaAction::VendTableCredential {
        table: &table,
        access: CredentialAccess::Read,
    };
    let result = DeltaBackend::authorize(&h, vend(), &user("reader")).await;
    assert!(
        matches!(result, Err(DeltaBackendError::PermissionDenied(_))),
        "{result:?}"
    );
    DeltaBackend::authorize(&h, vend(), &user("vendor"))
        .await
        .unwrap();

    let at = SchemaRef {
        catalog: "cat".into(),
        schema: "sch".into(),
    };
    let staging = || DeltaAction::CreateStaging {
        at: &at,
        name: "staged",
    };
    let result = DeltaBackend::authorize(&h, staging(), &user("reader")).await;
    assert!(
        matches!(result, Err(DeltaBackendError::PermissionDenied(_))),
        "{result:?}"
    );
    DeltaBackend::authorize(&h, staging(), &user("vendor"))
        .await
        .unwrap();

    let path = || DeltaAction::VendPathCredential {
        location: "s3://bucket/ext/landing",
        access: CredentialAccess::Read,
    };
    assert!(
        DeltaBackend::authorize(&h, path(), &user("reader"))
            .await
            .is_err()
    );
    DeltaBackend::authorize(&h, path(), &user("vendor"))
        .await
        .unwrap();
}

#[tokio::test]
async fn lists_are_filtered_by_name_without_lookups() {
    let h = handler(cedar());
    seed(&h).await;
    let tables = h
        .list_tables(
            ListTablesRequest {
                catalog_name: "cat".into(),
                schema_name: "sch".into(),
                ..Default::default()
            },
            user("reader"),
        )
        .await
        .unwrap();
    assert_eq!(tables.tables.len(), 1);

    // No grant on the metastore: no catalogs to list.
    let catalogs = h
        .list_catalogs(ListCatalogsRequest::default(), user("reader"))
        .await;
    assert!(
        matches!(&catalogs, Err(Error::NotAllowed))
            || catalogs.as_ref().is_ok_and(|c| c.catalogs.is_empty()),
        "{catalogs:?}"
    );
}

#[tokio::test]
async fn an_undecidable_request_is_denied() {
    let h = handler(Unreachable);
    let result = h
        .get_table(
            GetTableRequest {
                full_name: "cat.sch.t".into(),
                ..Default::default()
            },
            user("admin"),
        )
        .await;
    denied(result);
}

#[tokio::test]
async fn an_unverified_caller_is_denied_without_asking() {
    let recording = Recording::new(cedar());
    let h = handler(recording.clone());
    for cx in [
        RequestContext::anonymous(),
        RequestContext::new(crate::policy::Principal::user("admin")),
    ] {
        let result = h
            .get_table(
                GetTableRequest {
                    full_name: "cat.sch.t".into(),
                    ..Default::default()
                },
                cx,
            )
            .await;
        denied(result);
    }
    assert!(recording.seen().is_empty());
}

/// The work package's end-to-end check: behind a decision point that denies
/// everything, every securable route and every vend answers 403. The catalog is
/// seeded through an allow-all handler on the same stores, so each request
/// reaches the decision point instead of stopping at a missing object.
#[tokio::test]
async fn a_denying_decision_point_forbids_every_route_and_vend() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use axum::routing::post;
    use axum::{Extension, Json, Router};
    use olai_auth::wire::{
        AUTHORIZE_BATCH_PATH, AUTHORIZE_PATH, BatchAuthorizeRequest, BatchAuthorizeResponse,
    };
    use tower::ServiceExt as _;
    use unitycatalog_delta_api::coordinator::InMemoryCommitCoordinator;
    use unitycatalog_storage_access::InMemoryStorageAccess;

    use crate::policy::ConstantPolicy;
    use crate::run::build_rest_router;

    // The decision point: denies, and counts what it is asked.
    let asked = Arc::new(AtomicUsize::new(0));
    let deny = || AuthorizeResponse {
        decision: olai_auth::Decision::Deny,
        policies: vec![],
        errors: vec![],
    };
    let pdp = Router::new()
        .route(
            AUTHORIZE_PATH,
            post({
                let asked = asked.clone();
                move || async move {
                    asked.fetch_add(1, Ordering::SeqCst);
                    Json(deny())
                }
            }),
        )
        .route(
            AUTHORIZE_BATCH_PATH,
            post({
                let asked = asked.clone();
                move |Json(batch): Json<BatchAuthorizeRequest>| async move {
                    asked.fetch_add(batch.requests.len(), Ordering::SeqCst);
                    Json(BatchAuthorizeResponse {
                        responses: batch.requests.iter().map(|_| deny()).collect(),
                    })
                }
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let pdp_url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, pdp).await.unwrap() });

    let encryptor =
        EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
    let store = Arc::new(InMemoryResourceStore::new(encryptor));
    let storage_access = Arc::new(InMemoryStorageAccess::new());
    let coordinator = Arc::new(InMemoryCommitCoordinator::default());
    let local = |h: ServerHandler<RequestContext>| {
        h.with_local_storage_policy(
            crate::services::location_policy::LocalStoragePolicy::new([local_root()]).unwrap(),
        )
    };
    let seeder = local(
        ServerHandler::try_new_tokio_with_coordinator(
            Arc::new(ConstantPolicy::default()),
            store.clone(),
            storage_access.clone(),
            coordinator.clone(),
        )
        .unwrap(),
    );
    let (table_id, volume_id) = seed(&seeder).await;

    let policy =
        AuthorizerPolicy::remote(&pdp_url, "pdp-secret", None, Duration::ZERO, store.clone());
    let handler = local(
        ServerHandler::try_new_tokio_with_coordinator(
            Arc::new(policy),
            store,
            storage_access,
            coordinator,
        )
        .unwrap(),
    );
    let verified = user("admin").verified_principal().unwrap().clone();
    let app = build_rest_router(handler).layer(Extension(verified));

    let uc = "/api/2.1/unity-catalog";
    let delta = format!("{uc}/delta/v1");
    let forbidden: Vec<(&str, String, Option<String>)> = vec![
        ("GET", format!("{uc}/catalogs/cat"), None),
        ("POST", format!("{uc}/catalogs"), Some(r#"{"name":"other"}"#.into())),
        ("GET", format!("{uc}/schemas/cat.sch"), None),
        ("GET", format!("{uc}/tables/cat.sch.t"), None),
        ("DELETE", format!("{uc}/tables/cat.sch.t"), None),
        ("GET", format!("{uc}/volumes/cat.sch.v"), None),
        ("GET", format!("{uc}/credentials/cred"), None),
        ("GET", format!("{uc}/external-locations/ext"), None),
        (
            "POST",
            format!("{uc}/temporary-table-credentials"),
            Some(format!(r#"{{"table_id":"{table_id}","operation":"READ"}}"#)),
        ),
        (
            "POST",
            format!("{uc}/temporary-volume-credentials"),
            Some(format!(r#"{{"volume_id":"{volume_id}","operation":"READ_VOLUME"}}"#)),
        ),
        (
            "POST",
            format!("{uc}/temporary-path-credentials"),
            Some(r#"{"url":"s3://bucket/ext/landing","operation":"PATH_READ"}"#.into()),
        ),
        (
            "POST",
            format!("{uc}/temporary-model-version-credentials"),
            Some(
                r#"{"catalog_name":"cat","schema_name":"sch","model_name":"m","version":1,"operation":"READ_MODEL_VERSION"}"#
                    .into(),
            ),
        ),
        ("GET", format!("{delta}/config?catalog=cat&protocol-versions=1.0"), None),
        ("GET", format!("{delta}/catalogs/cat/schemas/sch/tables/t"), None),
        (
            "GET",
            format!("{delta}/catalogs/cat/schemas/sch/tables/t/credentials?operation=READ"),
            None,
        ),
        (
            "GET",
            format!("{delta}/temporary-path-credentials?location=s3://bucket/ext/landing&operation=READ"),
            None,
        ),
    ];
    for (method, uri, body) in forbidden {
        let mut request = Request::builder().method(method).uri(&uri);
        if body.is_some() {
            request = request.header(header::CONTENT_TYPE, "application/json");
        }
        let request = request
            .body(body.map(Body::from).unwrap_or_else(Body::empty))
            .unwrap();
        let before = asked.load(Ordering::SeqCst);
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{method} {uri}");
        assert!(
            asked.load(Ordering::SeqCst) > before,
            "{method} {uri} asked nobody"
        );
    }

    // Lists answer, with nothing in them.
    let request = Request::builder()
        .uri(format!("{uc}/tables?catalog_name=cat&schema_name=sch"))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    assert!(
        status == StatusCode::FORBIDDEN || body["tables"].as_array().is_none_or(|t| t.is_empty()),
        "{status} {body}"
    );
}

#[path = "../../tests/support/audit_capture.rs"]
mod audit_capture;

/// A vend's audit event names the caller and the policies that allowed its
/// checks: read on the table and EXTERNAL USE on its schema.
#[tokio::test]
async fn vend_audit_events_carry_the_determining_policies() {
    let h = handler(cedar());
    let (table_id, _) = seed(&h).await;

    let (captured, _guard) = audit_capture::capture();
    h.generate_temporary_table_credentials(table_vend(&table_id), user("vendor"))
        .await
        .unwrap();
    let [event] = captured.events().try_into().unwrap();
    assert_eq!(event["principal"], "vendor");
    assert_eq!(event["securable"], "cat.sch.t");
    assert_eq!(event["credential_kind"], "local");
    let policies = &event["policies"];
    assert!(policies.contains("vendor-schema"), "{policies}");
    assert!(
        policies.contains("vendor-schema-external-use"),
        "{policies}"
    );
    assert!(!policies.contains("admin"), "{policies}");

    // A denied caller never reaches the vend, so nothing is audited for it.
    denied(
        h.generate_temporary_table_credentials(table_vend(&table_id), user("reader"))
            .await,
    );
    assert_eq!(captured.events().len(), 1);
}
