//! Integration tests for the embedded SQLite store.
//!
//! These cover the runtime correctness of the ported SQL (the offline `.sqlx`
//! cache validates the queries at compile time, but not their behavior). Each
//! test opens an isolated temp-file database so persistence across reopen can be
//! exercised; the helper deletes the file (and WAL sidecars) on drop.

use std::path::PathBuf;

use bytes::Bytes;
use olai_store::name::ResourceName;
use olai_store::{
    AssociationStore, AssociationStoreReader, EdgeQuery, ObjectStore, ObjectStoreReader,
    Precondition,
};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};
use unitycatalog_common::services::secrets::SecretManager;
use unitycatalog_sqlite::SqliteStore;

/// A temp-file SQLite path that cleans up its files on drop.
struct TempDb {
    path: PathBuf,
}

impl TempDb {
    fn new(tag: &str) -> Self {
        let mut path = std::env::temp_dir();
        // Unique-enough per test without needing `Math.random`: tag + pid + nanos.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!(
            "uc-sqlite-test-{tag}-{}-{nanos}.db",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        TempDb { path }
    }

    fn path(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", self.path.display()));
        }
    }
}

fn encryptor() -> EnvelopeEncryptor {
    EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap())
}

async fn store(temp: &TempDb) -> SqliteStore {
    let store = SqliteStore::connect(temp.path(), encryptor())
        .await
        .expect("connect");
    store.migrate().await.expect("migrate");
    store
}

fn name(parts: &[&str]) -> ResourceName {
    ResourceName::new(parts.iter().map(|s| s.to_string()))
}

