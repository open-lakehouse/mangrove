//! Native spike checks: the record/replay engine produces the same snapshots / scans as the stock
//! (blocking) `DefaultEngine`, over commit-only and checkpointed tables.

#![cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]

mod common;

use std::sync::Arc;

use buoyant_kernel::arrow::array::RecordBatch;
use buoyant_kernel::engine::arrow_data::EngineDataArrowExt as _;
use buoyant_kernel::plans::ir::nodes::Operator;
use buoyant_kernel::snapshot::{Snapshot, SnapshotRef};
use buoyant_kernel_engine::DefaultEngineBuilder;
use kernel_spike::{ReplayEngine, replay};
use object_store::ObjectStore;

fn reference_snapshot(store: Arc<dyn ObjectStore>, version: Option<u64>) -> SnapshotRef {
    let engine = DefaultEngineBuilder::new(store).build();
    let mut builder = Snapshot::builder_for(common::table_url());
    if let Some(v) = version {
        builder = builder.at_version(v);
    }
    builder.build(&engine).unwrap()
}

fn count_rows(batches: &[RecordBatch]) -> usize {
    batches.iter().map(|b| b.num_rows()).sum()
}

fn assert_same_snapshot(replayed: &SnapshotRef, reference: &SnapshotRef) {
    assert_eq!(replayed.version(), reference.version());
    assert_eq!(replayed.schema(), reference.schema());
    assert_eq!(
        replayed.table_configuration().protocol(),
        reference.table_configuration().protocol()
    );
    assert_eq!(
        replayed.table_configuration().metadata(),
        reference.table_configuration().metadata()
    );
}

/// Snapshot + full data scan through the replay engine; returns (snapshot, live row count, passes).
async fn replay_snapshot_and_scan(store: &dyn ObjectStore) -> (SnapshotRef, usize, usize, usize) {
    let engine = ReplayEngine::new();
    let (snapshot, snap_stats) = replay(store, &engine, |e| {
        Snapshot::builder_for(common::table_url()).build(e)
    })
    .await
    .unwrap();

    let scan = Arc::new(snapshot.clone().scan_builder().build().unwrap());
    let (batches, scan_stats) = replay(store, &engine, |e| {
        let engine: Arc<dyn buoyant_kernel::Engine> = Arc::new(e.clone());
        scan.execute(engine)?
            .map(|data| data?.try_into_record_batch())
            .collect::<Result<Vec<_>, _>>()
    })
    .await
    .unwrap();
    (
        snapshot,
        count_rows(&batches),
        snap_stats.passes,
        scan_stats.passes,
    )
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn commit_only_matches_default_engine() {
    let store = common::three_commit_table().await;
    let reference = reference_snapshot(store.clone(), None);

    let (snapshot, rows, snap_passes, scan_passes) = replay_snapshot_and_scan(store.as_ref()).await;
    assert_same_snapshot(&snapshot, &reference);
    assert_eq!(snapshot.version(), 2);
    assert_eq!(rows, 6, "b + c + d live after the v2 remove");
    eprintln!("commit-only: snapshot passes={snap_passes}, scan passes={scan_passes}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn checkpointed_matches_default_engine() {
    let store = common::three_commit_table().await;
    // Write a classic parquet checkpoint at v1 with the stock engine, so v2 replays on top of it.
    {
        let engine = DefaultEngineBuilder::new(store.clone() as Arc<dyn ObjectStore>).build();
        let at_v1 = reference_snapshot(store.clone(), Some(1));
        at_v1.checkpoint(&engine, None).unwrap();
    }
    eprintln!("objects: {:?}", common::object_paths(store.as_ref()).await);
    let reference = reference_snapshot(store.clone(), None);
    assert_eq!(reference.log_segment().checkpoint_version, Some(1));

    let (snapshot, rows, snap_passes, scan_passes) = replay_snapshot_and_scan(store.as_ref()).await;
    assert_same_snapshot(&snapshot, &reference);
    assert_eq!(snapshot.log_segment().checkpoint_version, Some(1));
    assert_eq!(rows, 6);
    eprintln!("checkpointed: snapshot passes={snap_passes}, scan passes={scan_passes}");
}

/// The declarative metadata plan (the connector-native scan path the port would compile into
/// DataFusion) is buildable through the IO-only plan executor; report its operator shape.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn declarative_metadata_plan_shape() {
    let store = common::three_commit_table().await;
    {
        let engine = DefaultEngineBuilder::new(store.clone() as Arc<dyn ObjectStore>).build();
        reference_snapshot(store.clone(), Some(1))
            .checkpoint(&engine, None)
            .unwrap();
    }
    let engine = ReplayEngine::new();
    let (snapshot, _) = replay(store.as_ref(), &engine, |e| {
        Snapshot::builder_for(common::table_url()).build(e)
    })
    .await
    .unwrap();
    let scan = snapshot.scan_builder().build().unwrap();

    let plan_engine = engine.sharing_cache(true);
    let (plan, stats) = replay(store.as_ref(), &plan_engine, |e| {
        scan.declarative_metadata_scan_plan(e)
    })
    .await
    .unwrap();
    let plan = plan.expect("non-empty table has a metadata plan");
    let ops: Vec<String> = plan
        .nodes
        .iter()
        .map(|n| match &n.op {
            Operator::ScanParquet(s) => format!("ScanParquet[{} files]", s.files.len()),
            Operator::ScanJson(s) => format!("ScanJson[{} files]", s.files.len()),
            other => format!("{other}"),
        })
        .collect();
    eprintln!(
        "declarative metadata plan (passes={}): {ops:?}",
        stats.passes
    );
    assert!(
        ops.iter().any(|o| o.starts_with("ScanParquet")),
        "reads the checkpoint"
    );
    assert!(
        ops.iter().any(|o| o.starts_with("ScanJson")),
        "reads the v2 commit"
    );
}
