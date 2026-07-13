// @generated — do not edit by hand.
#![allow(unexpected_cfgs)]
#![allow(clippy::empty_docs)]
#![allow(clippy::useless_borrows_in_formatting)]
#![allow(non_camel_case_types)]
#![allow(clippy::derivable_impls)]
use std::collections::HashMap;
pub type PropertyMap = HashMap<String, serde_json::Value>;
pub mod open_sharing {
    pub mod v1 {
        include!("./open_sharing.v1.rs");
    }
}
#[cfg(feature = "axum")]
pub mod extractors;
