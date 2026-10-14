//! wasm32 spike check: the published (patched) `buoyant_kernel` builds a snapshot and runs a full
//! scan on `wasm32-unknown-unknown` under the record/replay engine — no tokio, no blocking. The
//! kernel's snapshot path also exercises the `Instant` shim (std's clock panics on this target).

#![cfg(all(target_arch = "wasm32", target_os = "unknown"))]

mod common;

use std::sync::Arc;

use buoyant_kernel::engine::arrow_data::EngineDataArrowExt as _;
use buoyant_kernel::snapshot::Snapshot;
use kernel_spike::{ReplayEngine, replay};
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
async fn snapshot_and_scan_on_wasm() {
    let store = common::three_commit_table().await;
    let engine = ReplayEngine::new();

    let (snapshot, snap_stats) = replay(store.as_ref(), &engine, |e| {
        Snapshot::builder_for(common::table_url()).build(e)
    })
    .await
    .unwrap();
    assert_eq!(snapshot.version(), 2);

    let scan = Arc::new(snapshot.scan_builder().build().unwrap());
    let (rows, scan_stats) = replay(store.as_ref(), &engine, |e| {
        let engine: Arc<dyn buoyant_kernel::Engine> = Arc::new(e.clone());
        scan.execute(engine)?
            .map(|data| data?.try_into_record_batch().map(|b| b.num_rows()))
            .sum::<Result<usize, _>>()
    })
    .await
    .unwrap();
    assert_eq!(rows, 6);
    assert!(snap_stats.passes > 1 && scan_stats.passes > 1);
}
