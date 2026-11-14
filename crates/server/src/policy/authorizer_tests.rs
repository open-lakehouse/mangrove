//! `AuthorizerPolicy` in front of the server handlers, deciding with the
//! published Cedar schema, a UC-rules policy (`testdata/uc_rules.cedar`) and the
//! grants the handlers keep, through an in-process `CedarAuthorizer` whose
//! `GrantsEntityLoader` reads the handler's own grant store.

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use olai_auth::grants::{
    Grant, GrantChange, GrantError, GrantStore, GroupMembership, MemoryGrantStore, ObjectRef,
    Subject,
};
use olai_auth::{
    AuthMethod, AuthorizeRequest, AuthorizeResponse, Authorizer, CedarAuthorizer, EntityRef,
    GrantsEntityLoader, PrincipalKind, SubjectTypes, async_trait,
};
use rstest::rstest;
use unitycatalog_common::models::catalogs::v1::{
    CreateCatalogRequest, DeleteCatalogRequest, ListCatalogsRequest, UpdateCatalogRequest,
};
use unitycatalog_common::models::credentials::v1::{
    AwsIamRoleConfig, CreateCredentialRequest, Purpose,
};
use unitycatalog_common::models::external_locations::v1::CreateExternalLocationRequest;
use unitycatalog_common::models::grants::v1::{
    GetGrantsRequest, GrantChange as ApiChange, UpdateGrantsRequest,
};
use unitycatalog_common::models::schemas::v1::{CreateSchemaRequest, DeleteSchemaRequest};
use unitycatalog_common::models::tables::v1::{
    CreateTableRequest, DataSourceFormat, DeleteTableRequest, GetTableRequest, ListTablesRequest,
    TableType,
};
use unitycatalog_common::models::temporary_credentials::v1::*;
use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};
use unitycatalog_delta_api::backend::{CredentialAccess, TableRef};
use unitycatalog_delta_api::{DeltaAction, DeltaBackend, DeltaBackendError};

use super::*;
use crate::Error;
use crate::api::{
    CatalogHandler, CredentialHandler, ExternalLocationHandler, GrantHandler, SchemaHandler,
    TableHandler, TemporaryCredentialHandler,
};
use crate::grants::{Grants, IdentityDirectory, OWNER};
use crate::memory::InMemoryResourceStore;
use crate::policy::CEDAR_SCHEMA_FRAGMENTS;
use crate::services::ServerHandler;

/// The host's principal types, and the groups they are in.
const LAKEHOUSE: &str = "namespace Lakehouse { entity Group in [Group]; entity User in [Group]; entity ServiceAccount in [Group]; }";

/// The UC rules the tests decide with.
const POLICIES: &str = include_str!("testdata/uc_rules.cedar");

/// One grant store, shared by the handler (which writes it) and the loader
/// (which reads it).
#[derive(Clone, Default)]
struct Shared(Arc<MemoryGrantStore>);

