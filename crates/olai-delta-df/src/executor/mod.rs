//! [`DataFusionExecutor`]: drives kernel SSA coroutine state machines and compiles their
//! step payloads to DataFusion plans.
//!
//! Every step the SM yields is either an `EngineRequest::SchemaQuery` (parquet footer read) or
//! an `EngineRequest::Consume` (SSA dataflow drained into a [`ConsumeSink`]). Terminal
//! `ResultPlan`s describe a single self-contained dataflow DAG that compiles to a `LogicalPlan`.
//!
//! # Engine-free
//!
//! Unlike the POC, this executor holds **no kernel `Engine`**. Both POC touch-points that needed
//! one are handled without it:
//!
//!   1. deletion-vector reads — dropped entirely (DVs are gated to `Unsupported` upstream), and
//!   2. checkpoint-footer `SchemaQuery` — serviced async over the session object store via
//!      `parquet::arrow::async_reader` (a metadata-only footer read; see [`Self::read_footer_schema`]),
//!      converting the arrow schema to a kernel `StructType`. This is what lets the P&M
//!      snapshot-construction SM ([`build_snapshot_pm`](Self::build_snapshot_pm)) resolve a
//!      classic-checkpointed table, and ungates classic checkpoints on the scan path.
//!
//! [`ConsumeSink`]: delta_kernel::sm_plans::ir::nodes::ConsumeSink

use std::sync::Arc;

use datafusion::catalog::Session;
use datafusion_common::error::DataFusionError;
use datafusion_expr::LogicalPlan;
use datafusion_physical_plan::coalesce_partitions::CoalescePartitionsExec;
use datafusion_physical_plan::{ExecutionPlan, ExecutionPlanProperties};
use delta_kernel::arrow::array::RecordBatch;
use delta_kernel::engine::arrow_conversion::TryFromArrow;
use delta_kernel::engine::arrow_data::ArrowEngineData;
use delta_kernel::log_segment::LogSegment;
use delta_kernel::parquet::arrow::async_reader::{
    ParquetObjectReader, ParquetRecordBatchStreamBuilder,
};
use delta_kernel::scan::Scan;
use delta_kernel::schema::{SchemaRef as KernelSchemaRef, StructType};
use delta_kernel::sm_plans::errors::{DeltaError, KernelErrAsDelta};
use delta_kernel::sm_plans::ir::nodes::ConsumeSink;
use delta_kernel::sm_plans::kernel_consumers::{FinishedHandle, KdfControl};
use delta_kernel::sm_plans::state_machines::framework::coroutine::driver::CoroutineSM;
use delta_kernel::sm_plans::state_machines::framework::engine_error::{
    EngineError, EngineErrorKind,
};
use delta_kernel::sm_plans::state_machines::framework::state_machine::{NextStep, StateMachine};
use delta_kernel::sm_plans::state_machines::framework::step::{EngineRequest, SchemaQuery};
use delta_kernel::sm_plans::state_machines::framework::step_payload::EngineResponse;
use delta_kernel::sm_plans::state_machines::scan::FullState;
use delta_kernel::sm_plans::state_machines::snapshot::SnapshotPm;
use delta_kernel::snapshot::{Snapshot, SnapshotRef};
use futures::TryStreamExt;
use url::Url;
use uuid::Uuid;

use crate::compile::{CompileContext, compile_ssa};
use crate::error::DfResultIntoDelta;

