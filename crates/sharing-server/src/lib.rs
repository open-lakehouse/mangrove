//! A composable Open Sharing / Delta Sharing server.
//!
//! The protocol itself — discovery, the table query path, credential endpoints,
//! recipient auth — comes from `olai-sharing-api`. This crate supplies a
//! deployable host for it:
//!
//! - [`store`]: the server's own state (shares, recipients, tokens, grants,
//!   sealed storage credentials) behind the [`ShareStore`](store::ShareStore)
//!   port, in memory or in Postgres;
//! - [`source`]: the [`CatalogSource`](source::CatalogSource) port shares are
//!   mapped onto — a Unity Catalog REST server or a static configuration;
//! - [`catalog`] and [`access`]: the recipient-facing share catalog, and storage
//!   access that is either *delegated* to the source catalog (no stored secrets)
//!   or *local* (sealed credentials, vended by `olai-uc-credentials`);
//! - [`secrets`]: envelope encryption with lazy key rotation;
//! - [`recipients`] and [`provider`]: recipient tokens with activation links, and
//!   the provider admin API;
//! - [`embed`]: the socket-less [`Router`](axum::Router) any host can serve.
//!
//! See `crates/sharing-api/ARCHITECTURE.md` for the design.

pub mod access;
pub mod catalog;
pub mod config;
pub mod embed;
pub mod provider;
pub mod recipients;
pub mod secrets;
pub mod source;
pub mod store;
mod tokens;

pub use config::Config;
pub use open_sharing_api::{Error, Result};
