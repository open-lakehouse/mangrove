//! DataFusion integration for Unity Catalog.
//!
//! This crate bridges Unity Catalog to Apache DataFusion so a UC-governed
//! catalog can be queried through DataFusion's `SessionContext`. It also owns the
//! Delta-kernel-backed query path — reading the Delta transaction log and managed
//! tables — so downstream server crates can depend on the query machinery without
//! pulling the git-pinned `delta-kernel` dependency directly.
//!
//! # Layout
//!
//! - [`catalog`] — a DataFusion `CatalogProvider` over the UC hierarchy.
//! - [`storage`] — [`RoutingObjectStore`], which routes object-store requests to
//!   the backend for each `uc://`/cloud URL.
//! - `log_explorer` — the Delta transaction log exposed as DataFusion tables
//!   (raw + reconciled), read through `delta-kernel` (feature `delta`).
//! - `managed` / `sql` — the managed `CREATE TABLE` path and UC DDL planner
//!   (feature `delta`).
//! - `metric_view` — Unity Catalog metric-view support (feature `metric-view`).
//!
//! # Feature flags
//!
//! `delta` enables the `delta-kernel`-backed log/managed/SQL modules;
//! `metric-view` enables metric-view planning. Without them, only the plain
//! catalog provider and object-store routing are compiled.

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
