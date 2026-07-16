//! Seed a running UC server with a few catalog-**managed** Delta tables holding
//! interesting sample data — the data-writing half of the `just ui-dev` flow.
//!
//! Unlike [`managed_table_azurite`](./managed_table_azurite.rs), this example only
//! **writes** (create + append); it never reads back. That is deliberate: the
//! read-back path assembles a snapshot through a synchronous delta-kernel engine
//! and panics with "Cannot start a runtime from within a runtime" when driven
//! from inside a Tokio runtime. Seeding does not need to read — the UI (and its
//! in-browser wasm query preview) is the reader — so we skip it entirely and the
//! panic never fires.
//!
//! It drives the same public `create_managed_table` / `append_to_managed_table`
//! APIs the server's own write path uses, so every store is built from a vended
//! credential and it is cloud-agnostic (works against Azurite, S3, or real Azure).
//!
//! ## Usage
//!
//! The `ui-dev` recipe invokes this after the backend is up and the catalog +
//! schema are seeded. Standalone:
//!
//! ```bash
//! UC_ENDPOINT=http://localhost:8080/api/2.1/unity-catalog/ \
//! UC_CATALOG=demo UC_SCHEMA=default \
//! cargo run -p olai-uc-datafusion --features delta --example seed_managed_tables
//! ```
//!
//! Env (defaults shown): `UC_ENDPOINT` (…:8080…), `UC_CATALOG=demo`,
//! `UC_SCHEMA=default`, optional `UC_TOKEN` (unauthenticated dev server otherwise).

use std::sync::Arc;

use datafusion::arrow::array::{Float64Array, Int64Array, RecordBatch, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion_unitycatalog::managed::{append_to_managed_table, create_managed_table};
use unitycatalog_object_store::UnityObjectStoreFactory;

type BoxError = Box<dyn std::error::Error>;

const ENGINE_INFO: &str = "seed_managed_tables-example/0.1";

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .init();

    let endpoint = std::env::var("UC_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:8080/api/2.1/unity-catalog/".to_string());
    let catalog = std::env::var("UC_CATALOG").unwrap_or_else(|_| "demo".to_string());
    let schema = std::env::var("UC_SCHEMA").unwrap_or_else(|_| "default".to_string());

    let mut builder = UnityObjectStoreFactory::builder().with_uri(endpoint);
    match std::env::var("UC_TOKEN") {
        Ok(token) => builder = builder.with_token(token),
        Err(_) => builder = builder.with_allow_unauthenticated(true),
    }
    if let Ok(region) = std::env::var("AWS_REGION") {
        builder = builder.with_aws_region(region);
    }
    let factory = Arc::new(builder.build().await?);
    let client = Arc::new(factory.unity_client().delta_v1());

    for table in tables() {
        println!("seeding {catalog}.{schema}.{}…", table.name);
        match create_managed_table(
            client.clone(),
            &catalog,
            &schema,
            table.name,
            table.batch.schema(),
            table.partition_columns.clone(),
            ENGINE_INFO,
        )
        .await
        {
            Ok(_) => {}
            // Idempotent: a table left over from a prior run is fine — skip it
            // rather than fail the whole seed. (The ephemeral `ui-dev` DB starts
            // empty, but standalone re-runs and other flows may not.)
            Err(e) if is_already_exists(&e) => {
                println!("  already exists — skipping");
                continue;
            }
            Err(e) => return Err(e.into()),
        }
        let version = append_to_managed_table(
            factory.clone(),
            &catalog,
            &schema,
            table.name,
            table.batch.clone(),
            ENGINE_INFO,
        )
        .await?;
        println!(
            "  created + appended {} rows (version {version})",
            table.batch.num_rows()
        );
    }

    println!("\nSeeded {catalog}.{schema} with managed tables. Browse them in the UI.");
    Ok(())
}

/// Whether a create failed only because the table already exists (409 /
/// `AlreadyExistsException`). The connector surfaces this as an opaque client
/// error, so we match on its rendered message — adequate for a dev seed.
fn is_already_exists(err: &datafusion_unitycatalog::managed::CreateManagedTableError) -> bool {
    let msg = err.to_string();
    msg.contains("Already exists") || msg.contains("AlreadyExists") || msg.contains("409")
}

/// A managed table to seed: a name, a single batch of sample rows, and its
/// partition columns (empty for unpartitioned).
struct SeedTable {
    name: &'static str,
    batch: RecordBatch,
    partition_columns: Vec<String>,
}

/// The demo tables. Kept small but varied (distinct column types, a nullable
/// column) so the UI has something interesting to render.
fn tables() -> Vec<SeedTable> {
    vec![
        SeedTable {
            name: "customers",
            partition_columns: vec![],
            batch: RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new("id", DataType::Int64, false),
                    Field::new("name", DataType::Utf8, false),
                    Field::new("city", DataType::Utf8, true),
                ])),
                vec![
                    Arc::new(Int64Array::from(vec![1, 2, 3, 4, 5])),
                    Arc::new(StringArray::from(vec![
                        "Ada Lovelace",
                        "Alan Turing",
                        "Grace Hopper",
                        "Katherine Johnson",
                        "Edsger Dijkstra",
                    ])),
                    Arc::new(StringArray::from(vec![
                        Some("London"),
                        Some("Manchester"),
                        Some("New York"),
                        None,
                        Some("Amsterdam"),
                    ])),
                ],
            )
            .expect("customers batch"),
        },
        SeedTable {
            name: "orders",
            // Unpartitioned: `append_to_managed_table` writes through the
            // non-partitioned kernel write context, so a partitioned managed
            // table can be created but not appended to via this path. `region`
            // stays a regular column — plenty varied for the UI.
            partition_columns: vec![],
            batch: RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new("order_id", DataType::Int64, false),
                    Field::new("customer_id", DataType::Int64, false),
                    Field::new("amount", DataType::Float64, false),
                    Field::new("region", DataType::Utf8, false),
                ])),
                vec![
                    Arc::new(Int64Array::from(vec![100, 101, 102, 103, 104, 105])),
                    Arc::new(Int64Array::from(vec![1, 2, 1, 3, 5, 2])),
                    Arc::new(Float64Array::from(vec![
                        19.99, 5.50, 120.00, 42.10, 8.75, 63.00,
                    ])),
                    Arc::new(StringArray::from(vec!["EU", "EU", "US", "US", "EU", "EU"])),
                ],
            )
            .expect("orders batch"),
        },
    ]
}
