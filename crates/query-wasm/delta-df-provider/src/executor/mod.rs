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

use datafusion::dataframe::DataFrame;
use datafusion::execution::context::SessionContext;
use datafusion_common::error::DataFusionError;
use datafusion_execution::TaskContext;
use datafusion_execution::config::SessionConfig;
use datafusion_physical_plan::ExecutionPlan;
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

/// Minimal executor: a [`TaskContext`] for [`ExecutionPlan::execute`] calls and a
/// [`SessionContext`] for DataFusion compile/optimize/lower. Carries no kernel engine.
pub struct DataFusionExecutor {
    task_ctx: Arc<TaskContext>,
    session_ctx: SessionContext,
}

impl Default for DataFusionExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl DataFusionExecutor {
    /// Builds an executor over a single-partition [`SessionContext`] tuned for the kernel SSA
    /// scan/FSR replay shape, plus a default [`TaskContext`]. For a scan that actually reads
    /// remote files, drive against a [`SessionContext`] whose runtime env has the object store
    /// registered (see the `TableProvider` integration in `query-wasm`).
    pub fn new() -> Self {
        let session_ctx = SessionContext::new_with_config(Self::replay_session_config());
        Self {
            task_ctx: Arc::new(TaskContext::default()),
            session_ctx,
        }
    }

    /// The `SessionConfig` the SSA reconciliation / P&M replay shape requires.
    ///
    /// These three overrides are load-bearing for driving the reconciliation `Consume` plans,
    /// so any executor session that *runs* those plans (not just compiles a `ResultPlan` for the
    /// caller to run) must apply them — hence [`Self::new`] and [`Self::new_with_store`] share
    /// this, and callers who hand in their own session via [`Self::from_session`] are responsible
    /// for matching it.
    fn replay_session_config() -> SessionConfig {
        let mut session_config = SessionConfig::new();
        // DataFusion's leaf-expression-pushdown pass interacts badly with our FSR replay shape
        // (Filter over a Projection that builds a struct via named_struct): it inlines the full
        // struct definition into every Filter leaf, and downstream either CommonSubexprEliminate
        // dedups badly (duplicate `__common_expr_N`) or the qualified/unqualified struct field
        // refs (`scan."metaData"` vs `"metaData"`) become ambiguous. Keep it disabled
        // (apache/datafusion#20432).
        session_config
            .options_mut()
            .optimizer
            .enable_leaf_expression_pushdown = false;
        // Statistics collection is a session-level setting; disable it -- kernel does its own
        // file-level data skipping, and DF's parquet stats collector mishandles
        // column-mapping/field-id renamed columns (it stamps missing-by-logical-name columns as
        // all-null, which the projection then folds to Literal::NULL before the field-id rename
        // applies). See compile/logical/scan.rs for the full rationale.
        session_config.options_mut().execution.collect_statistics = false;
        // Force single-partition execution at the session level. The consume-sink drain reads
        // partition 0 only, and scan/FSR correctness does not depend on intra-file parallelism.
        // (Also required on wasm, where multi-partition repartition tasks never run.)
        session_config.options_mut().execution.target_partitions = 1;
        session_config
    }

    /// Builds an executor over a reconciliation-safe session (see [`Self::replay_session_config`])
    /// with `store` registered for `table_url`'s authority.
    ///
    /// Unlike [`Self::from_session`], this owns the session config, so it is the right constructor
    /// for **driving** replay SMs (scan/FSR/P&M) that both run the reconciliation `Consume`/
    /// `SchemaQuery` plans *and* read log/data files over `store` — e.g. snapshot construction,
    /// where a checkpointed table's P&M replay reads the checkpoint parquet during the drive.
    pub fn new_with_store(
        table_url: &url::Url,
        store: Arc<dyn delta_kernel::object_store::ObjectStore>,
    ) -> Self {
        let session_ctx = SessionContext::new_with_config(Self::replay_session_config());
        session_ctx
            .runtime_env()
            .register_object_store(table_url, store);
        let task_ctx = session_ctx.task_ctx();
        Self {
            task_ctx,
            session_ctx,
        }
    }

