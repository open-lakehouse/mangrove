//! The object-graph store against Postgres: the `location_key` property index.
//!
//! Gated behind the `integration-pg` feature; `#[sqlx::test]` provisions a fresh
//! database. Run with `DATABASE_URL` pointing at a live Postgres:
//!
//! ```sh
//! cargo test -p olai-uc-postgres --features integration-pg --test graph_store
//! ```
#![cfg(feature = "integration-pg")]
use unitycatalog_common::LocationKey;
use unitycatalog_common::models::tables::v1::Table;
use unitycatalog_common::models::{ObjectLabel, Resource};
use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};
use unitycatalog_common::store::{ObjectStoreAdapter, ResourceStore, ResourceStoreReader};

/// `ensure_property_indexes` creates the index once, is safe to run again, and
/// location searches over the indexed store find the right tables.
#[sqlx::test(migrations = false)]
async fn location_key_index(pool: sqlx::PgPool) {
    unitycatalog_postgres::unified_migrator()
        .run(&pool)
        .await
        .unwrap();
    for _ in 0..2 {
        unitycatalog_postgres::ensure_property_indexes(&pool)
            .await
            .unwrap();
    }
    let names: Vec<(String,)> = sqlx::query_as(
        "SELECT indexname FROM pg_indexes WHERE tablename = 'objects' \
         AND indexname LIKE 'objects\\_prop\\_%' ESCAPE '\\'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(names, [("objects_prop_location_key".to_string(),)]);

    let encryptor =
        EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![7u8; 32]).unwrap());
    let store = ObjectStoreAdapter::new(unitycatalog_postgres::connect_graph(pool, encryptor));
    for (name, location) in [("t1", "s3://b/data/t1"), ("t2", "s3://b/database/t2")] {
        store
            .create(
                Table {
                    name: name.into(),
                    catalog_name: "c".into(),
                    schema_name: "s".into(),
                    storage_location: Some(location.into()),
                    ..Default::default()
                }
                .into(),
            )
            .await
            .unwrap();
    }
    let key = |url: &str| LocationKey::parse(url).unwrap();
    // The three shapes the index serves: a range, an IN-list and an exact match.
    for filter in [
        key("s3a://b/data").covered_by(),
        key("s3://b/data/t1/x").covering(),
        key("s3://b/data/t1/").at(),
    ] {
        let (found, _) = store
            .search(&ObjectLabel::Table, None, &filter, None, None)
            .await
            .unwrap();
        let names: Vec<_> = found
            .into_iter()
            .map(|r| match r {
                Resource::Table(t) => t.name,
                other => panic!("expected a table, got {other:?}"),
            })
            .collect();
        assert_eq!(names, ["t1"]);
    }
}
