//! The `storageAccess` capability helper.
//!
//! Kept as a tiny, pure module so both the server's capabilities endpoint and any
//! other consumer agree on the announced posture string without duplicating the
//! `"direct"` / `"proxy"` literals.

use crate::backend::ProxyCapabilities;

/// Posture announced when the proxy surface is not mounted: the browser reads/writes
/// storage directly (today's behavior).
pub const STORAGE_ACCESS_DIRECT: &str = "direct";

/// Posture announced when the proxy surface is mounted: the browser routes storage
/// bytes through the same-origin proxy.
pub const STORAGE_ACCESS_PROXY: &str = "proxy";

/// The `storageAccess` string for a set of capabilities.
pub fn storage_access(caps: ProxyCapabilities) -> &'static str {
    if caps.enabled {
        STORAGE_ACCESS_PROXY
    } else {
        STORAGE_ACCESS_DIRECT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posture_reflects_enabled() {
        assert_eq!(
            storage_access(ProxyCapabilities { enabled: false }),
            "direct"
        );
        assert_eq!(storage_access(ProxyCapabilities { enabled: true }), "proxy");
    }
}
