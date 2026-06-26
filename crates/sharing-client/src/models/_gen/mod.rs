// @generated — do not edit by hand.
#![allow(unexpected_cfgs)]
#![allow(clippy::empty_docs)]
use std::collections::HashMap;
pub type PropertyMap = HashMap<String, serde_json::Value>;
pub mod open_sharing {
    pub mod v1 {
        include!("./open_sharing.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./open_sharing.v1.tonic.rs");
    }
}
#[cfg(feature = "axum")]
pub mod extractors;
