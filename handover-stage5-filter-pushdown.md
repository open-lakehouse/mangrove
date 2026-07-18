# Handover: Stage 5 — Delta filter pushdown in `olai-delta-df` (two-layer)

**Status:** Layer 2 (DataFusion parquet pruning) landed on branch
`stage5-filter-pushdown` (2 commits). Kernel Layer-1 change committed on the
kernel `sm-plans-consolidation` branch (1 commit). **Remaining:** user pushes the
kernel commit, mangrove re-pins, then the mangrove Layer-1 wiring (forward
predicate translator + `build_scan().with_predicate`) is added and validated.

Closes the column-mapping narrow-waist arc (Stages 1–4 + K already merged; see
`handover-column-mapping-narrow-waist.md`).

---

## What Stage 5 does

Re-enables filter pushdown so a predicate on a table column actually **skips data
files**, in two composing layers:

1. **Layer 1 (kernel) — file-list skipping** on the `sm_plans` SSA scan path:
   prune whole files *before* they enter the plan, via a declarative
   data-skipping `FilterNode` over the reconciled `add.stats_parsed` rows.
2. **Layer 2 (mangrove) — row-group/page skipping** *within* surviving files:
   hand the per-file parquet `ParquetSource` a pushdown predicate that prunes
   against the Stage-4 per-file `Statistics`.

**Design constraint (confirmed with user): declarative / engine-free.** The
classic `DataSkippingFilter` is technically wasm-safe (its `Engine` use is pure
expression evaluation, no I/O) but is deliberately NOT adopted — we express
skipping as an SSA `FilterNode` evaluated lazily by the same engine that runs
every other reconciliation filter, keeping one evaluation path and wasm/async
compatibility. Only the pure `Predicate → Predicate` rewriter is reused.

---

## LANDED — Layer 2 (mangrove, branch `stage5-filter-pushdown`)

Two commits, both `-p olai-delta-df` green (50 lib + all integration tests),
fmt + clippy clean:

1. `feat(olai-delta-df): thread a pushdown-predicate side channel to the Load leaf`
   — `predicate: Option<Arc<dyn PhysicalExpr>>` added to `CompileContext`; compile
   entry point regrouped into `SideChannels { file_stats, predicate }`
   (`compile_result_plan_with_side_channels`); threaded
   `lower_load → LoadTableProvider → LoadExec → build_file_source`, applied once via
   `ParquetSource::with_predicate` (parquet only). No-op plumbing (`None` everywhere).
2. `feat(olai-delta-df): re-enable filter pushdown via per-file parquet pruning`
   — `scan()` lowers filters (`conjunction` + `Session::create_physical_expr` against
   the logical arrow schema, degrade-to-`None`); `supports_filters_pushdown` →
   `Inexact` (top-level provider only). Tests in `provider.rs::stats_e2e_tests`:
   `filter_pushdown_result_correct_{name,id}_mode` (e2e correctness over the two-file
   CM fixture), `per_file_predicate_prunes_out_of_range_file` /
   `per_file_predicate_keeps_in_range_file` (build the per-file `DataSourceExec`
   directly, read `files_ranges_pruned_statistics` off its `MetricsSet` — proves
   pruning fires; the metric is unreachable through the top-level plan since
   `LoadExec` builds per-file execs lazily). Also corrects the stale `column_mapping`
   resolver docs.

**Key verified facts (DataFusion 54.0.0):** pruning runs in *logical* file-schema
space (`FilePruner::try_new(pred, &logical_file_schema, &partitioned_file, …)`,
`opener/mod.rs:622`), so the DataFusion predicate stays **logical-named** — the
wired `FieldIdPhysicalExprAdapterFactory` reconciles logical↔physical at decode,
after pruning. No `ColumnMappingResolver::logical_to_physical` needed on this path.

## LANDED — Layer 1 kernel change (kernel `sm-plans-consolidation`, commit `82431096`)

On top of `bb721add` (the rev mangrove pins today). One commit:
`feat(sm-plans): data-skipping filter on the SSA scan terminal`. All 3236 kernel
lib tests pass + 3 new; fmt + clippy clean (built with
`--features sm-plans,internal-api,default-engine-base,arrow-58`).

- **`as_ssa_add_stats_skipping_predicate(pred, stats_columns)`**
  (`kernel/src/scan/data_skipping.rs`): pure, engine-free. Reuses
  `as_sql_data_skipping_predicate_with_stats_columns` (no `&dyn Engine`), then
  prefixes stats refs with `add` → `add.stats_parsed.*` (the rewriter already roots
  at `stats_parsed`). Stats-only (empty partitions); conservative `None` bail if any
  ref escapes `add.stats_parsed` (opaque `is_add` guard).
- **`apply_data_skipping_ssa(reconciled, scan)`**
  (`kernel/src/sm_plans/state_machines/scan/ssa_scan.rs`): inserts the `FilterNode`
  between reconciliation and `project_scan_file_row` in `build_scan_ssa`. Reads
  `scan.state_info().physical_predicate` directly (to honor `StaticSkipAll` →
  const-false filter; `physical_predicate()` collapses it to `None`). No predicate →
  byte-identical plan.
