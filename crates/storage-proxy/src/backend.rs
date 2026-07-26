//! The `StorageProxyBackend` port and its request vocabulary.
//!
//! The port is intentionally tiny: it [authorizes](StorageProxyBackend::authorize)
//! a request (including the confused-deputy scope guard) and [opens](StorageProxyBackend::open)
//! a **verb-scoped, securable-prefixed** object store. All the HTTP/streaming
//! semantics live in the [handler](crate::handler), so a backend implementer only
//! decides *who may touch what* and *which credential-scoped store to hand back* —
//! never how bytes move.

use std::sync::Arc;

use object_store::{DynObjectStore, GetRange};
use url::Url;

use crate::error::ProxyResult;

/// The HTTP verb, which decides the credential scope: `Get`/`Head` open a
/// read-scoped store, `Put` a write-scoped one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyVerb {
    /// A body-returning read.
    Get,
    /// A metadata-only read.
    Head,
    /// A whole-object write.
    Put,
}

impl ProxyVerb {
    /// Whether this verb requires write access.
    pub fn is_write(self) -> bool {
        matches!(self, ProxyVerb::Put)
    }
}

/// What a request targets, parsed from the `{securable}` path segment.
///
/// The wire encoding is a single typed, colon-delimited path segment:
/// - `table:<catalog.schema.table>`
/// - `vol:<catalog.schema.volume>`
/// - `path:<percent-encoded cloud URL>` (e.g. `path:s3%3A%2F%2Fbucket%2Fprefix%2F`)
///
/// Keeping the securable in one path segment (no unescaped `/`) lets the router's
/// `{*key}` wildcard cleanly capture the object key as the remaining path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Securable {
    /// A managed/external table, by fully-qualified `catalog.schema.table` name.
    Table {
        /// The fully-qualified table name.
        full_name: String,
    },
    /// A volume, by fully-qualified `catalog.schema.volume` name.
    Volume {
        /// The fully-qualified volume name.
        full_name: String,
    },
    /// A raw cloud storage URL (`s3://`, `abfss://`, `gs://`, …).
    Path {
        /// The parsed cloud URL.
        url: Url,
    },
}

impl Securable {
    /// Parse a `{securable}` path segment (already percent-decoded by the router's
    /// path extractor) into a [`Securable`].
    ///
    /// Returns [`ProxyError::InvalidArgument`](crate::error::ProxyError::InvalidArgument)
    /// for an unknown kind prefix, an empty identifier, or a `path:` value that is
    /// not a valid URL.
    pub fn parse(segment: &str) -> ProxyResult<Securable> {
        use crate::error::ProxyError;

        let (kind, ident) = segment.split_once(':').ok_or_else(|| {
            ProxyError::InvalidArgument(format!(
                "securable must be `<kind>:<ident>` (got `{segment}`)"
            ))
        })?;
        if ident.is_empty() {
            return Err(ProxyError::InvalidArgument(format!(
                "securable `{kind}:` has an empty identifier"
            )));
        }
        match kind {
            "table" => Ok(Securable::Table {
                full_name: ident.to_string(),
            }),
            "vol" => Ok(Securable::Volume {
                full_name: ident.to_string(),
            }),
            "path" => {
                let url = Url::parse(ident).map_err(|e| {
                    ProxyError::InvalidArgument(format!("path securable is not a valid URL: {e}"))
                })?;
                Ok(Securable::Path { url })
            }
            other => Err(ProxyError::InvalidArgument(format!(
                "unknown securable kind `{other}` (expected table|vol|path)"
            ))),
        }
    }
}

/// A fully-parsed proxy request handed to the [port](StorageProxyBackend).
///
/// The handler has already parsed `Range`/`If-Match` from HTTP headers, so the
/// backend never touches axum or `http` types.
#[derive(Debug, Clone)]
pub struct ProxyReq {
    /// The verb (decides read vs write credential scope).
    pub verb: ProxyVerb,
    /// The target securable.
    pub securable: Securable,
    /// The object key **relative to the securable root** (the `{*key}` tail).
    pub key: String,
    /// The parsed `Range` header (`Get` only; `None` otherwise).
    pub range: Option<GetRange>,
    /// The parsed `If-Match` header value (`Put` only), relayed to storage as a
    /// conditional write.
    pub if_match: Option<String>,
}

/// What the server announces about its storage-access posture.
///
/// Surfaced by the server-level capabilities endpoint as
/// `storageAccess: "proxy" | "direct"`; extensible with further flags.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProxyCapabilities {
    /// Whether the proxy surface is mounted (announced as `"proxy"` when true).
    pub enabled: bool,
}

/// Read a forwarded end-user identity out of a backend's per-request context.
///
/// The [`StorageProxyBackend`] port is generic over the host's context type `Cx`,
/// but the client arm needs to forward the caller's identity to the upstream
/// Unity Catalog credential vend. This trait lets a backend extract that identity
/// from any `Cx` without the backend having to name the host's concrete context
/// type: the standalone binary's context is a `ForwardedIdentity` (which forwards
/// the validated reverse-proxy user), while a host that has no forwarded identity
/// (or the unit context `()` used by tests) simply returns `None`.
pub trait ForwardedUser {
    /// The forwarded end-user name, or `None` when the request is anonymous /
    /// carries no forwarded identity.
    fn forwarded_user(&self) -> Option<&str>;
}

