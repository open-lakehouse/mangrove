//! Reusable handler patterns.
//!
//! Implementations of the per-resource handler traits that compose with or
//! delegate to other backends, rather than serving from the local store:
//!
//! - [`Authorized`] — a **decorator** that enforces this server's
//!   [`Policy`](crate::policy::Policy) in front of any inner handler. With the
//!   `proxy` feature the `unitycatalog-client` service clients implement the
//!   handler traits too, so wrapping one yields a policy-checked proxy to an
//!   upstream Unity Catalog instance.
//!
//! Future patterns (audit, cache, guard, federation) will live alongside and
//! stay ungated where they need only `server` + `common`.
//!
//! # Genericity
//!
//! Patterns here are generic over the request context `Cx` and never reference
//! [`RequestContext`](crate::api::RequestContext) directly, so they remain
//! portable if these modules are later lifted into a dedicated crate.

mod authorized;

pub use authorized::Authorized;
