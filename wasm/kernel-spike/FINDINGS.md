# B0 spike findings (#198): wasm engine on published buoyant_kernel 0.28

## Question

The roeap `sm-plans` fork gave the wasm engine an async seam: kernel state machines (`CoroutineSM`, `SnapshotPm`) handed IO back to us. Upstream dropped `sm-plans`. Published `buoyant_kernel` 0.28 is **synchronous** for everything the kernel does itself: listing, `_last_checkpoint`/CRC, P&M replay, checkpoint-shape probes, log replay. Its declarative plans (`plans::ir`) also run through a sync `PlanExecutor::execute_op`. Can the browser engine use it without blocking on fetch?

## Answer: yes, with a record/replay engine (`src/lib.rs`)

`ReplayEngine` is a plain kernel `Engine`. Its storage, JSON and parquet handlers read **only** from an in-memory cache.

- **Misses fail fast.** A miss is recorded and returns a pending error immediately. It never blocks.
- **The replay loop.** `replay()` runs a kernel call, fetches all recorded misses concurrently through `object_store`, and re-runs the call.
- **Only a clean pass counts.** The result of a pass is accepted only when that pass had **zero** misses. So any fallback the kernel took on a pending error in an earlier pass never leaks out: the accepted pass saw exactly what a blocking engine would see.
- **Re-runs repeat CPU work only.** No IO is repeated.

Verified (`tests/replay_native.rs`, `tests/replay_wasm.rs`):

| Case | Snapshot passes | Scan passes | Parity with stock `DefaultEngine` |
|---|---|---|---|
| commit-only, 3 commits incl. a remove | 4 | 4 | version, schema, protocol, metadata; 6 live rows |
| classic parquet checkpoint at v1 + v2 commit | 5 | 4 | same, `checkpoint_version == 1` |
| wasm32 (node runner), commit-only | ✓ | ✓ | 6 live rows, no tokio, no blocking |

The handlers mirror the default engine's sync decode paths, using the kernel's `internal-api` helpers: `json_arrow_schema` / `build_json_reorder_indices` / `fixup_json_read`, `parquet_read_plan` / `fixup_parquet_read` / `RowIndexBuilder`, and `parse_json`.

## Findings

1. **Published buoyant 0.28.1 does not build or run on wasm32.**
   - It calls `Url::{from_file_path, from_directory_path, to_file_path}` unguarded; the url crate doesn't provide these on wasm32.
   - It uses `std::time::Instant` / `SystemTime` (snapshot build, metrics, transactions). std's clock panics on `wasm32-unknown-unknown`.
   - The 0.28 source commit (`b7cedb67`) is on no public remote.
   - Fix carried as `roeap/delta-kernel-rs` `buoyant/v0.28.1-wasm`: the crates.io source verbatim, plus one commit porting the fork's cfg guards and js-sys time shim. It's applied in `wasm/Cargo.toml` `[patch.crates-io]` only; native stays on pure crates.io. To be upstreamed (#199).
2. **Consumer-side features:** getrandom 0.3 `wasm_js` (buoyant → rand 0.9), uuid `js` (buoyant mints `Uuid::new_v4()`), and getrandom 0.4 `wasm_js` (object_store → rand 0.10). The fork used to enable the first two itself.
3. **`PlanBasedEngine` doesn't fit.** It turns every JSON/parquet read into a `QueryPlan(ScanJson/ScanParquet)` and sends P&M replay through declarative plans. Using it would mean interpreting plans synchronously over prefetched data. The handler-based engine avoids that.
   - The plan executor is enabled only where the API requires one (`Scan::declarative_metadata_scan_plan`).
   - Its executor serves IO operations plus leaf-only scan plans. The checkpoint-shape sidecar probe is exactly a handler read.
4. **The declarative metadata plan is compilable.** For the checkpointed table it uses 7 of 9 operators: `ScanJson → Filter → Project → Aggregate → Filter → Project`, `ScanParquet → Filter → Project → SemiJoin(anti) → Project`, `UnionAll`. All of these map onto concepts the current `compile/logical/ssa.rs` already lowers (Load/Scan, Filter, Project, MaxByVersion, EquiJoin anti, Union).

## Limits of the spike

- **Whole objects are cached.** A footer probe fetches the full checkpoint. Fine for `_delta_log`-sized inputs; large checkpoints want range-aware caching (or option (ii) below).
- **Scans used `Scan::execute`.** The kernel's sync parquet reads, after the data files are fetched. The real port streams data files through DataFusion's async parquet path instead.
- **Read-only.** Deletion vectors are unsupported (as today).

## Recommended port (B1)

1. **Snapshot:** `replay(|e| Snapshot::builder_for(url).with_log_tail(..).with_max_catalog_version(..).build(e))`. This replaces `SnapshotPm` / `LogSegment::from_listed_files`.
   - To keep the **list-free** property, seed the cache's `_delta_log` listing from the UC manifest the engine already discovers. That's a small `ReplayEngine::seed_listing` and saves one pass.
2. **Scan metadata, two options:**
   - (i) Replay the kernel's `scan.scan_metadata()` over the cached log, then feed the add files plus per-file transforms into the existing DataFusion data-file Load leaf. This is the smallest delta and reuses the kernel's log-replay correctness.
   - (ii) Compile `declarative_metadata_scan_plan` into DataFusion (the 7 operators above) so log replay also streams asynchronously. That's the scalable path for large checkpoints.
   - Start with (i); add (ii) once big-checkpoint tables matter.
3. **Log explorers:** rebuild Reconciled/Actions on `scan_metadata` and `log_segment` replay (`internal-api`). That's the same surface native `crates/datafusion/src/log_explorer` uses, so the code may become shareable.
4. **Version moves:**
   - datafusion 55 / datafusion-functions-json 0.55 / arrow 59.
   - Rebase the 3-commit roeap arrow-rs wasm fork onto 59.3.0.
   - Switch the buoyant feature to `arrow-59`.
   - Then drop the roeap `delta-kernel-rs` sm-plans pin.
