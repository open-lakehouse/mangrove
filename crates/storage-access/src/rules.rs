//! Shared building blocks for [`StorageAccessStore`](crate::StorageAccessStore)
//! backends, so every backend stamps records and reports rule violations alike.

use crate::Error;

/// A fresh record id.
pub fn new_id() -> String {
    uuid::Uuid::now_v7().hyphenated().to_string()
}

/// The current time, in milliseconds since the Unix epoch.
pub fn now_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// The error for a location URL overlapping the registered location `existing`.
pub fn overlap_error(url: &str, existing_name: &str, existing_url: &str) -> Error {
    Error::invalid_argument(format!(
        "external location url '{url}' overlaps existing external location \
         '{existing_name}' ('{existing_url}')"
    ))
}

/// The error for deleting a credential that external locations still use.
pub fn credential_in_use_error(name: &str, locations: &[String]) -> Error {
    Error::invalid_argument(format!(
        "credential '{name}' is used by external location(s) {}",
        locations
            .iter()
            .map(|l| format!("'{l}'"))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}
