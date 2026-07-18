//! Delta-log surfaces as DataFusion table functions:
//! `delta_reconciled_log('c.s.t')` and `delta_log_actions('c.s.t')`.
//!
//! Addressing a log view by function argument rather than by a reserved table
//! name is collision-free: a query can inspect any table's log without a
//! registered-name clash, and the physical table rides in the argument (see
//! [`crate::engine::extract_log_udtf_calls`]).
//!
//! `TableFunctionImpl::call_with_args` is **synchronous**, but resolving a Unity
//! Catalog table (fetch, credentials, snapshot build) is async. The unified
//! entrypoint bridges the gap by *pre-resolving* every addressed table into a
//! [`ResolvedTables`] map before planning, so `call_with_args` here is a pure,
//! infallible-modulo-lookup map read that builds the provider from an
//! already-constructed snapshot.

use std::fmt;
use std::sync::Arc;

use datafusion::catalog::{Session, TableFunctionArgs, TableFunctionImpl, TableProvider};
use datafusion::common::{DataFusionError, Result as DfResult, plan_err};
use datafusion::logical_expr::Expr;
use olai_delta_df::{ActionsLogProvider, DeltaSsaScanConfig, ReconciledLogProvider, SnapshotRef};

use crate::catalog::ResolvedTables;
use crate::engine::{LogKind, parse_table_address, wasm_scan_config};

/// A registered log table function over a query-scoped [`ResolvedTables`] map.
///
/// One instance is registered per surface (`delta_reconciled_log` /
/// `delta_log_actions`); both share the same pre-resolved map. `Debug` is derived
/// through a manual impl because `ResolvedTables` holds opaque snapshots.
#[derive(Clone)]
pub struct DeltaLogUdtf {
    kind: LogKind,
    resolved: ResolvedTables,
}

impl DeltaLogUdtf {
    /// Build the UDTF for `kind` reading from the pre-populated `resolved` map.
    pub fn new(kind: LogKind, resolved: ResolvedTables) -> Self {
        Self { kind, resolved }
    }
}

impl fmt::Debug for DeltaLogUdtf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeltaLogUdtf")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl TableFunctionImpl for DeltaLogUdtf {
    fn call_with_args(&self, args: TableFunctionArgs) -> DfResult<Arc<dyn TableProvider>> {
        let target = single_string_literal(args.exprs())?;
        // Complete a partial argument the same way pre-resolution did, off the
        // session's default catalog/schema, so the map key matches.
        let (default_catalog, default_schema) = session_defaults(args.session());
        let address = parse_table_address(&target, Some(&default_catalog), Some(&default_schema))
            .map_err(|e| DataFusionError::Plan(e.to_string()))?;

        let resolved = self.resolved.get(&address).ok_or_else(|| {
            // Pre-resolution populates the map for every addressed table before
            // planning; a miss means the argument named a table the resolve pass
            // did not see (e.g. a UDTF call the extractor could not reach).
            DataFusionError::Plan(format!(
                "delta log table `{}` was not pre-resolved; the in-browser engine \
                 resolves log tables before planning",
                address.full_name()
            ))
        })?;

        build_log_provider(self.kind, resolved.snapshot, wasm_scan_config())
    }
}

/// Build the log `TableProvider` for `kind` from a resolved snapshot + scan config.
///
/// Shared with the direct-scan execution path so the two never diverge in which
/// provider a surface maps to.
pub fn build_log_provider(
    kind: LogKind,
    snapshot: SnapshotRef,
    config: DeltaSsaScanConfig,
) -> DfResult<Arc<dyn TableProvider>> {
    Ok(match kind {
        LogKind::Reconciled => Arc::new(ReconciledLogProvider::new(snapshot, config)?),
        LogKind::Actions => Arc::new(ActionsLogProvider::new(snapshot, config)?),
    })
}

/// Extract the single string-literal argument of a UDTF call, erroring on wrong
/// arity or a non-string argument.
///
/// This mirrors [`crate::engine::extract_log_udtf_calls`]'s AST-level check but at
/// the resolved-`Expr` level DataFusion hands the function at plan time.
fn single_string_literal(exprs: &[Expr]) -> DfResult<String> {
    let [Expr::Literal(scalar, _)] = exprs else {
        return plan_err!(
            "a delta log table function takes exactly one 'catalog.schema.table' \
             string argument, got {} argument(s)",
            exprs.len()
        );
    };
    match scalar.try_as_str() {
        Some(Some(s)) => Ok(s.to_string()),
        _ => plan_err!("delta log table function argument must be a non-null string literal"),
    }
}

/// The session's configured default catalog and schema, for completing a partial
/// table argument to match the pre-resolved map key.
fn session_defaults(session: &dyn Session) -> (String, String) {
    let catalog = &session.config_options().catalog;
    (
        catalog.default_catalog.clone(),
        catalog.default_schema.clone(),
    )
}
