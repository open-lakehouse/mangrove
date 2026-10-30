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
//!   mapped onto — a static configuration, a Unity Catalog REST server
//!   (feature `unity`), or your own;
//! - [`catalog`] and [`access`]: the recipient-facing share catalog, and storage
//!   access that is either *delegated* to the source catalog (no stored secrets)
//!   or *local* (sealed credentials, minted by `olai-credential-vending`);
//! - [`secrets`]: envelope encryption with lazy key rotation;
//! - [`recipients`]: recipient tokens with activation links;
//! - [`embed`]: the socket-less [`Router`](axum::Router) any host can serve;
//! - `unity` (feature `unity`): the Unity Catalog source and the provider admin
//!   API in the UC REST shape.
//!
//! # Without Unity Catalog
//!
//! With `default-features = false` the crate depends on no catalog's models:
//! the host core, the static source and storage access (over the
//! catalog-independent trestle credential vending crates) remain. A host with its own
//! catalog implements [`CatalogSource`](source::CatalogSource) and manages
//! shares and recipients through the [`ShareStore`](store::ShareStore) (or its
//! own admin surface), then serves [`embed::router`].
//!
//! See `crates/sharing-api/ARCHITECTURE.md` for the design.

pub mod access;
pub mod catalog;
pub mod config;
pub mod embed;
pub mod recipients;
pub mod secrets;
pub mod source;
pub mod store;
mod tokens;
#[cfg(feature = "unity")]
pub mod unity;

pub use config::Config;
pub use open_sharing_api::{Error, Result};
