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
use datafusion_execution::TaskContext;
use datafusion_physical_plan::ExecutionPlan;
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

/// Minimal executor over a caller-supplied [`Session`]: the executor borrows the session for
/// DataFusion compile/optimize/lower (`create_physical_plan`) and caches its [`TaskContext`] for
/// [`ExecutionPlan::execute`] calls. Carries no kernel engine and owns no session of its own.
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
pub struct DataFusionExecutor<'a> {
    session: &'a dyn Session,
    task_ctx: Arc<TaskContext>,
}

impl<'a> DataFusionExecutor<'a> {
    /// Build an executor that drives against `session`, caching its [`TaskContext`].
    ///
    /// `session` is borrowed for the executor's lifetime; the executor holds nothing `!Send` (a
    /// `&dyn Session` is `Send + Sync`), so it may be built inside a synchronous `block_on` drive
    /// without making the surrounding future `!Send`.
    pub fn new(session: &'a dyn Session) -> Self {
        let task_ctx = session.task_ctx();
        Self { session, task_ctx }
    }

    // ================================================================
    // High-level SM and result-plan driving
    // ================================================================

    /// Drive `sm` until it terminates, executing any intermediate phase operations it yields
    /// (kernel-side decision plans, schema queries) and returning the SM's terminal value.
    ///
    /// The terminal value is whatever `R` the SM was constructed for: for read-style SMs that
    /// is typically a [`ResultPlan`] the caller compiles via
    /// [`Self::ssa_result_to_dataframe`].
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
                Ok(op) => self.run_phase(op, sm_id, sm_kind, step_name).await,
                Err(_) => Ok(EngineResponse::Empty),
            };
            match sm.submit(phase_result)? {
                NextStep::Continue => {}
                NextStep::Done(value) => return Ok(value),
            }
        }
    }

    /// Drive a coroutine that yields a [`ResultPlan`] and collect its terminal output into a
    /// `Vec<RecordBatch>`. SSA plans describe a single self-contained dataflow DAG; the compiled
    /// `LogicalPlan` is planned + executed against the caller's session.
    ///
    /// [`ResultPlan`]: delta_kernel::sm_plans::ir::plan::ResultPlan
    pub async fn drive_ssa_to_batches(
        &self,
        sm: CoroutineSM<delta_kernel::sm_plans::ir::plan::ResultPlan>,
    ) -> Result<Vec<RecordBatch>, DeltaError> {
        let rp = self.drive_to_completion(sm).await?;
        self.collect_result_plan(&rp).await
    }

    /// Compile a [`ResultPlan`] to a `LogicalPlan`, plan it against the caller's session, and
    /// collect the results into a `Vec<RecordBatch>`. Useful for callers that already hold a
    /// `ResultPlan` (for example after driving a coroutine by hand) and don't need the
    /// `CoroutineSM` wrapping that [`Self::drive_ssa_to_batches`] provides; also the canonical
    /// entry point for tests that construct SSA plans directly without an SM.
    ///
    /// This drives everything through the [`Session`] trait (`create_physical_plan` +
    /// [`collect`](datafusion_physical_plan::collect)) — no `DataFrame`, so it never needs to
    /// recover a concrete `SessionState` from the borrowed `&dyn Session`. Callers that want the
    /// bare plan instead (to apply projection/limit and plan it themselves) use
    /// [`Self::compile_result_plan`].
    ///
    /// [`ResultPlan`]: delta_kernel::sm_plans::ir::plan::ResultPlan
    pub async fn collect_result_plan(
        &self,
        rp: &delta_kernel::sm_plans::ir::plan::ResultPlan,
    ) -> Result<Vec<RecordBatch>, DeltaError> {
        let logical = self.compile_result_plan(rp)?;
        let physical = self
            .session
            .create_physical_plan(&logical)
            .await
            .into_delta()?;
        datafusion_physical_plan::collect(physical, Arc::clone(&self.task_ctx))
            .await
            .into_delta()
    }

    /// Compile a [`ResultPlan`] to a bare [`LogicalPlan`], unbound to any session.
    ///
    /// Unlike [`Self::collect_result_plan`], this does not plan or execute — the caller (e.g.
    /// [`crate::DeltaSsaTableProvider::scan`]) plans it against the *scan's* session so the
    /// object store and config are the caller's, and can splice projection/limit on top first.
    /// Compilation itself is session-independent: it lowers SSA nodes to logical operators and
    /// `LoadTableProvider`s.
    ///
    /// [`ResultPlan`]: delta_kernel::sm_plans::ir::plan::ResultPlan
    /// [`LogicalPlan`]: datafusion_expr::LogicalPlan
    pub fn compile_result_plan(
        &self,
        rp: &delta_kernel::sm_plans::ir::plan::ResultPlan,
    ) -> Result<datafusion_expr::LogicalPlan, DeltaError> {
        let ctx = CompileContext {
            sm_id: crate::next_sm_id(),
            sm_kind: "standalone",
            step_name: "compile_result_plan",
        };
        compile_ssa(&rp.plan.stmts, rp.result, &ctx).into_delta()
    }

    /// Drive a combined metadata + data scan and collect the data rows.
    ///
    /// Sugar for `self.drive_ssa_to_batches(scan.scan_state_machine()?)`.
    pub async fn scan_data(&self, scan: &Scan) -> Result<Vec<RecordBatch>, DeltaError> {
        self.drive_ssa_to_batches(scan.scan_state_machine()?).await
    }

    /// Drive a metadata-only scan and collect the live-actions rows.
    ///
    /// Sugar for `self.drive_ssa_to_batches(scan.scan_metadata_state_machine()?)`.
    pub async fn scan_metadata(&self, scan: &Scan) -> Result<Vec<RecordBatch>, DeltaError> {
        self.drive_ssa_to_batches(scan.scan_metadata_state_machine()?)
            .await
    }

    /// Drive a Full State Reconstruction and collect the reconciled-actions rows.
    ///
    /// Sugar for `self.drive_ssa_to_batches(fsr.state_machine()?)`.
    pub async fn full_state(&self, fsr: &FullState) -> Result<Vec<RecordBatch>, DeltaError> {
        self.drive_ssa_to_batches(fsr.state_machine()?).await
    }

    /// Build a kernel [`SnapshotRef`] from a pre-listed [`LogSegment`], **async-native and
    /// engine-free**: drive the [`SnapshotPm`] state machine to resolve `(Protocol, Metadata)`
    /// (log replay streamed lazily through this executor's session object store — commits + any
    /// checkpoint parquet, short-circuiting once both are found), then assemble the snapshot via
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
        log_segment: Arc<LogSegment>,
        table_root: Url,
    ) -> Result<SnapshotRef, DeltaError> {
        let sm = SnapshotPm::for_log_segment(Arc::clone(&log_segment)).state_machine()?;
        let (protocol, metadata) = self.drive_to_completion(sm).await?;
        let log_segment = Arc::unwrap_or_clone(log_segment);
        let snapshot = Snapshot::from_parts(table_root, log_segment, protocol, metadata)
            .map_err(KernelErrAsDelta::into_delta_default)?;
        Ok(Arc::new(snapshot))
    }

    /// Execute a single [`EngineRequest`] against the executor and return the resulting
    /// [`EngineResponse`]. Used internally by [`Self::drive_to_completion`] and exposed for
    /// callers (typically tests) that need to drive an individual phase op directly.
    pub async fn execute_step(&self, op: EngineRequest) -> Result<EngineResponse, EngineError> {
        self.run_phase(op, crate::next_sm_id(), "standalone", "execute")
            .await
    }

    /// Execute one [`EngineRequest`], stamping any `Consume` handles minted during the run
    /// with `(sm_id, sm_kind, step_name)`.
    async fn run_phase(
        &self,
        op: EngineRequest,
        sm_id: Uuid,
        sm_kind: &'static str,
        step_name: &'static str,
    ) -> Result<EngineResponse, EngineError> {
        match op {
            EngineRequest::SchemaQuery(node) => self.read_footer_schema(&node).await,
            EngineRequest::Consume {
                stmts,
                terminal,
                sink,
            } => {
                let finished = self
                    .run_consume(&stmts, terminal, &sink, sm_id, sm_kind, step_name)
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
        stmts: &[delta_kernel::sm_plans::ir::plan::PlanNode],
        terminal: delta_kernel::sm_plans::ir::plan::Ref,
        sink: &ConsumeSink,
        sm_id: Uuid,
        sm_kind: &'static str,
        step_name: &'static str,
    ) -> Result<FinishedHandle, DataFusionError> {
        let ctx = CompileContext {
            sm_id,
            sm_kind,
            step_name,
        };
        let logical = compile_ssa(stmts, terminal, &ctx)?;
        // Plan against the caller's session. `Session::create_physical_plan` runs the logical
        // optimizer first, so this subsumes the previous explicit `optimize` + `create_physical_plan`
        // — and it uses the caller's config, scalar functions, and execution_props, keeping the
        // reconciliation drive consistent with the final scan plan the caller runs.
        let physical = self.session.create_physical_plan(&logical).await?;
        self.drain_consume_sink(physical, sink, &ctx).await
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
    async fn read_footer_schema(&self, node: &SchemaQuery) -> Result<EngineResponse, EngineError> {
        self.read_footer_schema_inner(node)
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
        node: &SchemaQuery,
    ) -> Result<KernelSchemaRef, DataFusionError> {
        let url =
            Url::parse(&node.file_path).map_err(|e| DataFusionError::External(Box::new(e)))?;
        // Resolve the caller-registered object store for this URL's authority (the same store the
        // scan path reads through), then footer-read via DataFusion's async parquet reader.
        let listing = datafusion_datasource::ListingTableUrl::parse(url.as_str())?;
        let object_store_url = listing.object_store();
        let store = self
            .task_ctx
            .runtime_env()
            .object_store(&object_store_url)?;
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
        physical: Arc<dyn ExecutionPlan>,
        sink: &ConsumeSink,
        ctx: &CompileContext,
    ) -> Result<FinishedHandle, DataFusionError> {
        let mut handle = sink.new_handle(ctx.sm_id, ctx.sm_kind, ctx.step_name);
        // Consume sinks are single-partition by construction; read partition 0 directly without
        // coalesce.
        let mut stream = physical.execute(0, Arc::clone(&self.task_ctx))?;
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
