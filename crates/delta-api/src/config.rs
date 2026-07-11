//! `getConfig` support: the capability-driven endpoint list and protocol-version
//! negotiation.
//!
//! Kept out of [`handler`](crate::handler) so the blanket `DeltaApiHandler` impl
//! stays focused on business logic and this small, pure logic is unit-testable in
//! isolation.

use crate::backend::DeltaCapabilities;
use crate::error::{DeltaApiError, DeltaApiResult};

/// Protocol versions this crate implements, **highest first**. Today: just `1.0`.
///
/// This is the single source of truth for both negotiation and the version string
/// returned to clients.
pub(crate) const SUPPORTED_VERSIONS: &[&str] = &["1.0"];

/// The endpoints served regardless of backend capability: the 10 always-present
/// operations. `getConfig` itself is deliberately **not** listed — it is the
/// bootstrap endpoint a client must already know to reach this response, not a
/// discoverable member of the surface it describes. The spurious `listTables`
/// (`GET .../tables`) is likewise absent: the UC Delta spec defines no such
/// operation.
///
/// Format is `"METHOD /v1/<path>"`, relative to the delta API root (no `/delta` or
/// `/api/2.1/unity-catalog/delta` prefix), per the ManagedTables spec.
pub(crate) const CORE_ENDPOINTS: &[&str] = &[
    "POST /v1/catalogs/{catalog}/schemas/{schema}/staging-tables",
    "POST /v1/catalogs/{catalog}/schemas/{schema}/tables",
    "GET /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}",
    "POST /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}",
    "DELETE /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}",
    "HEAD /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}",
    "GET /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/credentials",
    "POST /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/metrics",
    "GET /v1/staging-tables/{table_id}/credentials",
    "GET /v1/temporary-path-credentials",
];

/// The `renameTable` endpoint, advertised only when the backend supports rename.
const RENAME_ENDPOINT: &str = "POST /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/rename";

/// Build the advertised endpoint list for a backend's capabilities: the core
/// endpoints plus any capability-gated ones the backend opts into.
pub(crate) fn endpoints_for(caps: DeltaCapabilities) -> Vec<String> {
    let mut endpoints: Vec<String> = CORE_ENDPOINTS.iter().map(|s| s.to_string()).collect();
    if caps.rename {
        endpoints.push(RENAME_ENDPOINT.to_string());
    }
    endpoints
}

/// A parsed `major.minor` protocol version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Version {
    major: u32,
    minor: u32,
}

impl Version {
    /// Parse a `"major.minor"` string (e.g. `"1.0"`). Returns `None` for anything
    /// that isn't two non-negative integers separated by a single dot.
    fn parse(s: &str) -> Option<Version> {
        let (major, minor) = s.trim().split_once('.')?;
        Some(Version {
            major: major.trim().parse().ok()?,
            minor: minor.trim().parse().ok()?,
        })
    }

    /// Whether a client that lists this version as its *highest supported minor for
    /// this major* also supports `other`. Per the spec, listing `1.1` means the
    /// client supports every `1.0..=1.1` — same major, minor at least as high.
    fn covers(self, other: Version) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

/// Negotiate the protocol version: the highest version this crate supports that the
/// client also supports.
///
/// `client_versions` is the raw comma-separated `protocol-versions` query value:
/// each entry is the highest minor the client supports for that major (e.g.
/// `"1.1,2.3"` ⇒ supports `1.0-1.1` and `2.0-2.3`). Unparseable entries are ignored
/// so a client advertising a version we can't parse still negotiates against the
/// ones we can.
///
/// Returns the negotiated version string, or a 400
/// [`InvalidParameterValueException`](crate::models::DeltaErrorType::InvalidParameterValueException)
/// naming the supported versions when there is no overlap — as the spec mandates.
pub(crate) fn negotiate_version(client_versions: &str) -> DeltaApiResult<String> {
    let client: Vec<Version> = client_versions
        .split(',')
        .filter_map(Version::parse)
        .collect();

    // SUPPORTED_VERSIONS is highest-first, so the first server version any client
    // entry covers is the highest mutually supported one.
    for supported in SUPPORTED_VERSIONS {
        let server = Version::parse(supported).expect("SUPPORTED_VERSIONS entries are valid");
        if client.iter().any(|c| c.covers(server)) {
            return Ok((*supported).to_string());
        }
    }

    Err(DeltaApiError::invalid_argument(format!(
        "no mutually supported protocol version; server supports: {}",
        SUPPORTED_VERSIONS.join(", ")
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DeltaBackendError;

    #[test]
    fn negotiate_exact_match() {
        assert_eq!(negotiate_version("1.0").unwrap(), "1.0");
    }

    #[test]
    fn negotiate_higher_minor_covers_lower() {
        // A client whose highest 1.x is 1.1 supports 1.0.
        assert_eq!(negotiate_version("1.1").unwrap(), "1.0");
    }

    #[test]
    fn negotiate_picks_from_multi_major_list() {
        assert_eq!(negotiate_version("2.3,1.0").unwrap(), "1.0");
        assert_eq!(negotiate_version("1.1,2.3").unwrap(), "1.0");
    }

    #[test]
    fn negotiate_tolerates_whitespace_and_garbage() {
        assert_eq!(negotiate_version(" 1.0 ").unwrap(), "1.0");
        // Unparseable entries are skipped; the valid 1.0 still negotiates.
        assert_eq!(negotiate_version("nonsense, 1.0").unwrap(), "1.0");
    }

    #[test]
    fn negotiate_no_overlap_is_invalid_argument() {
        let err = negotiate_version("2.0,3.1").unwrap_err();
        assert!(
            matches!(err.0, DeltaBackendError::InvalidArgument(_)),
            "{err:?}"
        );
        // The message names the supported versions.
        assert!(err.to_string().contains("1.0"), "{err}");
    }

    #[test]
    fn negotiate_empty_and_garbage_only_is_400() {
        assert!(negotiate_version("").is_err());
        assert!(negotiate_version("garbage").is_err());
    }

    #[test]
    fn core_endpoints_omit_config_and_list_tables() {
        let core = endpoints_for(DeltaCapabilities::default());
        assert_eq!(core.len(), 10);
        assert!(!core.iter().any(|e| e == "GET /v1/config"));
        assert!(
            !core
                .iter()
                .any(|e| e == "GET /v1/catalogs/{catalog}/schemas/{schema}/tables"),
            "listTables must not be advertised"
        );
        assert!(!core.iter().any(|e| e == RENAME_ENDPOINT));
    }

    #[test]
    fn rename_capability_adds_the_rename_endpoint() {
        let with_rename = endpoints_for(DeltaCapabilities { rename: true });
        assert_eq!(with_rename.len(), 11);
        assert!(with_rename.iter().any(|e| e == RENAME_ENDPOINT));
    }
}
