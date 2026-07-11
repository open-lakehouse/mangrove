pub mod catalog;
// Delta transaction log exposed as DataFusion tables (raw + reconciled), read
// through delta-kernel. Rides the `delta` feature for the kernel deps.
#[cfg(feature = "delta")]
pub mod log_explorer;
#[cfg(feature = "delta")]
pub mod managed;
// Unity Catalog DDL statements + planner. The managed `CREATE TABLE` path calls
// into `managed`, so the module rides the same `delta` feature.
#[cfg(feature = "metric-view")]
pub mod metric_view;
#[cfg(feature = "delta")]
pub mod sql;
pub mod storage;

pub use self::storage::RoutingObjectStore;