/// Minimal, **stateless** executor that drives kernel SSA state machines against a
/// caller-supplied [`Session`]. Carries no kernel engine and holds no session of its own — the
/// session is passed in per call (like [`TableProvider::scan`](datafusion::catalog::TableProvider::scan)),
/// and the [`TaskContext`](datafusion_execution::TaskContext) for
/// [`ExecutionPlan::execute`] calls is derived from it.
///
/// Threading the *caller's* session through the drive (rather than a session the executor spins up
/// itself) is what keeps object store, scalar functions, and `execution_props` (the `now()` /
/// query-start anchor) consistent between the reconciliation `Consume` plans driven here and the
/// final scan plan the caller runs against the same session.
///
/// The session must carry the Delta engine config — see
/// [`crate::session::configure_delta_engine_config`]; build one via
/// [`crate::delta_engine_session`] or [`crate::DeltaEngineSessionExt::with_delta_engine`], and (at
/// integration boundaries) assert it via [`crate::validate_delta_engine_session`].
#[derive(Debug, Default, Clone, Copy)]
pub struct DataFusionExecutor;

impl DataFusionExecutor {
    /// Construct the (stateless) executor. Drive methods take the [`Session`] per call.
    pub fn new() -> Self {
        Self
    }

    /// Compile a [`ResultPlan`] to a bare [`LogicalPlan`], unbound to any session.
    ///
    /// Associated (session-free) function: this is pure compilation — it lowers SSA nodes to
    /// logical operators and `LoadTableProvider`s but does **not** plan or execute — so it needs
    /// neither an executor instance nor a session. Callers plan it against their session: the
    /// [`crate::DeltaSsaTableProvider::scan`] path splices projection/limit and calls
    /// `session.create_physical_plan`; a caller wanting rows plans it and executes via
    /// `datafusion_physical_plan::execute_stream`. Kept here (rather than as a loose free function)
    /// so it sits next to the drive methods that produce the `ResultPlan` it consumes.
    ///
    /// [`ResultPlan`]: delta_kernel::sm_plans::ir::plan::ResultPlan
    /// [`LogicalPlan`]: datafusion_expr::LogicalPlan
    pub fn compile_result_plan(
        rp: &delta_kernel::sm_plans::ir::plan::ResultPlan,
    ) -> Result<LogicalPlan, DeltaError> {
        Self::compile_result_plan_with_stats(rp, None)
    }

    /// [`compile_result_plan`](Self::compile_result_plan) with per-file [`Statistics`] attached: the
    /// `file_stats` map (keyed by raw `add.path`) is threaded onto the compiled `Load` leaf so each
    /// per-file `PartitionedFile` carries its statistics for DataFusion pruning. `None` reproduces
    /// the plain compile exactly. The provider's stats-enabled scan is the only caller that passes
    /// `Some`.
    pub fn compile_result_plan_with_stats(
        rp: &delta_kernel::sm_plans::ir::plan::ResultPlan,
        file_stats: Option<Arc<crate::compile::stats::FileStatsMap>>,
    ) -> Result<LogicalPlan, DeltaError> {
        let ctx = CompileContext {
            sm_id: crate::next_sm_id(),
            sm_kind: "standalone",
            step_name: "compile_result_plan",
            file_stats,
        };
        compile_ssa(&rp.plan.stmts, rp.result, &ctx).into_delta()
    }

    // ================================================================
    // High-level SM and result-plan driving
    // ================================================================

