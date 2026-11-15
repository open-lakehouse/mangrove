// @generated — do not edit by hand.
pub use handler::RegisteredModelHandler;

#[cfg(feature = "authz")]
mod authz;
mod handler;
#[cfg(feature = "axum")]
pub mod server;
