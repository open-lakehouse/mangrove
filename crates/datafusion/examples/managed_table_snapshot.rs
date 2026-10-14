//! Read a Unity Catalog **catalog-managed** Delta table by assembling the read
//! from its building blocks — the lower-level counterpart to the resolver-driven
//! `unity_catalog` example.
//!
//! A *catalog-managed* (coordinated-commit) Delta table is one where the catalog,
//! not the filesystem `_delta_log/`, is the source of truth for the latest
//! version. The newest commits are ratified by the catalog but may not yet be
//! backfilled into `_delta_log/` — they live as staged commits under
//! `_delta_log/_staged_commits/`. The `/delta/v1` `loadTable` endpoint hands a
//! reader exactly what it needs to materialize the ratified snapshot: the table
//! type, the unbackfilled commit tail, and the latest ratified version.
//!
//! This example shows that read assembled by hand, in two stages:
//!
//! **Stage 1 — build the catalog-managed snapshot.** Call `loadTable`, confirm
//! the table is `MANAGED`, vend a credentialed object store for its location, and
//! call [`build_catalog_managed_snapshot`] with the commit tail + latest version.
//! Printing the resulting snapshot version proves it was assembled from the
//! catalog's ratified state rather than a filesystem scan.
//!
//! **Stage 2 — scan it.** Wrap the snapshot in a `DeltaScanNext` provider, register
//! it on the session, and run `SELECT * FROM …`.
//!
//! # Prerequisite: a populated managed table
//!
//! This crate has no Delta *write* path yet (the `/delta/v1` write/commit surface
//! is a follow-up), so the example does not seed data — it assumes a managed table
//! already exists and is populated. The simplest way to get one locally is the
//! open-source **Java** Unity Catalog server, which supports managed tables:
//!
//! ```text
//! docker compose -f dev/uc-oss.compose.yaml up -d --wait
//! ```
//!
//! then register and populate a managed Delta table against it with an external
//! Delta writer (e.g. Delta Spark / delta-rs driving the staging-table + commit
//! flow).
//!
//! # Run
//!
//! ```text
//! UC_ENDPOINT=http://localhost:8080/api/2.1/unity-catalog/ \
//! UC_TABLE=unity.default.my_managed_table \
//! cargo run -p datafusion-unitycatalog --features delta --example managed_table_snapshot
//! ```
//!
//! Set `UC_TOKEN` for an authenticated server (omit it for a local unauthenticated
//! OSS server) and `AWS_REGION` if the storage backend needs it.
//!
//! [`build_catalog_managed_snapshot`]: datafusion_unitycatalog::catalog::build_catalog_managed_snapshot

use std::sync::Arc;