/// The unit context carries no identity — always anonymous.
impl ForwardedUser for () {
    fn forwarded_user(&self) -> Option<&str> {
        None
    }
}

/// The backend port: authorize a request and open a verb-scoped, prefixed store.
///
/// `Cx` is the host's per-request context (mangrove uses its `RequestContext`; the
/// in-memory testing backend uses `()`). [`authorize`](Self::authorize) runs first
/// and MUST enforce both the caller's permission for the verb and the
/// confused-deputy path-scope guard; [`open`](Self::open) then returns a store
/// **prefixed at the securable root**, so a [`ProxyReq::key`] addresses relative to
/// it and cannot escape.
#[async_trait::async_trait]
pub trait StorageProxyBackend<Cx = ()>: Send + Sync + 'static {
    /// The proxy capabilities this backend advertises.
    fn capabilities(&self) -> ProxyCapabilities {
        ProxyCapabilities::default()
    }

    /// Authorize + scope-check the request. Runs before [`open`](Self::open).
    ///
    /// Must reject a caller lacking the verb's access level
    /// ([`ProxyError::PermissionDenied`](crate::error::ProxyError::PermissionDenied)
    /// / [`Unauthenticated`](crate::error::ProxyError::Unauthenticated)) and any key
    /// that escapes the securable's authorized root
    /// ([`ProxyError::OutOfScope`](crate::error::ProxyError::OutOfScope)).
    async fn authorize(&self, req: &ProxyReq, cx: &Cx) -> ProxyResult<()>;

    /// Open a store scoped to `req.securable` at the access level implied by the
    /// verb (`Get`/`Head` → read, `Put` → read-write). The returned store is
    /// **prefixed at the securable root**, so `req.key` addresses relative to it.
    ///
    /// `open` performs no authorization — that already happened in
    /// [`authorize`](Self::authorize).
    async fn open(&self, req: &ProxyReq, cx: &Cx) -> ProxyResult<Arc<DynObjectStore>>;
}

/// Reject an object key that could escape the securable root (confused-deputy
/// guard). Rejects `..` segments, absolute keys (leading `/`), backslashes, and
/// NUL bytes; tolerates `.`/empty segments.
///
/// This is layer 1 of the defense; prefix confinement (the store returned by
/// [`StorageProxyBackend::open`] is rooted at the securable) and verb-scoped
/// credentials are layers 2–3. Backends should call this from
/// [`authorize`](StorageProxyBackend::authorize).
pub fn reject_key_traversal(key: &str) -> ProxyResult<()> {
    use crate::error::ProxyError;

    if key.starts_with('/') {
        return Err(ProxyError::OutOfScope("absolute key not allowed".into()));
    }
    if key.contains('\\') || key.contains('\0') {
        return Err(ProxyError::OutOfScope(
            "key contains an illegal character".into(),
        ));
    }
    for segment in key.split('/') {
        if segment == ".." {
            return Err(ProxyError::OutOfScope(
                "key contains a `..` path traversal".into(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_table_and_volume() {
        assert_eq!(
            Securable::parse("table:main.default.events").unwrap(),
            Securable::Table {
                full_name: "main.default.events".into()
            }
        );
        assert_eq!(
            Securable::parse("vol:main.default.landing").unwrap(),
            Securable::Volume {
                full_name: "main.default.landing".into()
            }
        );
    }

    #[test]
    fn parse_path_url() {
        let s = Securable::parse("path:s3://bucket/prefix/").unwrap();
        match s {
            Securable::Path { url } => assert_eq!(url.as_str(), "s3://bucket/prefix/"),
            other => panic!("expected Path, got {other:?}"),
        }
    }

    #[test]
    fn parse_rejects_unknown_kind_and_empty_and_bad_url() {
        assert!(Securable::parse("bogus:x").is_err());
        assert!(Securable::parse("table:").is_err());
        assert!(Securable::parse("no-colon").is_err());
        assert!(Securable::parse("path:not a url").is_err());
    }

    #[test]
    fn traversal_guard() {
        assert!(reject_key_traversal("a/b/c.parquet").is_ok());
        assert!(reject_key_traversal("_delta_log/00000000000000000001.json").is_ok());
        assert!(reject_key_traversal("a/../../etc/passwd").is_err());
        assert!(reject_key_traversal("/absolute").is_err());
        assert!(reject_key_traversal("a\\b").is_err());
        // A leading `..` and a bare `..` segment are both rejected.
        assert!(reject_key_traversal("../sibling").is_err());
        assert!(reject_key_traversal("..").is_err());
        // `.` and empty segments are tolerated (normalized by object_store::Path).
        assert!(reject_key_traversal("a/./b").is_ok());
        assert!(reject_key_traversal("a//b").is_ok());
    }
}