#[tokio::test]
async fn create_get_by_id_and_name() {
    let temp = TempDb::new("crud");
    let s = store(&temp).await;

    let props = serde_json::json!({"comment": "hi"});
    let created = ObjectStore::create(
        &s,
        ObjectLabel::Catalog,
        &name(&["main"]),
        Some(props.clone()),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(created.label, ObjectLabel::Catalog);
    assert_eq!(created.name, name(&["main"]));
    assert_eq!(created.properties, Some(props));

    let by_id = ObjectStoreReader::get(&s, &created.id).await.unwrap();
    assert_eq!(by_id.id, created.id);

    let by_name = ObjectStoreReader::get_by_name(&s, ObjectLabel::Catalog, &name(&["main"]))
        .await
        .unwrap();
    assert_eq!(by_name.id, created.id);
}

#[tokio::test]
async fn duplicate_name_is_already_exists() {
    let temp = TempDb::new("dup");
    let s = store(&temp).await;

    ObjectStore::create(&s, ObjectLabel::Catalog, &name(&["main"]), None, None, None)
        .await
        .unwrap();
    let err = ObjectStore::create(&s, ObjectLabel::Catalog, &name(&["main"]), None, None, None)
        .await
        .unwrap_err();
    assert!(
        matches!(err, olai_store::Error::AlreadyExists),
        "expected AlreadyExists, got {err:?}"
    );
}

#[tokio::test]
async fn update_and_delete() {
    let temp = TempDb::new("update");
    let s = store(&temp).await;

    let created = ObjectStore::create(&s, ObjectLabel::Catalog, &name(&["main"]), None, None, None)
        .await
        .unwrap();
    let updated = ObjectStore::update(
        &s,
        &created.id,
        Some(serde_json::json!({"k": "v"})),
        Precondition::Any,
        None,
    )
    .await
    .unwrap();
    assert_eq!(updated.properties, Some(serde_json::json!({"k": "v"})));
    assert!(updated.updated_at.is_some());

    ObjectStore::delete(&s, &created.id).await.unwrap();
    let err = ObjectStoreReader::get(&s, &created.id).await.unwrap_err();
    assert!(matches!(err, olai_store::Error::NotFound));
}

#[tokio::test]
async fn list_by_namespace_with_pagination() {
    let temp = TempDb::new("list");
    let s = store(&temp).await;

    // Two schemas in catalog `main`, one in catalog `other`.
    for ns_schema in ["a", "b"] {
        ObjectStore::create(
            &s,
            ObjectLabel::Schema,
            &name(&["main", ns_schema]),
            None,
            None,
            None,
        )
        .await
        .unwrap();
    }
    ObjectStore::create(
        &s,
        ObjectLabel::Schema,
        &name(&["other", "c"]),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Listing schemas under namespace `main` returns exactly the two children.
    let (page, token) =
        ObjectStoreReader::list(&s, ObjectLabel::Schema, Some(&name(&["main"])), None, None)
            .await
            .unwrap();
    assert_eq!(page.len(), 2);
    assert!(token.is_none());
    for obj in &page {
        assert_eq!(obj.name.path()[0], "main");
    }

    // Page size 1 yields a token; following it returns the rest.
    let (page1, token1) = ObjectStoreReader::list(
        &s,
        ObjectLabel::Schema,
        Some(&name(&["main"])),
        Some(1),
        None,
    )
    .await
    .unwrap();
    assert_eq!(page1.len(), 1);
    let token1 = token1.expect("expected a next-page token");
    let (page2, _) = ObjectStoreReader::list(
        &s,
        ObjectLabel::Schema,
        Some(&name(&["main"])),
        Some(1),
        Some(token1),
    )
    .await
    .unwrap();
    assert_eq!(page2.len(), 1);
    assert_ne!(page1[0].id, page2[0].id);
}

#[tokio::test]
async fn associations_create_inverse_and_cascade() {
    let temp = TempDb::new("assoc");
    let s = store(&temp).await;

    let catalog = ObjectStore::create(&s, ObjectLabel::Catalog, &name(&["main"]), None, None, None)
        .await
        .unwrap();
    let schema = ObjectStore::create(
        &s,
        ObjectLabel::Schema,
        &name(&["main", "s"]),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // parent_of catalog -> schema; the inverse child_of is auto-created.
    AssociationStore::add(&s, catalog.id, schema.id, "parent_of", None)
        .await
        .unwrap();

    let (children, _) =
        AssociationStoreReader::query_edges(&s, EdgeQuery::from(catalog.id, "parent_of"))
            .await
            .unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].to_id, schema.id);
    assert_eq!(children[0].to_label, ObjectLabel::Schema);

    let (parents, _) =
        AssociationStoreReader::query_edges(&s, EdgeQuery::from(schema.id, "child_of"))
            .await
            .unwrap();
    assert_eq!(parents.len(), 1);
    assert_eq!(parents[0].to_id, catalog.id);

    // target_label filter excludes non-matching target types.
    let (filtered, _) = AssociationStoreReader::query_edges(
        &s,
        EdgeQuery::from(catalog.id, "parent_of").target_label(ObjectLabel::Table),
    )
    .await
    .unwrap();
    assert_eq!(filtered.len(), 0);

    // Deleting the catalog cascades its association rows away.
    ObjectStore::delete(&s, &catalog.id).await.unwrap();
    let (after, _) =
        AssociationStoreReader::query_edges(&s, EdgeQuery::from(schema.id, "child_of"))
            .await
            .unwrap();
    assert_eq!(after.len(), 0);
}

#[tokio::test]
async fn remove_association_removes_inverse() {
    let temp = TempDb::new("assoc-remove");
    let s = store(&temp).await;

    let a = ObjectStore::create(&s, ObjectLabel::Catalog, &name(&["a"]), None, None, None)
        .await
        .unwrap();
    let b = ObjectStore::create(
        &s,
        ObjectLabel::Schema,
        &name(&["a", "b"]),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    AssociationStore::add(&s, a.id, b.id, "parent_of", None)
        .await
        .unwrap();
    AssociationStore::remove(&s, a.id, b.id, "parent_of")
        .await
        .unwrap();

    let (children, _) = AssociationStoreReader::query_edges(&s, EdgeQuery::from(a.id, "parent_of"))
        .await
        .unwrap();
    assert_eq!(children.len(), 0);
    let (parents, _) = AssociationStoreReader::query_edges(&s, EdgeQuery::from(b.id, "child_of"))
        .await
        .unwrap();
    assert_eq!(parents.len(), 0);
}

#[tokio::test]
async fn secrets_round_trip() {
    let temp = TempDb::new("secrets");
    let s = store(&temp).await;

    s.put_secret("token", Bytes::from_static(b"sekret"))
        .await
        .unwrap();
    let got = s.get_secret("token").await.unwrap();
    assert_eq!(got, Bytes::from_static(b"sekret"));

    // Upsert overwrites.
    s.put_secret("token", Bytes::from_static(b"rotated"))
        .await
        .unwrap();
    assert_eq!(
        s.get_secret("token").await.unwrap(),
        Bytes::from_static(b"rotated")
    );

    s.delete_secret("token").await.unwrap();
    assert!(s.get_secret("token").await.is_err());
    assert!(s.delete_secret("token").await.is_err());
}

#[tokio::test]
async fn data_persists_across_reopen() {
    let temp = TempDb::new("persist");
    let created_id = {
        let s = store(&temp).await;
        let c = ObjectStore::create(&s, ObjectLabel::Catalog, &name(&["main"]), None, None, None)
            .await
            .unwrap();
        s.put_secret("k", Bytes::from_static(b"v")).await.unwrap();
        c.id
    };

    // Reopen the same file: the catalog and secret survive.
    let s2 = store(&temp).await;
    let reread = ObjectStoreReader::get(&s2, &created_id).await.unwrap();
    assert_eq!(reread.name, name(&["main"]));
    assert_eq!(s2.get_secret("k").await.unwrap(), Bytes::from_static(b"v"));
}

#[tokio::test]
async fn name_matching_is_ascii_case_insensitive() {
    // Documents the NOCASE parity gap: ASCII case folding matches Postgres,
    // so `MAIN` and `main` collide on the unique (label, name) constraint.
    let temp = TempDb::new("nocase");
    let s = store(&temp).await;

    ObjectStore::create(&s, ObjectLabel::Catalog, &name(&["main"]), None, None, None)
        .await
        .unwrap();
    let err = ObjectStore::create(&s, ObjectLabel::Catalog, &name(&["MAIN"]), None, None, None)
        .await
        .unwrap_err();
    assert!(
        matches!(err, olai_store::Error::AlreadyExists),
        "ASCII case-insensitive collision expected, got {err:?}"
    );

    // And lookup by a differently-cased name finds the original.
    let found = ObjectStoreReader::get_by_name(&s, ObjectLabel::Catalog, &name(&["Main"]))
        .await
        .unwrap();
    assert_eq!(found.name, name(&["main"]));
}
