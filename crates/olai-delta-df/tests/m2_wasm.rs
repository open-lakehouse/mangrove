//! M2 verification: the DV-free SSA compile + Load path compiles **and runs on
//! `wasm32-unknown-unknown`**, driven under `wasm-bindgen-futures` (headless, Node runner) —
//! no tokio, no `spawn_blocking`, no local filesystem. See
//! `handover-wasm-async-native-table-provider.md`.
//!
//! This proves the wasm-critical pieces at the crate boundary:
//!   * `compile_ssa` lowers a hand-built `ResultPlan` on wasm, and
//!   * a `NodeKind::Load` reads parquet bytes back out of an in-memory `object_store` over
//!     DataFusion's async parquet source — the same code path a browser scan takes, minus the
//!     network fetch (which the `deltalake-wasm` facade + `UcFetchStore` supply in production,
//!     exercised by the M3 browser smoke test).
//!
//! This test drives the engine-free planning + Load execution directly — the part that previously
//! forced the inline executor. Snapshot *construction* is now async-native on this crate too
//! (`build_snapshot_from_manifest` awaits the P&M drive); its browser fetch behavior is covered by
//! the M3 browser smoke test, since an in-memory `object_store` resolves synchronously and would
//! not exercise the event-loop cooperation that a real `fetch` (and the async `.await`, not
//! `block_on`) requires.

#![cfg(all(target_arch = "wasm32", target_os = "unknown"))]

use std::sync::Arc;

use delta_kernel::arrow::array::{Int64Array, RecordBatch};
use delta_kernel::arrow::datatypes::{DataType as ArrowDataType, Field, Schema as ArrowSchema};
use delta_kernel::expressions::{ColumnName, Scalar};
use delta_kernel::parquet::arrow::arrow_writer::ArrowWriter;
use delta_kernel::schema::{DataType, StructField, StructType};
use delta_kernel::sm_plans::ir::nodes::{FileType, ScanFileColumns};
use delta_kernel::sm_plans::state_machines::framework::plan_context::{Context, LoadSpec};
use object_store::ObjectStoreExt;
use object_store::memory::InMemory;
use object_store::path::Path;
use olai_delta_df::{DataFusionExecutor, testing};
use url::Url;
use wasm_bindgen_test::wasm_bindgen_test;

/// Encode a one-column `Int64` parquet file to bytes via the kernel-re-exported (arrow-58 fork)
/// parquet writer — proving parquet *encode* works on wasm too.
fn parquet_i64(field: &str, values: &[i64]) -> Vec<u8> {
    let schema = Arc::new(ArrowSchema::new(vec![Field::new(
        field,
        ArrowDataType::Int64,
        false,
    )]));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(Int64Array::from_iter_values(
            values.iter().copied(),
        ))],
    )
    .unwrap();
    let mut buf = Vec::new();
    let mut writer = ArrowWriter::try_new(&mut buf, schema, None).unwrap();
    writer.write(&batch).unwrap();
    writer.close().unwrap();
    buf
}

/// The engine-free SSA compile + Load path runs on wasm32: a `NodeKind::Load` reads parquet
/// bytes back out of an in-memory object store over DataFusion's async parquet source, driven
/// under `wasm-bindgen-futures`. No tokio, no inline executor.
#[wasm_bindgen_test]
async fn load_over_in_memory_store_runs_on_wasm() {
    // Register an in-memory store on a DataFusion session under a `memory://` URL and write a
    // parquet data file into it. This stands in for the browser fetch store; the point is that
    // the read goes through DataFusion's async object-store path, poll-driven, on wasm.
    let store = Arc::new(InMemory::new());
    let base = Url::parse("memory:///data/").unwrap();
    let parquet = parquet_i64("x", &[10, 20, 30]);
    store
        .put(&Path::from("data/part-0.parquet"), parquet.into())
        .await
        .unwrap();

    // Build the session-backed executor and register the store so the compiled plan resolves
    // `memory://` reads.
    use datafusion::execution::context::SessionContext;
    use datafusion::execution::runtime_env::RuntimeEnv;
    use datafusion::prelude::SessionConfig;
    let mut config = SessionConfig::new();
    config.options_mut().execution.target_partitions = 1;
    config
        .options_mut()
        .execution
        .parquet
        .schema_force_view_types = false;
    let session = SessionContext::new_with_config_rt(config, Arc::new(RuntimeEnv::default()));
    session.runtime_env().register_object_store(&base, store);
    let exec = DataFusionExecutor::from_session(session);

    // Hand-build an SSA plan: a Values upstream (one row: the file path + a broadcast tag) fed
    // into a Load node reading the parquet file. This is the shape a scan replay produces, minus
    // the log reconciliation.
    let upstream_schema = Arc::new(
        StructType::try_new([
            StructField::not_null("path", DataType::STRING),
            StructField::not_null("tag", DataType::STRING),
        ])
        .unwrap(),
    );
    let file_schema =
        Arc::new(StructType::try_new([StructField::not_null("x", DataType::LONG)]).unwrap());

    let ctx = Context::new();
    let upstream = ctx
        .values(
            upstream_schema,
            vec![vec![
                Scalar::String("part-0.parquet".into()),
                Scalar::String("alpha".into()),
            ]],
        )
        .unwrap();
    let builder = upstream
        .load(LoadSpec {
            file_schema,
            file_type: FileType::Parquet,
            base_url: Some(base),
            passthrough_columns: vec![ColumnName::new(["tag"])],
            file_meta: ScanFileColumns {
                path: ColumnName::new(["path"]),
                size: None,
                record_count: None,
            },
            dv_ref: None,
        })
        .unwrap();
    let rp = ctx.into_result_plan(builder).unwrap();

    // Drive compile + async parquet read on wasm.
    let batches = testing::collect_ssa_result(&exec, rp).await.unwrap();
    let total: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        total, 3,
        "three rows read back from the in-memory parquet file"
    );

    let schema = batches[0].schema();
    assert!(schema.field_with_name("x").is_ok());
    assert!(schema.field_with_name("tag").is_ok());
}