    /// Drive `sm` against `session` until it terminates, executing any intermediate phase
    /// operations it yields (kernel-side decision plans, schema queries) and returning the SM's
    /// terminal value.
    ///
    /// The terminal value is whatever `R` the SM was constructed for: for read-style SMs that
    /// is typically a [`ResultPlan`] the caller compiles via [`Self::compile_result_plan`].
    ///
    /// # `!Send` future
    ///
    /// The kernel state machine is a CPU-only sequencer (see
    /// [`delta_kernel::sm_plans::state_machines::framework::coroutine::driver::CoroutineSM`] module
    /// docs); it intentionally does not implement `Send`. The future returned here inherits that
    /// and is therefore `!Send`. Callers needing a `Send` future drive this on a single-threaded
    /// runtime (`tokio::runtime::Builder::new_current_thread()` + `block_on`), wrap the call in a
    /// [`tokio::task::LocalSet`], or (on wasm) drive it under `wasm-bindgen-futures`.
    ///
    /// [`ResultPlan`]: delta_kernel::sm_plans::ir::plan::ResultPlan
    pub async fn drive_to_completion<R: 'static>(
        &self,
        session: &dyn Session,
        mut sm: CoroutineSM<R>,
    ) -> Result<R, DeltaError> {
        let sm_id = sm.sm_id();
        let sm_kind = sm.sm_kind();
        loop {
            // Zero-yield SMs have no step to fetch; the first `submit` hands the stored
            // terminal value back directly. Pass [`EngineResponse::Empty`] in that case so
            // `submit` has a valid (unused) input.
            let step_name = sm.step_name();
            let phase_result = match sm.get_step() {
                Ok(op) => self.run_phase(session, op, sm_id, sm_kind, step_name).await,
                Err(_) => Ok(EngineResponse::Empty),
            };
            match sm.submit(phase_result)? {
                NextStep::Continue => {}
                NextStep::Done(value) => return Ok(value),
            }
        }
    }

    /// Drive a coroutine that yields a [`ResultPlan`] against `session` and compile its terminal
    /// output to a bare [`LogicalPlan`]. The plan is unbound and unexecuted — the caller plans +
    /// executes it against its session (`session.create_physical_plan`, optionally after splicing
    /// projection/limit on top). SSA plans describe a single self-contained dataflow DAG.
    ///
    /// [`ResultPlan`]: delta_kernel::sm_plans::ir::plan::ResultPlan
    /// [`LogicalPlan`]: datafusion_expr::LogicalPlan
    pub async fn drive_ssa_to_plan(
        &self,
        session: &dyn Session,
        sm: CoroutineSM<delta_kernel::sm_plans::ir::plan::ResultPlan>,
    ) -> Result<LogicalPlan, DeltaError> {
        let rp = self.drive_to_completion(session, sm).await?;
        Self::compile_result_plan(&rp)
    }

    /// Drive a coroutine that yields a [`ResultPlan`], compile it, plan it against `session`, and
    /// **execute** it — returning the terminal rows as [`RecordBatch`]es.
    ///
    /// This is the rows-collecting sibling of [`drive_ssa_to_plan`](Self::drive_ssa_to_plan): where
    /// that hands back an unexecuted [`LogicalPlan`] for the caller to splice into a larger scan,
    /// this one materializes the terminal itself. Its use is the metadata-only *stats* SM
    /// (`Scan::scan_stats_metadata_state_machine`), whose small terminal (one row per live file) the
    /// provider consumes directly to build per-file [`Statistics`].
    ///
    /// Returns `Ok(None)` — **skipped, not an error** — **only on wasm32** when the compiled plan
    /// would perform object-store IO (a [`LoadExec`]/[`FileListingExec`] leaf; see
    /// [`plan_reads_no_files`]). Because this method *executes* the plan, running IO under
    /// `block_on` on a browser worker starves the `fetch` event loop and hangs. The stats terminal
    /// always reads the commit log (`Values -> LoadExec(JSON)`), so the guard *would* fire on every
    /// table — but native callers `block_on` real IO safely, so the guard is scoped to wasm32:
    /// there we forgo stats (a pruning optimization) rather than hang. On native the plan always
    /// executes.
    ///
    /// # `!Send`
    ///
    /// Like every SM drive here the future is `!Send`; nothing `!Send` is held across the internal
    /// `.await`s and the returned `Vec<RecordBatch>` is `Send`.
    ///
    /// [`ResultPlan`]: delta_kernel::sm_plans::ir::plan::ResultPlan
    /// [`RecordBatch`]: delta_kernel::arrow::array::RecordBatch
    pub async fn drive_ssa_to_batches(
        &self,
        session: &dyn Session,
        sm: CoroutineSM<delta_kernel::sm_plans::ir::plan::ResultPlan>,
    ) -> Result<Option<Vec<RecordBatch>>, DeltaError> {
        let logical = self.drive_ssa_to_plan(session, sm).await?;
        let physical = session.create_physical_plan(&logical).await.into_delta()?;
        // On wasm32 a `block_on` that performs object-store IO starves the `fetch` event loop and
        // hangs. The stats terminal always reads the commit log (`Values -> LoadExec(JSON)`), and a
        // checkpointed table additionally reads checkpoint parquet — both IO leaves. Native callers
        // `block_on` real IO safely (a blocking runtime), so only wasm skips; on wasm we forgo stats
        // (a pruning optimization) rather than hang. See `plan_reads_no_files`.
        if cfg!(target_arch = "wasm32") && !plan_reads_no_files(&physical) {
            return Ok(None);
        }
        let batches = datafusion_physical_plan::collect(physical, session.task_ctx())
            .await
            .into_delta()?;
        Ok(Some(batches))
    }

    /// Drive a combined metadata + data scan against `session` and compile it to a bare
    /// [`LogicalPlan`].
    ///
    /// Sugar for `self.drive_ssa_to_plan(session, scan.scan_state_machine()?)`.
    pub async fn scan_data(
        &self,
        session: &dyn Session,
        scan: &Scan,
    ) -> Result<LogicalPlan, DeltaError> {
        self.drive_ssa_to_plan(session, scan.scan_state_machine()?)
            .await
    }

    /// Drive a metadata-only scan against `session` and compile it to a bare [`LogicalPlan`].
    ///
    /// Sugar for `self.drive_ssa_to_plan(session, scan.scan_metadata_state_machine()?)`.
    pub async fn scan_metadata(
        &self,
        session: &dyn Session,
        scan: &Scan,
    ) -> Result<LogicalPlan, DeltaError> {
        self.drive_ssa_to_plan(session, scan.scan_metadata_state_machine()?)
            .await
    }

    /// Drive a Full State Reconstruction against `session` and compile it to a bare [`LogicalPlan`].
    ///
    /// Sugar for `self.drive_ssa_to_plan(session, fsr.state_machine()?)`.
    pub async fn full_state(
        &self,
        session: &dyn Session,
        fsr: &FullState,
    ) -> Result<LogicalPlan, DeltaError> {
        self.drive_ssa_to_plan(session, fsr.state_machine()?).await
    }

    /// Build a kernel [`SnapshotRef`] from a pre-listed [`LogSegment`], **async-native and
    /// engine-free**: drive the [`SnapshotPm`] state machine to resolve `(Protocol, Metadata)`
    /// (log replay streamed lazily through `session`'s object store — commits + any checkpoint
    /// parquet, short-circuiting once both are found), then assemble the snapshot via
    /// [`Snapshot::from_parts`].
    ///
    /// This is the async-native replacement for the eager `PrimedStore` + synchronous
    /// `DataFusionEngine` snapshot build: no up-front log prime, no `InlineExecutor`, no CRC.
    ///
    /// The `SnapshotPm` SM is `!Send` (like the scan SM), but — unlike the scan SM — its drive
    /// awaits real object-store reads (commit `.json`, checkpoint footer). Callers must therefore
    /// `.await` this, NOT `block_on` it: on a browser worker a blocked thread starves the event
    /// loop a `fetch` needs, hanging construction forever. Awaiting leaves the caller's future
    /// `!Send`, which every driver we target (wasm-bindgen-futures, native current-thread) accepts.
    pub async fn build_snapshot_pm(
        &self,
        session: &dyn Session,
        log_segment: Arc<LogSegment>,
        table_root: Url,
    ) -> Result<SnapshotRef, DeltaError> {
        let sm = SnapshotPm::for_log_segment(Arc::clone(&log_segment)).state_machine()?;
        let (protocol, metadata) = self.drive_to_completion(session, sm).await?;
        let log_segment = Arc::unwrap_or_clone(log_segment);
        let snapshot = Snapshot::from_parts(table_root, log_segment, protocol, metadata)
            .map_err(KernelErrAsDelta::into_delta_default)?;
        Ok(Arc::new(snapshot))
    }

    /// Execute a single [`EngineRequest`] against `session` and return the resulting
    /// [`EngineResponse`]. Used internally by [`Self::drive_to_completion`] and exposed for
    /// callers (typically tests) that need to drive an individual phase op directly.
    pub async fn execute_step(
        &self,
        session: &dyn Session,
        op: EngineRequest,
    ) -> Result<EngineResponse, EngineError> {
        self.run_phase(session, op, crate::next_sm_id(), "standalone", "execute")
            .await
    }

    /// Execute one [`EngineRequest`] against `session`, stamping any `Consume` handles minted
    /// during the run with `(sm_id, sm_kind, step_name)`.
    async fn run_phase(
        &self,
        session: &dyn Session,
        op: EngineRequest,
        sm_id: Uuid,
        sm_kind: &'static str,
        step_name: &'static str,
    ) -> Result<EngineResponse, EngineError> {
        match op {
            EngineRequest::SchemaQuery(node) => self.read_footer_schema(session, &node).await,
            EngineRequest::Consume {
                stmts,
                terminal,
                sink,
            } => {
                let ctx = CompileContext {
                    sm_id,
                    sm_kind,
                    step_name,
                    // Consume-phase compiles (kernel decision plans) never build a data-file Load
                    // leaf, so per-file stats do not apply here.
                    file_stats: None,
                };
                let finished = self
                    .run_consume(session, &stmts, terminal, &sink, &ctx)
                    .await
                    .map_err(EngineError::internal)?;
                Ok(EngineResponse::Consumer(finished))
            }
        }
    }

    /// Compile an SSA [`EngineRequest::Consume`] into a DataFusion physical plan, drain it through
    /// the consume sink, and return the finalized handle.
    async fn run_consume(
        &self,
        session: &dyn Session,
        stmts: &[delta_kernel::sm_plans::ir::plan::PlanNode],
        terminal: delta_kernel::sm_plans::ir::plan::Ref,
        sink: &ConsumeSink,
        ctx: &CompileContext,
    ) -> Result<FinishedHandle, DataFusionError> {
        let logical = compile_ssa(stmts, terminal, ctx)?;
        // Plan against the caller's session. `Session::create_physical_plan` runs the logical
        // optimizer first, so this subsumes the previous explicit `optimize` + `create_physical_plan`
        // — and it uses the caller's config, scalar functions, and execution_props, keeping the
        // reconciliation drive consistent with the final scan plan the caller runs.
        let physical = session.create_physical_plan(&logical).await?;
        self.drain_consume_sink(session, physical, sink, ctx).await
    }

    /// Service a checkpoint-footer [`SchemaQuery`] step: read the parquet file's schema from its
    /// footer over the session object store, engine-free.
    ///
    /// The kernel emits this only for **checkpointed** tables (the reconciliation SM probes the
    /// checkpoint parquet footer to classify inline-vs-manifest / stats layout). Commit-only
    /// previews emit zero `SchemaQuery` steps. Reading only the footer (metadata) — not the row
    /// groups — keeps this cheap and unaffected by page compression, so it works even for
    /// compressed checkpoints. The arrow schema is converted to a kernel [`StructType`] via
    /// [`TryFromArrow`], matching what the kernel's own footer reader would produce.
    async fn read_footer_schema(
        &self,
        session: &dyn Session,
        node: &SchemaQuery,
    ) -> Result<EngineResponse, EngineError> {
        self.read_footer_schema_inner(session, node)
            .await
            .map(EngineResponse::Schema)
            .map_err(|e| {
                EngineError::new(EngineErrorKind::IoError {
                    message: format!(
                        "checkpoint-footer SchemaQuery for `{}` failed: {e}",
                        node.file_path
                    ),
                })
            })
    }

    async fn read_footer_schema_inner(
        &self,
        session: &dyn Session,
        node: &SchemaQuery,
    ) -> Result<KernelSchemaRef, DataFusionError> {
        let url =
            Url::parse(&node.file_path).map_err(|e| DataFusionError::External(Box::new(e)))?;
        // Resolve the caller-registered object store for this URL's authority (the same store the
        // scan path reads through), then footer-read via DataFusion's async parquet reader.
        let listing = datafusion_datasource::ListingTableUrl::parse(url.as_str())?;
        let object_store_url = listing.object_store();
        let store = session.runtime_env().object_store(&object_store_url)?;
        let path = listing.prefix().clone();
        let meta = {
            use delta_kernel::object_store::ObjectStoreExt;
            store.head(&path).await?
        };
        let reader = ParquetObjectReader::new(store, meta.location).with_file_size(meta.size);
        let builder = ParquetRecordBatchStreamBuilder::new(reader)
            .await
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        let arrow_schema = builder.schema().as_ref();
        let kernel_schema = StructType::try_from_arrow(arrow_schema)
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        Ok(Arc::new(kernel_schema))
    }

    /// Drain `physical` through a
    /// [`KernelConsumer`](delta_kernel::sm_plans::kernel_consumers::KernelConsumer) handle
    /// minted from `sink` and return the finalized handle.
    async fn drain_consume_sink(
        &self,
        session: &dyn Session,
        physical: Arc<dyn ExecutionPlan>,
        sink: &ConsumeSink,
        ctx: &CompileContext,
    ) -> Result<FinishedHandle, DataFusionError> {
        let mut handle = sink.new_handle(ctx.sm_id, ctx.sm_kind, ctx.step_name);
        // The consumer must see the whole result as one ordered stream, so coalesce to a single
        // partition before reading. This makes the drain correct for *any* input partitioning
        // rather than assuming one — so single-partition is a structural guarantee of the drain,
        // not something the session config must force globally (`target_partitions=1` is now only a
        // wasm/perf knob, applied via `DeltaEngineSessionOptions::disable_repartition`). Coalescing
        // an already-single-partition plan is a cheap passthrough.
        let coalesced: Arc<dyn ExecutionPlan> =
            if physical.output_partitioning().partition_count() > 1 {
                Arc::new(CoalescePartitionsExec::new(physical))
            } else {
                physical
            };
        let mut stream = coalesced.execute(0, session.task_ctx())?;
        while let Some(batch) = stream.try_next().await? {
            let arrow = ArrowEngineData::new(batch);
            match handle
                .apply_consumer(&arrow)
                .map_err(crate::error::wrap_delta_err)?
            {
                KdfControl::Continue => {}
                KdfControl::Break => break,
            }
        }
        Ok(handle.finish())
    }
}

/// Whether `plan` performs **no object-store IO** — i.e. it can be executed synchronously under a
/// `block_on` on a browser worker (where a blocked thread would starve the `fetch` event loop)
/// without hanging.
///
/// This crate's SSA compiler emits exactly two IO-performing leaves: [`LoadExec`] (opens per-file
/// parquet/json readers at runtime — including the commit-`.json` log replay) and
/// [`FileListingExec`] (lists a store prefix). Every other node — `Values`, `Project`, `Filter`,
/// coalesce — is CPU-only. So the plan is IO-free iff neither of those two node types appears
/// anywhere in the tree. Used **on wasm32 only** by [`drive_ssa_to_batches`](DataFusionExecutor::drive_ssa_to_batches)
/// to gate its `block_on`: the stats terminal always reads the commit log, so on wasm we skip stats
/// rather than `fetch` under a blocked worker. Native executes regardless.
pub(crate) fn plan_reads_no_files(plan: &Arc<dyn ExecutionPlan>) -> bool {
    let any: &dyn std::any::Any = plan.as_ref();
    if any.is::<crate::exec::LoadExec>() || any.is::<crate::exec::FileListingExec>() {
        return false;
    }
    plan.children().iter().all(|c| plan_reads_no_files(c))
}
