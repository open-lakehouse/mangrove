//! Reusable handler patterns.
//!
//! Implementations of the per-resource handler traits that compose with or
//! delegate to other backends, rather than serving from the local store:
//!
//! - `upstream` — **proxy leaves** that forward requests to an upstream Unity
//!   Catalog instance via `unitycatalog-client`, enforcing this server's
//!   [`Policy`](crate::policy::Policy) locally before delegating. Gated behind
//!   the `proxy` feature (pulls the client dependency), so the module and its
//!   links are only present in a build with that feature enabled.
//!
//! Future patterns (decorators that wrap an inner handler — audit, cache,
//! guard, federation) will live alongside and stay ungated where they need only
//! `server` + `common`.
//!
//! # Genericity
//!
//! Patterns here are generic over the request context `Cx` and never reference
//! [`RequestContext`](crate::api::RequestContext) directly, so they remain
//! portable if these modules are later lifted into a dedicated crate.

#[cfg(feature = "proxy")]
pub mod upstream;
