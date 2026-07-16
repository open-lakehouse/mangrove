pub mod api;
mod codegen;
pub mod error;
pub mod handlers;
#[cfg(feature = "memory")]
pub mod memory;
pub mod policy;
pub mod rest;
pub mod services;
pub mod store;
pub mod telemetry;

// Deployable-binary support: config loading, the CLI/subcommand surface, and the
// server-launch wiring used by the `uc-server` binary (see `src/main.rs`). Gated
// behind `bin` so a plain library build doesn't pull the CLI/serve/store stack.
#[cfg(feature = "bin")]
pub mod cli;
#[cfg(feature = "bin")]
pub mod config;
#[cfg(feature = "bin")]
pub mod hybrid;
#[cfg(feature = "bin")]
pub mod run;

pub use crate::error::{Error, Result};
