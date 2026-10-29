//! Unity Catalog integration (feature `unity`).
//!
//! The rest of the crate depends on no catalog's models. This module is where
//! Unity Catalog comes in:
//!
//! - [`source`]: a UC REST server as a [`CatalogSource`](crate::source::CatalogSource),
//!   with delegated (catalog-vended) storage access;
//! - [`admin`]: the provider admin API in the UC REST shape
//!   (`/api/2.1/unity-catalog/shares|recipients|credentials|external-locations`).
//!
//! A host with its own catalog and admin surface builds without this feature
//! and implements those parts itself.

pub mod admin;
pub mod source;