#[async_trait]
impl GrantStore for Shared {
    async fn list(&self, object: &ObjectRef) -> Result<Vec<Grant>, GrantError> {
        self.0.list(object).await
    }
    async fn apply(
        &self,
        object: &ObjectRef,
        changes: &[GrantChange],
    ) -> Result<Vec<Grant>, GrantError> {
        self.0.apply(object, changes).await
    }
    async fn delete_object(&self, object: &ObjectRef, types: &[&str]) -> Result<u64, GrantError> {
        self.0.delete_object(object, types).await
    }
    async fn rename(
        &self,
        object: &ObjectRef,
        new_id: &str,
        types: &[&str],
    ) -> Result<u64, GrantError> {
        self.0.rename(object, new_id, types).await
    }
    async fn groups(&self, subject: &Subject) -> Result<Vec<GroupMembership>, GrantError> {
        self.0.groups(subject).await
    }
    async fn effective(
        &self,
        subject: &Subject,
        objects: &[ObjectRef],
    ) -> Result<Vec<Grant>, GrantError> {
        self.0.effective(subject, objects).await
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

    fn asked(&self, action: &str, kind: &str, id: &str) -> bool {
        self.seen.lock().unwrap().iter().any(|(a, r)| {
            a == action && r.entity_type == format!("UnityCatalog::{kind}") && r.id == id
        })
    }

    fn count(&self) -> usize {
        self.seen.lock().unwrap().len()
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

/// The host's fragment and the server's.
fn fragments() -> Vec<&'static str> {
    let mut fragments = vec![LAKEHOUSE];
    fragments.extend_from_slice(CEDAR_SCHEMA_FRAGMENTS);
    fragments
}

fn cedar(store: &Shared) -> CedarAuthorizer {
    let loader = GrantsEntityLoader::new(
        store.clone(),
        SubjectTypes::new(
            "Lakehouse::User",
            "Lakehouse::ServiceAccount",
            "Lakehouse::Group",
        ),
    );
    CedarAuthorizer::from_fragments(&fragments(), POLICIES, loader).unwrap()
}

/// The local storage the seeded tables live in: local locations vend without
/// minting, so an authorized vend succeeds without any cloud.
fn local_root() -> std::path::PathBuf {
    let root = std::env::temp_dir().join("uc-authorizer-policy-tests");
    std::fs::create_dir_all(&root).unwrap();
    root.canonicalize().unwrap()
}

fn local(path: &str) -> String {
    format!("file://{}/{path}", local_root().display())
}

fn handler(authorizer: impl Authorizer, store: &Shared) -> ServerHandler<RequestContext> {
    let encryptor =
        EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
    let resources = Arc::new(InMemoryResourceStore::new(encryptor));
    let policy = AuthorizerPolicy::new(authorizer);
    ServerHandler::try_new_tokio(Arc::new(policy), resources)
        .unwrap()
        .with_grants(Grants::new(
            Arc::new(store.clone()),
            Arc::new(IdentityDirectory),
        ))
        .with_local_storage_policy(
            crate::services::location_policy::LocalStoragePolicy::new([local_root()]).unwrap(),
        )
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

fn uc(kind: &str, id: &str) -> ObjectRef {
    ObjectRef::new(format!("UnityCatalog::{kind}"), id)
}

/// Grant `privileges` on `object` to `subject`, straight in the store.
async fn grant(store: &Shared, subject: Subject, object: ObjectRef, privileges: &[&str]) {
    let change = GrantChange {
        subject,
        add: privileges.iter().map(|p| p.to_string()).collect(),
        remove: vec![],
    };
    store.apply(&object, &[change]).await.unwrap();
}

/// External location `local` over the local root, catalog `cat`, schema
/// `cat.sch` and the external tables `cat.sch.t` and `cat.sch.t2`, created by
/// `admin`. Returns the id of `cat.sch.t`.
async fn seed(h: &ServerHandler<RequestContext>) -> String {
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
            storage_root: Some(local("cat")),
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
    let mut ids = Vec::new();
    for name in ["t", "t2"] {
        let table = h.create_table(create_table(name), admin()).await.unwrap();
        ids.push(table.table_id.unwrap());
    }
    ids.swap_remove(0)
}

fn create_table(name: &str) -> CreateTableRequest {
    CreateTableRequest {
        name: name.into(),
        catalog_name: "cat".into(),
        schema_name: "sch".into(),
        table_type: TableType::External.into(),
        data_source_format: DataSourceFormat::Delta.into(),
        storage_location: Some(local(&format!("cat/sch/{name}"))),
        ..Default::default()
    }
}

/// The callers of the matrix, each with the grants that make it what it is.
const SUBJECTS: [&str; 7] = [
    "owner",
    "table_reader",
    "schema_reader",
    "catalog_reader",
    "group_reader",
    "writer",
    "nobody",
];

/// A handler over `cat.sch.{t,t2}` and the grants of [`SUBJECTS`]. Returns the
/// handler, the store, and the id of `cat.sch.t`.
async fn world() -> (ServerHandler<RequestContext>, Shared, String) {
    let store = Shared::default();
    let h = handler(cedar(&store), &store);
    let table_id = seed(&h).await;
    let uses = |who: &'static str| {
        let store = store.clone();
        async move {
            grant(
                &store,
                Subject::user(who),
                uc("Catalog", "cat"),
                &["USE_CATALOG"],
            )
            .await;
            grant(
                &store,
                Subject::user(who),
                uc("Schema", "cat.sch"),
                &["USE_SCHEMA"],
            )
            .await;
        }
    };
    // The owner of `t`, who may create in the schema.
    uses("owner").await;
    grant(
        &store,
        Subject::user("owner"),
        uc("Table", "cat.sch.t"),
        &[OWNER],
    )
    .await;
    grant(
        &store,
        Subject::user("owner"),
        uc("Schema", "cat.sch"),
        &["CREATE_TABLE"],
    )
    .await;
    // SELECT on the table, the schema, the catalog.
    uses("table_reader").await;
    grant(
        &store,
        Subject::user("table_reader"),
        uc("Table", "cat.sch.t"),
        &["SELECT"],
    )
    .await;
    uses("schema_reader").await;
    grant(
        &store,
        Subject::user("schema_reader"),
        uc("Schema", "cat.sch"),
        &["SELECT"],
    )
    .await;
    grant(
        &store,
        Subject::user("catalog_reader"),
        uc("Catalog", "cat"),
        &["USE_CATALOG", "USE_SCHEMA", "SELECT"],
    )
    .await;
    // The catalog reader's grants, through a group.
    store.0.add_group("g-analysts", "analysts");
    store
        .0
        .add_member("g-analysts", Subject::user("group_reader"));
    grant(
        &store,
        Subject::group("g-analysts"),
        uc("Catalog", "cat"),
        &["USE_CATALOG", "USE_SCHEMA", "SELECT"],
    )
    .await;
    // Reads, writes, creates and receives credentials in the schema.
    uses("writer").await;
    grant(
        &store,
        Subject::user("writer"),
        uc("Schema", "cat.sch"),
        &["SELECT", "MODIFY", "CREATE_TABLE", "EXTERNAL_USE_SCHEMA"],
    )
    .await;
    (h, store, table_id)
}

/// An operation of the matrix, run as `who`. `Ok` when it was allowed.
#[derive(Debug, Clone, Copy)]
enum Op {
    Get,
    Create,
    Delete,
    VendRead,
    VendReadWrite,
    Commit,
    Rename,
}

async fn run(op: Op, h: &ServerHandler<RequestContext>, table_id: &str, who: &str) -> bool {
    let cx = user(who);
    let table = TableRef {
        catalog: "cat".into(),
        schema: "sch".into(),
        table: "t".into(),
    };
    let vend = |operation: generate_temporary_table_credentials_request::Operation| {
        GenerateTemporaryTableCredentialsRequest {
            table_id: table_id.into(),
            operation: operation.into(),
            ..Default::default()
        }
    };
    let result: std::result::Result<(), String> = match op {
        Op::Get => h
            .get_table(
                GetTableRequest {
                    full_name: "cat.sch.t".into(),
                    ..Default::default()
                },
                cx,
            )
            .await
            .map(|_| ())
            .map_err(|e| e.to_string()),
        Op::Create => h
            .create_table(create_table("t3"), cx)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string()),
        Op::Delete => TableHandler::delete_table(
            h,
            DeleteTableRequest {
                full_name: "cat.sch.t".into(),
                ..Default::default()
            },
            cx,
        )
        .await
        .map_err(|e| e.to_string()),
        Op::VendRead => h
            .generate_temporary_table_credentials(
                vend(generate_temporary_table_credentials_request::Operation::READ),
                cx,
            )
            .await
            .map(|_| ())
            .map_err(|e| e.to_string()),
        Op::VendReadWrite => h
            .generate_temporary_table_credentials(
                vend(generate_temporary_table_credentials_request::Operation::READ_WRITE),
                cx,
            )
            .await
            .map(|_| ())
            .map_err(|e| e.to_string()),
        Op::Commit => DeltaBackend::authorize(h, DeltaAction::WriteTable { table: &table }, &cx)
            .await
            .map_err(|e| e.to_string()),
        Op::Rename => DeltaBackend::authorize(
            h,
            DeltaAction::RenameTable {
                from: &table,
                to: "t9",
            },
            &cx,
        )
        .await
        .map_err(|e| e.to_string()),
    };
    match result {
        Ok(()) => true,
        Err(e) if e.to_lowercase().contains("not allowed") || e.contains("ermission") => false,
        Err(e) => panic!("{op:?} as {who} failed for another reason: {e}"),
    }
}

/// Who may do what: each operation on `cat.sch.t` (or creating `cat.sch.t3`).
#[rstest]
#[case::get(
    Op::Get,
    &["owner", "table_reader", "schema_reader", "catalog_reader", "group_reader", "writer"]
)]
#[case::create(Op::Create, &["owner", "writer"])]
#[case::delete(Op::Delete, &["owner"])]
#[case::vend_read(Op::VendRead, &["writer"])]
#[case::vend_read_write(Op::VendReadWrite, &["writer"])]
#[case::delta_commit(Op::Commit, &["owner", "writer"])]
// Renaming needs the table (owner) and creating the new name (CREATE_TABLE).
#[case::delta_rename(Op::Rename, &["owner"])]
#[tokio::test]
async fn the_matrix(#[case] op: Op, #[case] allowed: &[&str]) {
    for who in SUBJECTS {
        // A fresh world per caller: deletes and creates change it.
        let (h, _, table_id) = world().await;
        assert_eq!(
            run(op, &h, &table_id, who).await,
            allowed.contains(&who),
            "{op:?} as {who}"
        );
    }
}

