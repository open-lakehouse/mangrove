// @generated — do not edit by hand.
pub use handler::ExternalLocationHandler;

#[cfg(feature = "authz")]
mod authz;
mod handler;
#[cfg(feature = "axum")]
pub mod server;