    /// Builds an executor over a caller-provided [`SessionContext`] (whose runtime env should
    /// have the scan's object store registered) and its task context.
    ///
    /// The caller's session config must match [`Self::replay_session_config`] when this executor
    /// will *drive* a replay SM (as opposed to only compiling a `ResultPlan`).
    pub fn from_session(session_ctx: SessionContext) -> Self {
        let task_ctx = session_ctx.task_ctx();
        Self {
            task_ctx,
            session_ctx,
        }
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

    /// Drive a coroutine that yields an [`ResultPlan`] and open its terminal output as a
    /// [`DataFrame`]. SSA plans describe a single self-contained dataflow DAG; the compiled
    /// `LogicalPlan` is wrapped directly in a [`DataFrame`] for the caller.
    ///
    /// [`ResultPlan`]: delta_kernel::sm_plans::ir::plan::ResultPlan
    pub async fn drive_ssa_to_dataframe(
        &self,
        sm: CoroutineSM<delta_kernel::sm_plans::ir::plan::ResultPlan>,
    ) -> Result<DataFrame, DeltaError> {
        let rp = self.drive_to_completion(sm).await?;
        self.ssa_result_to_dataframe(&rp)
    }

    /// Compile an [`ResultPlan`] to a [`DataFrame`]. Useful for callers that already hold
    /// a `ResultPlan` (for example after driving a coroutine by hand) and don't need the
    /// `CoroutineSM` wrapping that [`Self::drive_ssa_to_dataframe`] provides; also the
    /// canonical entry point for tests that construct SSA plans directly without an SM.
    ///
    /// [`ResultPlan`]: delta_kernel::sm_plans::ir::plan::ResultPlan
    pub fn ssa_result_to_dataframe(
        &self,
        rp: &delta_kernel::sm_plans::ir::plan::ResultPlan,
    ) -> Result<DataFrame, DeltaError> {
        let ctx = CompileContext {
            sm_id: crate::next_sm_id(),
            sm_kind: "standalone",
            step_name: "ssa_result_to_dataframe",
        };
        let logical = compile_ssa(&rp.plan.stmts, rp.result, &ctx).into_delta()?;
        Ok(DataFrame::new(self.session_ctx.state(), logical))
    }

    /// Compile a [`ResultPlan`] to a bare [`LogicalPlan`], unbound to any session.
    ///
    /// Unlike [`Self::ssa_result_to_dataframe`], this does not wrap the plan in a `DataFrame`
    /// tied to the executor's own `SessionContext` — the caller (e.g.
    /// [`crate::DeltaSsaTableProvider::scan`]) plans it against the *scan's* session so the
    /// object store and config are the caller's, not the executor's. Compilation itself is
    /// session-independent: it lowers SSA nodes to logical operators and `LoadTableProvider`s.
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

    /// Drive a combined metadata + data scan and return the data DataFrame.
    ///
    /// Sugar for `self.drive_ssa_to_dataframe(scan.scan_state_machine()?)`.
    pub async fn scan_data(&self, scan: &Scan) -> Result<DataFrame, DeltaError> {
        self.drive_ssa_to_dataframe(scan.scan_state_machine()?)
            .await
    }

    /// Drive a metadata-only scan and return the live-actions DataFrame.
    ///
    /// Sugar for `self.drive_ssa_to_dataframe(scan.scan_metadata_state_machine()?)`.
    pub async fn scan_metadata(&self, scan: &Scan) -> Result<DataFrame, DeltaError> {
        self.drive_ssa_to_dataframe(scan.scan_metadata_state_machine()?)
            .await
    }

    /// Drive a Full State Reconstruction and return the reconciled-actions DataFrame.
    ///
    /// Sugar for `self.drive_ssa_to_dataframe(fsr.state_machine()?)`.
    pub async fn full_state(&self, fsr: &FullState) -> Result<DataFrame, DeltaError> {
        self.drive_ssa_to_dataframe(fsr.state_machine()?).await
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
    /// The `SnapshotPm` SM is `!Send` (like the scan SM); callers confine the drive with
    /// `futures::executor::block_on` inside their `async` seam so nothing `!Send` crosses an
    /// `.await`.
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
        let df_state = self.session_ctx.state();
        let physical = df_state
            .create_physical_plan(&df_state.optimize(&logical)?)
            .await?;
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