#[tokio::test]
async fn lists_show_what_the_caller_may_see() {
    let (h, _, _) = world().await;
    let visible = |who: &'static str| {
        let h = h.clone();
        async move {
            h.list_tables(
                ListTablesRequest {
                    catalog_name: "cat".into(),
                    schema_name: "sch".into(),
                    ..Default::default()
                },
                user(who),
            )
            .await
            .map(|r| {
                r.tables
                    .into_iter()
                    .map(|t| t.name)
                    .collect::<BTreeSet<_>>()
            })
        }
    };
    let both: BTreeSet<String> = ["t".to_string(), "t2".to_string()].into();
    let only_t: BTreeSet<String> = ["t".to_string()].into();
    assert_eq!(visible("owner").await.unwrap(), only_t);
    assert_eq!(visible("table_reader").await.unwrap(), only_t);
    for who in ["schema_reader", "catalog_reader", "group_reader", "writer"] {
        assert_eq!(visible(who).await.unwrap(), both, "{who}");
    }
    assert!(matches!(visible("nobody").await, Err(Error::NotAllowed)));

    // Everyone lists catalogs; only those with a grant see `cat`.
    let catalogs = |who: &'static str| {
        let h = h.clone();
        async move {
            h.list_catalogs(ListCatalogsRequest::default(), user(who))
                .await
                .unwrap()
                .catalogs
                .len()
        }
    };
    assert_eq!(catalogs("catalog_reader").await, 1);
    assert_eq!(catalogs("nobody").await, 0);
}

