//! Session configuration for the Delta engine — the single source of truth for the DataFusion
//! `SessionConfig`/`SessionContext` the scan + snapshot-construction paths require.
//!
//! The engine drives kernel SSA reconciliation plans against a caller-supplied [`Session`], and
//! the same session then plans the compiled scan `LogicalPlan`. For that to be correct the session
//! must carry a specific config; historically every caller (`query-wasm`'s `build_query_session`,
//! the integration-test helpers) hand-copied that config and kept it in sync by comment. This
//! module centralizes it:
//!
//!   * [`configure_delta_engine_config`] / [`delta_engine_session_config`] — build the config.
//!   * [`install_delta_engine`] — fold the config into an existing [`SessionState`], preserving its
//!     registered object stores / catalogs / UDFs (mirrors the sibling headwaters
//!     `with_lineage` installer).
//!   * [`DeltaEngineSessionExt::with_delta_engine`] — the ergonomic consume-and-return
//!     `SessionContext -> SessionContext` extension, mirroring `SessionContext::enable_url_table`.
//!   * [`delta_engine_session`] — one-call build-config-and-register-store, the replacement for the
//!     hand-rolled `build_query_session` / `session_with_store` helpers.
//!   * [`validate_delta_engine_session`] — assert a session carries the load-bearing config.
//!
//! # WASM awareness
//!
//! Two of the knobs vary by build target and are exposed on [`DeltaEngineSessionOptions`]:
//! `schema_force_view_types` (off in the browser — the arrow-js IPC reader can't decode view
//! types) and `disable_repartition` (on in the browser — no threads). [`DeltaEngineSessionOptions`]
//! `::default()` picks the browser-safe preset on `wasm32` and the DataFusion-native preset
//! elsewhere, so `with_delta_engine(None)` "just works" per target.
//!
//! The load-bearing knobs applied unconditionally are NOT all wasm concerns — see
//! [`configure_delta_engine_config`].

use std::sync::Arc;

use datafusion::catalog::Session;
use datafusion::execution::context::SessionContext;
use datafusion::execution::runtime_env::RuntimeEnv;
use datafusion::execution::session_state::{SessionState, SessionStateBuilder};
use datafusion_common::Result as DfResult;
use datafusion_execution::config::SessionConfig;
use delta_kernel::object_store::ObjectStore;
use url::Url;

use crate::error::plan_compilation;

/// Tunables the Delta engine's DataFusion session may vary by build target / caller.
///
/// The load-bearing reconciliation config (leaf-pushdown off, single partition, no stats) is
/// applied unconditionally by [`configure_delta_engine_config`] and is deliberately NOT exposed
/// here — those are correctness requirements, not preferences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeltaEngineSessionOptions {
    /// `datafusion.execution.parquet.schema_force_view_types`. `false` on the browser path —
    /// arrow-js's IPC reader can't decode `Utf8View`/`BinaryView` in any published release
    /// (mangrove #28) — and `true` for native DataFusion, which handles view types natively.
    pub schema_force_view_types: bool,
    /// Disable every repartition pass plus round-robin repartition. Required on wasm, where
    /// multi-partition repartition tasks never run (no threads); harmless-but-unnecessary
    /// natively, where DataFusion may repartition for parallelism.
    pub disable_repartition: bool,
}

impl DeltaEngineSessionOptions {
    /// Native preset: view types on, repartition left to DataFusion.
    pub const fn native() -> Self {
        Self {
            schema_force_view_types: true,
            disable_repartition: false,
        }
    }

    /// Browser / wasm preset: view types off (arrow-js IPC can't decode them; #28), repartition
    /// off (single-threaded runtime).
    pub const fn wasm() -> Self {
        Self {
            schema_force_view_types: false,
            disable_repartition: true,
        }
    }

    /// Set [`Self::schema_force_view_types`].
    pub const fn with_schema_force_view_types(mut self, value: bool) -> Self {
        self.schema_force_view_types = value;
        self
    }

    /// Set [`Self::disable_repartition`].
    pub const fn with_disable_repartition(mut self, value: bool) -> Self {
        self.disable_repartition = value;
        self
    }
}

impl Default for DeltaEngineSessionOptions {
    /// Build-target-dependent: the browser preset ([`Self::wasm`]) on `wasm32`, the native preset
    /// ([`Self::native`]) elsewhere. Lets `with_delta_engine(None)` do the right thing per target.
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self::wasm()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::native()
        }
    }
}

