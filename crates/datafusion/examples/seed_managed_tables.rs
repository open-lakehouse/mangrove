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
//! It seeds several managed tables spread across multiple catalogs and schemas
//! (each [`SeedTable`] names its own `catalog`/`schema`), so the browser has a
//! non-trivial namespace tree to explore. The catalogs and schemas themselves
//! (and the other metadata-only entities — volumes, functions, models) are
//! created ahead of this example by `dev/scripts/seed-azurite.sh`.
//!
//! ## Usage
//!
//! The `ui-dev` recipe invokes this after the backend is up and the catalogs +
//! schemas are seeded. Standalone:
//!
//! ```bash
//! UC_ENDPOINT=http://localhost:8080/api/2.1/unity-catalog/ \
//! cargo run -p olai-uc-datafusion --features delta --example seed_managed_tables
//! ```
//!
//! Env (defaults shown): `UC_ENDPOINT` (…:8080…), optional `UC_TOKEN`
//! (unauthenticated dev server otherwise). The target catalog/schema of each
//! table is fixed by the internal [`tables`] list, not by env.

use std::sync::Arc;

use datafusion::arrow::array::{
    BooleanArray, Date32Array, Float64Array, Int64Array, RecordBatch, StringArray,
    TimestampMicrosecondArray,
};
use datafusion::arrow::datatypes::{DataType, Field, Schema, TimeUnit};
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
        let SeedTable {
            catalog,
            schema,
            name,
            batches,
            partition_columns,
        } = &table;
        println!("seeding {catalog}.{schema}.{name}…");
        let arrow_schema = batches
            .first()
            .expect("every seed table has at least one batch")
            .schema();
        match create_managed_table(
            client.clone(),
            catalog,
            schema,
            name,
            arrow_schema,
            partition_columns.clone(),
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
        // Each batch is its own append → its own Delta commit, so tables seeded
        // with multiple batches show real history in the Delta-log explorer.
        for (i, batch) in batches.iter().enumerate() {
            let version = append_to_managed_table(
                factory.clone(),
                catalog,
                schema,
                name,
                batch.clone(),
                ENGINE_INFO,
            )
            .await?;
            println!(
                "  append {}/{}: {} rows (version {version})",
                i + 1,
                batches.len(),
                batch.num_rows()
            );
        }
    }

    println!("\nSeeded managed tables across multiple catalogs/schemas. Browse them in the UI.");
    Ok(())
}

/// Whether a create failed only because the table already exists (409 /
/// `AlreadyExistsException`). The connector surfaces this as an opaque client
/// error, so we match on its rendered message — adequate for a dev seed.
fn is_already_exists(err: &datafusion_unitycatalog::managed::CreateManagedTableError) -> bool {
    let msg = err.to_string();
    msg.contains("Already exists") || msg.contains("AlreadyExists") || msg.contains("409")
}

/// A managed table to seed: its fully-qualified location, one-or-more batches of
/// sample rows (each appended as its own Delta commit), and its partition columns.
///
/// Every table here is **unpartitioned** (`partition_columns` empty):
/// `append_to_managed_table` writes through the non-partitioned kernel write
/// context, so a partitioned managed table can be created but not appended to via
/// this path. Richness comes from varied column types, nullable columns, and
/// multi-commit history instead.
struct SeedTable {
    catalog: &'static str,
    schema: &'static str,
    name: &'static str,
    batches: Vec<RecordBatch>,
    partition_columns: Vec<String>,
}

impl SeedTable {
    /// A single-commit table in the given namespace.
    fn single(
        catalog: &'static str,
        schema: &'static str,
        name: &'static str,
        batch: RecordBatch,
    ) -> Self {
        Self {
            catalog,
            schema,
            name,
            batches: vec![batch],
            partition_columns: vec![],
        }
    }
}

/// A `Timestamp(Microsecond, "UTC")` field/array helper — takes epoch-micros.
fn ts_field(name: &str, nullable: bool) -> Field {
    Field::new(
        name,
        DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
        nullable,
    )
}
fn ts_array(micros: Vec<i64>) -> Arc<TimestampMicrosecondArray> {
    Arc::new(TimestampMicrosecondArray::from(micros).with_timezone("UTC"))
}