#[tokio::test]
async fn the_schema_validates_with_the_host_principals() {
    let store = Shared::default();
    let loader = GrantsEntityLoader::new(
        store,
        SubjectTypes::new(
            "Lakehouse::User",
            "Lakehouse::ServiceAccount",
            "Lakehouse::Group",
        ),
    );
    CedarAuthorizer::from_fragments(&fragments(), POLICIES, loader)
        .expect("the schema and policies validate");
}

#[tokio::test]
async fn creators_own_what_they_create() {
    let (_, store, _) = world().await;
    for object in [
        uc("Catalog", "cat"),
        uc("Schema", "cat.sch"),
        uc("Table", "cat.sch.t2"),
        uc("ExternalLocation", "local"),
        uc("Credential", "cred"),
    ] {
        let grants = store.list(&object).await.unwrap();
        assert!(
            grants
                .iter()
                .any(|g| g.privilege == OWNER && g.subject == Subject::user("admin")),
            "{object}: {grants:?}"
        );
    }
}

fn change(principal: &str, add: &[&str], remove: &[&str]) -> ApiChange {
    ApiChange {
        principal: principal.into(),
        add: add.iter().map(|p| p.to_string()).collect(),
        remove: remove.iter().map(|p| p.to_string()).collect(),
        ..Default::default()
    }
}

fn update_grants(
    securable_type: &str,
    full_name: &str,
    changes: Vec<ApiChange>,
) -> UpdateGrantsRequest {
    UpdateGrantsRequest {
        securable_type: securable_type.into(),
        full_name: full_name.into(),
        changes,
        ..Default::default()
    }
}