/// Apply the Delta engine's session config onto `config`: the unconditional, load-bearing knobs
/// plus the caller-selected [`DeltaEngineSessionOptions`]. This is the single source of truth for
/// engine session config — every caller and test helper should route through it rather than
/// setting `options_mut()` by hand.
///
/// # Unconditional knobs (both targets)
///
/// These are correctness requirements, not preferences, and are **not** wasm-specific:
///
/// **`collect_statistics = false` — keep DataFusion's collector off the `ListingTable` path.**
/// DataFusion's parquet stats collector mishandles Delta column-mapping / field-id renamed columns:
/// it stamps missing-by-logical-name columns as all-null, which the projection folds to
/// `Literal::NULL` *before* the field-id rename applies — producing wrong results. This flag governs
/// only DataFusion's own `ListingTableProvider` collector, which is live on this crate's scan path
/// **only** for checkpoint / manifest *metadata* parquet reads (`NodeKind::Scan` ->
/// `scan_to_listing_logical_plan`, reached whenever the scanned table has a checkpoint); the
/// user-data files never use it (they read through the crate's own `LoadExec`). We keep it off so
/// that metadata `ListingTable` never runs DF's collector.
///
/// Per-file statistics for the **data** files are now sourced directly from the kernel's
/// metadata-stats state machine, remapped physical->logical, and attached to each
/// `PartitionedFile` (`crate::compile::stats`, wired in `provider::scan`) — a path entirely
/// independent of this flag. So disabling DF's collector costs nothing: correct data-file stats
/// still flow, and the kernel additionally does its own file-level data skipping.
///
/// **`enable_leaf_expression_pushdown = false` — kernel-integration compiler workaround.** Runs
/// identically native and wasm. It works around a name-qualification collision in *our own* SSA ->
/// `LogicalPlan` compiler: the FSR replay shape is a `Filter` reading nested fields over a `Project`
/// that builds the Delta action structs via `named_struct`, and DataFusion's leaf-expression-pushdown
/// pass inlines the whole struct into every filter leaf, producing a schema that carries both the
/// qualified `scan."metaData"` (from the scan alias) and the unqualified `"metaData"` (the projection
/// output) — an `AmbiguousReference` (apache/datafusion#20432). The proper fix is at the compile
/// layer (neutralize the collision); until then we disable the pass, which costs essentially nothing
/// on these plans. Tracked in mangrove#123.
///
/// # Single-partition
///
/// `target_partitions = 1` is applied only when [`DeltaEngineSessionOptions::disable_repartition`]
/// is set (the wasm preset) — it is a single-threaded-runtime / perf knob, **not** a correctness
/// requirement, because single-partition is now enforced *structurally* where it matters: the
/// consume-sink drain coalesces to one partition before reading
/// ([`DataFusionExecutor`](crate::DataFusionExecutor)), and the internal `LoadExec` leaf declares
/// single-partition output. So native (`disable_repartition = false`) keeps intra-plan parallelism
/// and concurrent file reads while staying correct.
pub fn configure_delta_engine_config(
    mut config: SessionConfig,
    options: &DeltaEngineSessionOptions,
) -> SessionConfig {
    // --- Unconditional, load-bearing (both targets); see fn docs ---
    // Keep DF's ListingTable stats collector off the checkpoint-metadata scan path (it mishandles
    // field-id renamed columns). Data-file stats come from the kernel via `PartitionedFile`,
    // independent of this flag (`crate::compile::stats`).
    config.options_mut().execution.collect_statistics = false;
    // Kernel-integration compiler workaround (apache/datafusion#20432, mangrove#123).
    config
        .options_mut()
        .optimizer
        .enable_leaf_expression_pushdown = false;

    // --- Caller-selected options ---
    config
        .options_mut()
        .execution
        .parquet
        .schema_force_view_types = options.schema_force_view_types;
    if options.disable_repartition {
        // Single-threaded runtime: force one partition and disable every repartition pass. Not a
        // correctness requirement (the drain coalesces structurally); native leaves this off to
        // keep parallelism.
        config.options_mut().execution.target_partitions = 1;
        config = config
            .with_round_robin_repartition(false)
            .with_repartition_joins(false)
            .with_repartition_aggregations(false)
            .with_repartition_windows(false)
            .with_repartition_sorts(false)
            .with_repartition_file_scans(false);
    }
    config
}

