// @generated — do not edit by hand.
#![allow(unexpected_cfgs)]
#![allow(clippy::empty_docs)]
#![allow(non_camel_case_types)]
#![allow(clippy::derivable_impls)]
use std::collections::HashMap;
pub type PropertyMap = HashMap<String, serde_json::Value>;
pub mod open_sharing {
    pub mod v1 {
        include!("./../gen/open_sharing.v1.rs");
    }
}