#[tokio::test]
async fn a_grant_allows_and_a_revoke_denies() {
    let (h, _, _) = world().await;
    let get = |who: &'static str| {
        let h = h.clone();
        async move {
            h.get_table(
                GetTableRequest {
                    full_name: "cat.sch.t".into(),
                    ..Default::default()
                },
                user(who),
            )
            .await
        }
    };
    assert!(matches!(get("nobody").await, Err(Error::NotAllowed)));

    // Both spellings of a privilege.
    let granted = h
        .update_grants(
            update_grants(
                "catalog",
                "cat",
                vec![change(
                    "nobody",
                    &["USE CATALOG", "use_schema", "SELECT"],
                    &[],
                )],
            ),
            user("admin"),
        )
        .await
        .unwrap();
    let nobody = granted
        .privilege_assignments
        .iter()
        .find(|a| a.principal == "nobody")
        .unwrap();
    assert_eq!(nobody.privileges, ["SELECT", "USE CATALOG", "USE SCHEMA"]);
    // Ownership is not a grant the API shows.
    assert!(
        granted
            .privilege_assignments
            .iter()
            .all(|a| a.privileges.iter().all(|p| p != OWNER))
    );
    get("nobody").await.unwrap();

    h.update_grants(
        update_grants("catalog", "cat", vec![change("nobody", &[], &["SELECT"])]),
        user("admin"),
    )
    .await
    .unwrap();
    assert!(matches!(get("nobody").await, Err(Error::NotAllowed)));

    let read = h
        .get_grants(
            GetGrantsRequest {
                securable_type: "catalog".into(),
                full_name: "cat".into(),
                principal: Some("nobody".into()),
                ..Default::default()
            },
            user("admin"),
        )
        .await
        .unwrap();
    assert_eq!(read.privilege_assignments.len(), 1);
    assert_eq!(
        read.privilege_assignments[0].privileges,
        ["USE CATALOG", "USE SCHEMA"]
    );
}

#[rstest]
#[case::owner("OWNER")]
#[case::unknown("FLY")]
#[case::empty("")]
#[tokio::test]
async fn only_known_privileges_are_granted(#[case] privilege: &str) {
    let (h, _, _) = world().await;
    let result = h
        .update_grants(
            update_grants("catalog", "cat", vec![change("nobody", &[privilege], &[])]),
            user("admin"),
        )
        .await;
    assert!(
        matches!(result, Err(Error::InvalidArgument(_))),
        "{result:?}"
    );
}

#[tokio::test]
async fn grants_are_their_owners_to_manage() {
    let (h, _, _) = world().await;
    // The owner of the table manages its grants; a reader doesn't.
    h.update_grants(
        update_grants(
            "table",
            "cat.sch.t",
            vec![change("nobody", &["SELECT"], &[])],
        ),
        user("owner"),
    )
    .await
    .unwrap();
    let result = h
        .update_grants(
            update_grants(
                "table",
                "cat.sch.t",
                vec![change("nobody", &["MODIFY"], &[])],
            ),
            user("table_reader"),
        )
        .await;
    assert!(matches!(result, Err(Error::NotAllowed)), "{result:?}");
    // A securable that doesn't exist.
    let result = h
        .get_grants(
            GetGrantsRequest {
                securable_type: "table".into(),
                full_name: "cat.sch.nope".into(),
                ..Default::default()
            },
            user("admin"),
        )
        .await;
    assert!(
        result
            .as_ref()
            .is_err_and(|e| e.to_string().to_lowercase().contains("not found")),
        "{result:?}"
    );
}

#[tokio::test]
async fn deletes_drop_the_grants_in_the_securable() {
    let (h, store, _) = world().await;
    for name in ["t", "t2"] {
        TableHandler::delete_table(
            &h,
            DeleteTableRequest {
                full_name: format!("cat.sch.{name}"),
                ..Default::default()
            },
            user("admin"),
        )
        .await
        .unwrap();
    }
    assert!(
        store
            .list(&uc("Table", "cat.sch.t"))
            .await
            .unwrap()
            .is_empty()
    );
    h.delete_schema(
        DeleteSchemaRequest {
            full_name: "cat.sch".into(),
            ..Default::default()
        },
        user("admin"),
    )
    .await
    .unwrap();
    assert!(
        store
            .list(&uc("Schema", "cat.sch"))
            .await
            .unwrap()
            .is_empty()
    );
    h.delete_catalog(
        DeleteCatalogRequest {
            name: "cat".into(),
            force: Some(true),
            ..Default::default()
        },
        user("admin"),
    )
    .await
    .unwrap();
    assert!(store.list(&uc("Catalog", "cat")).await.unwrap().is_empty());
}