/// A fresh [`SessionConfig`] configured for the Delta engine. Sugar for
/// [`configure_delta_engine_config`] over `SessionConfig::new()`.
pub fn delta_engine_session_config(options: &DeltaEngineSessionOptions) -> SessionConfig {
    configure_delta_engine_config(SessionConfig::new(), options)
}

/// Rebuild `state` with the Delta engine config folded into its [`SessionConfig`], preserving the
/// state's registered object stores, catalogs, and UDFs.
///
/// This is the [`SessionState`] -> [`SessionState`] core installer (mirrors the headwaters
/// `OpenLineageBuilder::instrument` idiom): callers who already hold a configured state — e.g. one
/// with an object store registered — can layer the engine config on top without losing it.
pub fn install_delta_engine(
    state: SessionState,
    options: &DeltaEngineSessionOptions,
) -> SessionState {
    let config = configure_delta_engine_config(state.config().clone(), options);
    SessionStateBuilder::from(state).with_config(config).build()
}

/// [`SessionContext`] ergonomics for the Delta engine, mirroring
/// `SessionContext::enable_url_table(self) -> Self` and the headwaters `with_lineage` extension.
pub trait DeltaEngineSessionExt: Sized {
    /// Configure this context for the Delta engine, preserving already-registered object stores.
    /// `None` uses [`DeltaEngineSessionOptions::default`] (build-target-dependent).
    fn with_delta_engine(self, options: impl Into<Option<DeltaEngineSessionOptions>>) -> Self;
}

impl DeltaEngineSessionExt for SessionContext {
    fn with_delta_engine(self, options: impl Into<Option<DeltaEngineSessionOptions>>) -> Self {
        let options = options.into().unwrap_or_default();
        SessionContext::new_with_state(install_delta_engine(self.state(), &options))
    }
}

/// Build a fresh single-partition [`SessionContext`] configured for the Delta engine with `store`
/// registered under `table_url`'s origin (`scheme://authority/`).
///
/// The one-call replacement for the hand-rolled `build_query_session` / test `session_with_store`
/// helpers: it applies [`delta_engine_session_config`] and registers the object store the same way
/// the kernel/DataFusion resolve stores (by [`object_store::ObjectStoreUrl`](datafusion::execution::object_store::ObjectStoreUrl)
/// authority).
pub fn delta_engine_session(
    store: Arc<dyn ObjectStore>,
    table_url: &Url,
    options: &DeltaEngineSessionOptions,
) -> SessionContext {
    let ctx = SessionContext::new_with_config_rt(
        delta_engine_session_config(options),
        Arc::new(RuntimeEnv::default()),
    );
    ctx.runtime_env()
        .register_object_store(&origin_of(table_url), store);
    ctx
}

/// Assert `session` carries the load-bearing Delta engine config, returning a descriptive error
/// naming each mismatched knob.
///
/// Checks the three unconditional knobs ([`configure_delta_engine_config`]) plus that
/// `schema_force_view_types` matches `expected_force_view_types` (the provider passes its own
/// [`DeltaSsaScanConfig`](crate::DeltaSsaScanConfig) value so the physical parquet reader's output
/// type agrees with what the caller — e.g. a browser IPC reader — can decode). This is a hard
/// error, not an auto-repair: a misconfigured session is fixed by building it via
/// [`delta_engine_session`] / [`DeltaEngineSessionExt::with_delta_engine`], not by silently
/// mutating planning behavior underneath the caller.
pub fn validate_delta_engine_session(
    session: &dyn Session,
    expected_force_view_types: bool,
) -> DfResult<()> {
    let options = session.config_options();
    let mut problems = Vec::new();

    // Only the correctness-critical knobs are validated. `target_partitions` is deliberately NOT
    // checked: single-partition is enforced structurally (the consume drain coalesces), so a
    // multi-partition native session is valid.
    if options.optimizer.enable_leaf_expression_pushdown {
        problems.push(
            "optimizer.enable_leaf_expression_pushdown=true (must be false; the FSR replay shape \
             trips push_down_leaf_projections — apache/datafusion#20432)"
                .to_string(),
        );
    }
    if options.execution.collect_statistics {
        problems.push(
            "execution.collect_statistics=true (must be false; keeps DF's ListingTable collector \
             off the checkpoint-metadata scan path, which mishandles column-mapping/field-id \
             renamed columns — data-file stats come from the kernel via PartitionedFile)"
                .to_string(),
        );
    }
    let force_view = options.execution.parquet.schema_force_view_types;
    if force_view != expected_force_view_types {
        problems.push(format!(
            "execution.parquet.schema_force_view_types={force_view} disagrees with the expected \
             {expected_force_view_types} (set them to match — the wasm preview path uses false so \
             the browser arrow IPC reader can decode)"
        ));
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(plan_compilation(format!(
            "session is not configured for the Delta engine — {}. Configure it via \
             olai_delta_df::delta_engine_session / SessionContext::with_delta_engine",
            problems.join("; ")
        )))
    }
}