/// The demo tables, spread across two catalogs (`demo`, `ml`) and several
/// schemas. Kept small but varied — distinct column types (int/float/utf8/
/// bool/date/timestamp), nullable columns, and one multi-commit table — so the
/// UI's list/detail/preview/Delta-log surfaces all have something to render.
fn tables() -> Vec<SeedTable> {
    vec![
        // ── demo.default ────────────────────────────────────────────────────
        SeedTable::single(
            "demo",
            "default",
            "customers",
            RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new("id", DataType::Int64, false),
                    Field::new("name", DataType::Utf8, false),
                    Field::new("city", DataType::Utf8, true),
                    Field::new("signup_date", DataType::Date32, false),
                    Field::new("active", DataType::Boolean, false),
                    Field::new("lifetime_value", DataType::Float64, true),
                ])),
                vec![
                    Arc::new(Int64Array::from(vec![1, 2, 3, 4, 5, 6, 7, 8])),
                    Arc::new(StringArray::from(vec![
                        "Ada Lovelace",
                        "Alan Turing",
                        "Grace Hopper",
                        "Katherine Johnson",
                        "Edsger Dijkstra",
                        "Barbara Liskov",
                        "Donald Knuth",
                        "Margaret Hamilton",
                    ])),
                    Arc::new(StringArray::from(vec![
                        Some("London"),
                        Some("Manchester"),
                        Some("New York"),
                        None,
                        Some("Amsterdam"),
                        Some("Boston"),
                        Some("Stanford"),
                        None,
                    ])),
                    // Days since the Unix epoch (Date32).
                    Arc::new(Date32Array::from(vec![
                        19000, 19010, 19125, 19200, 19277, 19300, 19412, 19500,
                    ])),
                    Arc::new(BooleanArray::from(vec![
                        true, true, false, true, true, false, true, true,
                    ])),
                    Arc::new(Float64Array::from(vec![
                        Some(1240.50),
                        Some(88.00),
                        Some(3125.75),
                        None,
                        Some(540.10),
                        Some(2000.00),
                        None,
                        Some(410.25),
                    ])),
                ],
            )
            .expect("customers batch"),
        ),
        SeedTable::single(
            "demo",
            "default",
            "orders",
            RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new("order_id", DataType::Int64, false),
                    Field::new("customer_id", DataType::Int64, false),
                    Field::new("amount", DataType::Float64, false),
                    Field::new("region", DataType::Utf8, false),
                    ts_field("order_ts", false),
                ])),
                vec![
                    Arc::new(Int64Array::from(vec![
                        100, 101, 102, 103, 104, 105, 106, 107, 108,
                    ])),
                    Arc::new(Int64Array::from(vec![1, 2, 1, 3, 5, 2, 6, 7, 1])),
                    Arc::new(Float64Array::from(vec![
                        19.99, 5.50, 120.00, 42.10, 8.75, 63.00, 250.00, 12.49, 77.77,
                    ])),
                    Arc::new(StringArray::from(vec![
                        "EU", "EU", "US", "US", "EU", "EU", "US", "APAC", "EU",
                    ])),
                    ts_array(vec![
                        1_700_000_000_000_000,
                        1_700_100_000_000_000,
                        1_700_200_000_000_000,
                        1_700_300_000_000_000,
                        1_700_400_000_000_000,
                        1_700_500_000_000_000,
                        1_700_600_000_000_000,
                        1_700_700_000_000_000,
                        1_700_800_000_000_000,
                    ]),
                ],
            )
            .expect("orders batch"),
        ),
        SeedTable::single(
            "demo",
            "default",
            "products",
            RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new("sku", DataType::Utf8, false),
                    Field::new("name", DataType::Utf8, false),
                    Field::new("price", DataType::Float64, false),
                    Field::new("in_stock", DataType::Boolean, false),
                    Field::new("weight_kg", DataType::Float64, true),
                ])),
                vec![
                    Arc::new(StringArray::from(vec![
                        "SKU-001", "SKU-002", "SKU-003", "SKU-004", "SKU-005", "SKU-006",
                        "SKU-007", "SKU-008", "SKU-009", "SKU-010",
                    ])),
                    Arc::new(StringArray::from(vec![
                        "Widget", "Gadget", "Sprocket", "Cog", "Flange", "Grommet", "Bushing",
                        "Washer", "Bearing", "Gasket",
                    ])),
                    Arc::new(Float64Array::from(vec![
                        9.99, 14.50, 3.25, 1.10, 22.00, 0.75, 5.40, 0.30, 18.60, 2.15,
                    ])),
                    Arc::new(BooleanArray::from(vec![
                        true, true, false, true, true, true, false, true, true, false,
                    ])),
                    Arc::new(Float64Array::from(vec![
                        Some(0.25),
                        Some(0.40),
                        None,
                        Some(0.05),
                        Some(1.20),
                        None,
                        Some(0.15),
                        Some(0.02),
                        Some(0.90),
                        None,
                    ])),
                ],
            )
            .expect("products batch"),
        ),
        // A multi-commit table: three appends → three Delta versions, so the
        // Delta-log explorer shows real history and multiple data files.
        SeedTable {
            catalog: "demo",
            schema: "default",
            name: "events",
            partition_columns: vec![],
            batches: vec![
                events_batch(&[
                    (1, 10, "login", 1_700_000_000_000_000),
                    (2, 11, "view", 1_700_000_100_000_000),
                    (3, 10, "click", 1_700_000_200_000_000),
                ]),
                events_batch(&[
                    (4, 12, "login", 1_700_100_000_000_000),
                    (5, 11, "purchase", 1_700_100_100_000_000),
                ]),
                events_batch(&[
                    (6, 13, "login", 1_700_200_000_000_000),
                    (7, 10, "logout", 1_700_200_100_000_000),
                    (8, 12, "view", 1_700_200_200_000_000),
                    (9, 13, "click", 1_700_200_300_000_000),
                ]),
            ],
        },
        // ── demo.sales ──────────────────────────────────────────────────────
        SeedTable::single(
            "demo",
            "sales",
            "regions",
            RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new("code", DataType::Utf8, false),
                    Field::new("name", DataType::Utf8, false),
                    Field::new("lead", DataType::Utf8, true),
                ])),
                vec![
                    Arc::new(StringArray::from(vec!["EU", "US", "APAC"])),
                    Arc::new(StringArray::from(vec![
                        "Europe",
                        "United States",
                        "Asia-Pacific",
                    ])),
                    Arc::new(StringArray::from(vec![Some("Sofia"), Some("Marcus"), None])),
                ],
            )
            .expect("regions batch"),
        ),
        SeedTable::single(
            "demo",
            "sales",
            "daily_revenue",
            RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new("day", DataType::Date32, false),
                    Field::new("region", DataType::Utf8, false),
                    Field::new("revenue", DataType::Float64, false),
                ])),
                vec![
                    Arc::new(Date32Array::from(vec![
                        19500, 19500, 19501, 19501, 19502, 19502,
                    ])),
                    Arc::new(StringArray::from(vec!["EU", "US", "EU", "US", "EU", "US"])),
                    Arc::new(Float64Array::from(vec![
                        12500.0, 22300.5, 11890.25, 24100.0, 13020.75, 21750.5,
                    ])),
                ],
            )
            .expect("daily_revenue batch"),
        ),
        // ── ml.features ─────────────────────────────────────────────────────
        SeedTable::single(
            "ml",
            "features",
            "feature_store",
            RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new("entity_id", DataType::Int64, false),
                    Field::new("feature_a", DataType::Float64, false),
                    Field::new("feature_b", DataType::Float64, true),
                    ts_field("updated_at", false),
                ])),
                vec![
                    Arc::new(Int64Array::from(vec![1, 2, 3, 4, 5])),
                    Arc::new(Float64Array::from(vec![0.12, 0.88, 0.45, 0.67, 0.30])),
                    Arc::new(Float64Array::from(vec![
                        Some(1.5),
                        None,
                        Some(2.7),
                        Some(0.9),
                        None,
                    ])),
                    ts_array(vec![
                        1_700_000_000_000_000,
                        1_700_050_000_000_000,
                        1_700_100_000_000_000,
                        1_700_150_000_000_000,
                        1_700_200_000_000_000,
                    ]),
                ],
            )
            .expect("feature_store batch"),
        ),
    ]
}

/// Build one `events` batch from `(event_id, user_id, kind, ts_micros)` tuples.
fn events_batch(rows: &[(i64, i64, &str, i64)]) -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("event_id", DataType::Int64, false),
            Field::new("user_id", DataType::Int64, false),
            Field::new("kind", DataType::Utf8, false),
            ts_field("ts", false),
        ])),
        vec![
            Arc::new(Int64Array::from(
                rows.iter().map(|r| r.0).collect::<Vec<_>>(),
            )),
            Arc::new(Int64Array::from(
                rows.iter().map(|r| r.1).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter().map(|r| r.2).collect::<Vec<_>>(),
            )),
            ts_array(rows.iter().map(|r| r.3).collect::<Vec<_>>()),
        ],
    )
    .expect("events batch")
}
