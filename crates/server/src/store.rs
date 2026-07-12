//! Storage abstraction traits.
//!
//! These traits live in `unitycatalog-common` so storage backends can implement
//! them without depending on this server crate. They are re-exported here to keep
//! the historical `unitycatalog_server::store::*` paths working.
pub use unitycatalog_common::store::{
    ObjectStoreAdapter, ProvidesResourceStore, ResourceStore, ResourceStoreReader,
};