/// Return `url` reduced to its origin (`scheme://authority/`): path `/`, no query/fragment. This is
/// the authority under which the kernel/DataFusion resolve the object store, so it is where the
/// store must be registered.
fn origin_of(url: &Url) -> Url {
    let mut base = url.clone();
    base.set_path("/");
    base.set_query(None);
    base.set_fragment(None);
    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_and_wasm_presets_differ_only_on_options() {
        let native = DeltaEngineSessionOptions::native();
        assert!(native.schema_force_view_types);
        assert!(!native.disable_repartition);

        let wasm = DeltaEngineSessionOptions::wasm();
        assert!(!wasm.schema_force_view_types);
        assert!(wasm.disable_repartition);
    }

    #[test]
    fn configure_sets_correctness_knobs_unconditionally() {
        // The correctness-critical knobs are set for both presets, regardless of options.
        for options in [
            DeltaEngineSessionOptions::native(),
            DeltaEngineSessionOptions::wasm(),
        ] {
            let config = delta_engine_session_config(&options);
            let opts = config.options();
            assert!(!opts.optimizer.enable_leaf_expression_pushdown);
            assert!(!opts.execution.collect_statistics);
            // The option is honored.
            assert_eq!(
                opts.execution.parquet.schema_force_view_types,
                options.schema_force_view_types
            );
        }
    }

    #[test]
    fn target_partitions_is_gated_on_disable_repartition() {
        // Single-partition is a wasm/perf knob (correctness is structural in the drain), so it is
        // forced only when repartition is disabled.
        let wasm = delta_engine_session_config(&DeltaEngineSessionOptions::wasm());
        assert_eq!(wasm.options().execution.target_partitions, 1);

        // Native leaves `target_partitions` untouched at the SessionConfig default (host
        // parallelism) rather than forcing 1. Compare against a fresh config so the assertion holds
        // even on a single-core runner where the default happens to be 1.
        let native = delta_engine_session_config(&DeltaEngineSessionOptions::native());
        assert_eq!(
            native.options().execution.target_partitions,
            SessionConfig::new().options().execution.target_partitions,
            "native must not override target_partitions"
        );
    }

    #[test]
    fn validate_accepts_a_configured_session_and_rejects_a_default_one() {
        // A session built via the convenience config passes validation.
        let ctx = SessionContext::new_with_config(delta_engine_session_config(
            &DeltaEngineSessionOptions::wasm(),
        ));
        validate_delta_engine_session(&ctx.state(), false)
            .expect("a wasm-configured session must validate");

        // A bare session fails, and the message names the leaf-pushdown knob.
        let bare = SessionContext::new();
        let err = validate_delta_engine_session(&bare.state(), false)
            .expect_err("a default session must fail validation");
        assert!(
            err.to_string().contains("enable_leaf_expression_pushdown"),
            "error should name the leaf-pushdown knob, got: {err}"
        );
    }

    #[test]
    fn validate_flags_a_view_type_mismatch() {
        // Configured for wasm (view types off) but validated expecting them on → mismatch.
        let ctx = SessionContext::new_with_config(delta_engine_session_config(
            &DeltaEngineSessionOptions::wasm(),
        ));
        let err = validate_delta_engine_session(&ctx.state(), true)
            .expect_err("view-type mismatch must fail");
        assert!(err.to_string().contains("schema_force_view_types"));
    }
}