#[tokio::test]
async fn renames_take_the_grants_along() {
    let (h, store, _) = world().await;
    h.update_catalog(
        UpdateCatalogRequest {
            name: "cat".into(),
            new_name: Some("kat".into()),
            ..Default::default()
        },
        user("admin"),
    )
    .await
    .unwrap();
    assert!(store.list(&uc("Catalog", "cat")).await.unwrap().is_empty());
    assert!(!store.list(&uc("Catalog", "kat")).await.unwrap().is_empty());
    // The grants inside it moved with it.
    let readers = store.list(&uc("Table", "kat.sch.t")).await.unwrap();
    assert!(
        readers
            .iter()
            .any(|g| g.subject == Subject::user("table_reader") && g.privilege == "SELECT"),
        "{readers:?}"
    );
}

#[tokio::test]
async fn an_owner_update_moves_ownership() {
    let (h, store, _) = world().await;
    h.update_catalog(
        UpdateCatalogRequest {
            name: "cat".into(),
            owner: Some("writer".into()),
            ..Default::default()
        },
        user("admin"),
    )
    .await
    .unwrap();
    let owners: Vec<Subject> = store
        .list(&uc("Catalog", "cat"))
        .await
        .unwrap()
        .into_iter()
        .filter(|g| g.privilege == OWNER)
        .map(|g| g.subject)
        .collect();
    assert_eq!(owners, [Subject::user("writer")]);
}

#[tokio::test]
async fn delta_staging_adoption_asks_whether_the_caller_reserved_it() {
    let store = Shared::default();
    let recording = Recording::new(cedar(&store));
    let h = handler(recording.clone(), &store);
    seed(&h).await;
    let at = unitycatalog_delta_api::backend::SchemaRef {
        catalog: "cat".into(),
        schema: "sch".into(),
    };
    DeltaBackend::authorize(
        &h,
        DeltaAction::CreateStaging {
            at: &at,
            name: "st",
        },
        &user("admin"),
    )
    .await
    .unwrap();
    assert!(recording.asked("CreateStagingTable", "StagingTable", "cat.sch.st"));
    let vend = DeltaBackend::authorize(
        &h,
        DeltaAction::VendTableCredential {
            table: &TableRef {
                catalog: "cat".into(),
                schema: "sch".into(),
                table: "t".into(),
            },
            access: CredentialAccess::Read,
        },
        &user("nobody"),
    )
    .await;
    assert!(
        matches!(vend, Err(DeltaBackendError::PermissionDenied(_))),
        "{vend:?}"
    );
    assert!(recording.asked("GenerateTemporaryTableCredentials", "Table", "cat.sch.t"));
}

#[tokio::test]
async fn path_credentials_are_asked_on_the_covering_location() {
    let store = Shared::default();
    let recording = Recording::new(cedar(&store));
    let h = handler(recording.clone(), &store);
    seed(&h).await;
    let result = h
        .generate_temporary_path_credentials(
            GenerateTemporaryPathCredentialsRequest {
                url: local("cat/landing"),
                operation: generate_temporary_path_credentials_request::Operation::PATH_READ.into(),
                ..Default::default()
            },
            user("nobody"),
        )
        .await;
    assert!(matches!(result, Err(Error::NotAllowed)), "{result:?}");
    assert!(recording.asked(
        "GenerateTemporaryPathCredentials",
        "ExternalLocation",
        "local"
    ));
}

#[tokio::test]
async fn an_undecidable_request_is_denied() {
    let store = Shared::default();
    let h = handler(Unreachable, &store);
    let result = h
        .get_table(
            GetTableRequest {
                full_name: "cat.sch.t".into(),
                ..Default::default()
            },
            user("admin"),
        )
        .await;
    assert!(matches!(result, Err(Error::NotAllowed)), "{result:?}");
}

#[tokio::test]
async fn an_unverified_caller_is_denied_without_asking() {
    let store = Shared::default();
    let recording = Recording::new(cedar(&store));
    let h = handler(recording.clone(), &store);
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
        assert!(matches!(result, Err(Error::NotAllowed)), "{result:?}");
    }
    assert_eq!(recording.count(), 0);
}