use datafusion::arrow::util::pretty::print_batches;
use datafusion::common::TableReference;
use datafusion::prelude::SessionContext;
use datafusion_unitycatalog::RoutingObjectStore;
use datafusion_unitycatalog::catalog::{
    ManagedReadState, build_catalog_managed_snapshot, resolve_managed_read_state,
};
use deltalake_core::delta_datafusion::DeltaScanNext;
use deltalake_core::delta_datafusion::engine::DataFusionEngine;
use deltalake_core::logstore::{StorageConfig, default_logstore};
use object_store::path::Path;
use unitycatalog_object_store::{TableOperation, UnityObjectStoreFactory};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let endpoint = std::env::var("UC_ENDPOINT").expect(
        "set UC_ENDPOINT to your Unity Catalog instance, e.g. http://localhost:8080/api/2.1/unity-catalog/",
    );
    let full_name = std::env::var("UC_TABLE")
        .expect("set UC_TABLE to a managed table's three-level name, e.g. unity.default.my_table");
    let reference = TableReference::parse_str(&full_name);
    let catalog = reference
        .catalog()
        .expect("UC_TABLE must be fully qualified");
    let schema = reference
        .schema()
        .expect("UC_TABLE must be fully qualified");
    let table = reference.table();

    // Build the UC object store factory used for both the `/delta/v1` metadata
    // lookup and credential vending. Auth mirrors the integration tests: a bearer
    // token if present, otherwise allow an unauthenticated local server.
    let mut builder = UnityObjectStoreFactory::builder().with_uri(endpoint);
    match std::env::var("UC_TOKEN") {
        Ok(token) => builder = builder.with_token(token),
        Err(_) => builder = builder.with_allow_unauthenticated(true),
    }
    if let Ok(region) = std::env::var("AWS_REGION") {
        builder = builder.with_aws_region(region);
    }
    let factory = builder.build().await?;

    // ===================================================================
    // Stage 1 — build the catalog-managed snapshot.
    // ===================================================================

    // Ask the catalog for the table's type, its unbackfilled commit tail, and the
    // latest ratified version. For a managed table this is the authoritative state.
    let loaded = factory
        .unity_client()
        .delta_v1()
        .load_table(catalog, schema, table)
        .await?;

    // Resolve how to read the table from the loadTable response: this validates the
    // latest ratified version (never substituting metadata.last-commit-version) and
    // tells us whether the table is catalog-managed.
    let (commits, latest) = match resolve_managed_read_state(&loaded)? {
        ManagedReadState::Managed { commits, latest } => (commits, latest),
        ManagedReadState::NotManaged => {
            return Err(format!(
                "{full_name} is not catalog-managed — this example demonstrates the \
                 catalog-managed read path; point UC_TABLE at a managed table"
            )
            .into());
        }
    };
    println!(
        "loadTable {full_name}: latest_table_version={latest} commit_tail={}",
        commits.len(),
    );

    // Vend a credentialed object store for the table's storage location, and
    // register it on the session runtime the way the async resolver does. The
    // resolver's helper is crate-private, so we replicate it here — which also
    // shows *why* the routing store exists: DataFusion's object store registry
    // keys on `scheme://host` only, but UC credentials are scoped to a sub-path,
    // so two tables in one bucket would collide on a single registry entry. A
    // `RoutingObjectStore` sits behind the bucket key and dispatches by path.
    let ctx = SessionContext::new();
    let uc_store = factory
        .for_table(full_name.clone(), TableOperation::Read)
        .await?;
    let location = uc_store.url().clone();

    let bucket_key = format!(
        "{}://{}",
        location.scheme(),
        &location[url::Position::BeforeHost..url::Position::AfterPort]
    );
    let router = RoutingObjectStore::new();
    router.register(
        Path::from_url_path(location.path()).unwrap_or_default(),
        uc_store.root(),
    );
    ctx.runtime_env()
        .register_object_store(&Url::parse(&format!("{bucket_key}/"))?, Arc::new(router));

    // Build the engine + log store against the registered store, then assemble the
    // snapshot from the catalog's ratified commit tail (rather than scanning
    // `_delta_log/`). `at_version = None` targets the latest ratified version.
    //
    // Build the log store directly from the registered store via
    // `default_logstore` rather than `logstore_with`: the latter dispatches on
    // the URL scheme to a registered logstore factory, but we depend only on
    // `deltalake-core`, so no cloud-scheme factories (`s3`/`gs`/`az`) exist. The
    // prefixed store roots paths at the table location; the root store stays
    // bucket-rooted.
    let root_store = ctx
        .runtime_env()
        .object_store_registry
        .get_store(&location)?;
    let config = StorageConfig::default();
    let prefixed_store = config.decorate_store(root_store.clone(), &location)?;
    let log_store = default_logstore(Arc::from(prefixed_store), root_store, &location, &config);
    let engine = DataFusionEngine::new_from_context(ctx.task_ctx());

    let snapshot =
        build_catalog_managed_snapshot(engine.clone(), &location, &commits, latest as i64, None)
            .await?;
    println!(
        "built catalog-managed snapshot for {full_name} at version {}",
        snapshot.version()
    );

    // ===================================================================
    // Stage 2 — scan the snapshot.
    // ===================================================================

    let provider = DeltaScanNext::builder()
        .with_snapshot(Arc::new(snapshot))
        .with_log_store(log_store)
        .await?;

    // Register under a single-part name in the session's default catalog/schema
    // and query that. Registering the three-level UC name directly would require
    // the `demo`/`managed_demo` catalog+schema to exist in the session — that
    // catalog wiring is the resolver's job (see the `unity_catalog` example);
    // here we only want to scan the snapshot we just built.
    ctx.register_table(table, provider)?;
    let df = ctx.sql(&format!("SELECT * FROM {table}")).await?;
    let batches = df.collect().await?;

    let rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    println!("scanned {rows} rows from {full_name} (registered as `{table}`)");
    print_batches(&batches)?;

    Ok(())
}