- `PrefixColumns` promoted to `pub(crate)` in `data_skipping.rs`; the private
  duplicate in `scan::mod` removed and re-imported.

Tests (in `data_skipping/tests.rs`): `ssa_add_stats_prefixes_refs_under_add_stats_parsed`,
`ssa_add_stats_keeps_overlapping_prunes_below_range` (SQL-WHERE: prune iff FALSE),
`ssa_add_stats_ineligible_predicate_never_prunes`.

---

## REMAINING WORK (next session)

### Step 1 — push the kernel + re-pin mangrove (cross-repo, same-owner)
1. **User pushes** kernel `sm-plans-consolidation` (now at `82431096`). No upstream
   PR/publish gate — it's our own integration branch.
2. Re-pin mangrove (mirror PR #126): in `Cargo.toml` bump `delta_kernel` +
   `delta_kernel_default_engine` rev `bb721add → 82431096` (or the pushed SHA),
   lock-step; if delta-rs needs to follow, bump `deltalake-core`/`-wasm` too (only if
   its kernel pin must match — K2 is additive & sm-plans-gated, so delta-rs likely
   needs no bump). Regenerate root `Cargo.lock` **and** `crates/query-wasm/Cargo.lock`
   (`--locked` CI gate). Update the pin-rationale comments to name the K2 commit.

> Local validation before the push: add a temporary `[patch."https://github.com/roeap/delta-kernel-rs"]`
> in the worktree `.cargo/config.toml` pointing `delta_kernel` + `delta_kernel_default_engine`
> at `/Users/robert.pack/code/delta-kernel-rs` (path or `branch = "sm-plans-consolidation"`),
> so mangrove builds against `82431096` while iterating on Step 2. Remove before commit.

### Step 2 — mangrove Layer-1 wiring
The kernel now applies a `FilterNode` from `Scan::physical_predicate()`, but mangrove
never *sets* the predicate. Two pieces:

- **Forward `Expr → kernel Predicate` translator** in
  `crates/olai-delta-df/src/compile/expr_translator.rs`. The crate already has the
  reverse (`kernel_pred_to_df`, line 130) — mirror it for the forward direction:
  comparison ops (eq/ne/lt/le/gt/ge), `AND`/`OR` junctions, `NOT`, `IS [NOT] NULL`,
  column refs (logical names — the kernel rewrites logical→physical itself via
  `physical_predicate`), and literals. **Drop untranslatable arms** (return `None`
  for the whole filter or skip the arm) — safe because `supports_filters_pushdown`
  is already `Inexact` (Layer 2). NO `deltalake_core` dep (engine-free), so
  `to_delta_predicate` is unavailable — hand-roll the subset data-skipping supports.
- **`build_scan().with_predicate(kernel_pred)`** in `provider.rs`
  (`build_scan`, ~line 158). Thread the query `filters` (already available in
  `scan()`) → forward-translate → `ScanBuilder::with_predicate(Some(Arc::new(pred)))`
  so the SSA path's `apply_data_skipping_ssa` reads it. Note: `build_scan` is called
  from both `new()` (schema only, no predicate) and `scan()` (with predicate) — give
  it an optional predicate param or add a predicate-aware variant used only by `scan()`.

### Step 3 — validate Layer 1 end-to-end
The mangrove `stats_e2e_tests` two-file CM fixture (file A `id∈[1,3]`, file B
`id∈[4,6]`) is the vehicle. With Layer 1 wired, `WHERE id >= 4` should drop file A
from the **kernel's live-file list** (fewer `Load` rows), not just DataFusion-prune
it. Assert the file count the scan enumerates, or that file A's parquet is never
fetched (a counting object-store wrapper). Layer 2's row correctness tests already
pass and must stay green.

---

## Delivery
- **Layer 2** is a self-contained mangrove PR off `main` (branch
  `stage5-filter-pushdown`), the two commits above. Can merge independently of the
  kernel push.
- **Layer 1 (mangrove side)** lands after the re-pin, either folded into the same PR
  (if the push happens first) or a follow-up PR.
- Sign the branch once before opening the PR (machine-wide flow); the local
  `.cargo/config.toml` trestle patch + any kernel `[patch]` must NOT be committed
  (gitignored), and `Cargo.lock`'s trestle path-dep edits must NOT be committed.

## Status ledger
| Stage | Status |
|---|---|
| 1–3 | ✅ PR #125 |
| K | ✅ kernel `bb721add` / PR #126 |
| 4 | ✅ PR #128 |
| 5 Layer 2 (DataFusion parquet pruning) | ✅ branch `stage5-filter-pushdown` (2 commits) |
| 5 Layer 1 kernel (SSA `FilterNode`) | ✅ kernel `82431096` (unpushed) |
| 5 Layer 1 mangrove wiring (fwd translator + `with_predicate`) | ☐ next session (after re-pin) |
