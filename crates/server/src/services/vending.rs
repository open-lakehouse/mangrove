//! Minting temporary credentials from registered catalog credentials.
//!
//! Vending goes through one trestle [`Vendor`] per handler, over a
//! [`CatalogResolver`]: a location resolves to the credential of the external
//! location governing it, or, for paths under the metastore's managed storage
//! root, to the credential configured for that root. The vendor keeps one cached
//! minter per registered credential and is cleared when credentials change (see
//! [`ServerHandler::forget_vended_credentials`](super::ServerHandler)), so rotated
//! or deleted secrets do not linger. Local (`file://`) storage, which no
//! credential governs, is minted by the [`LocalMinter`].
//!
//! Authorization is not done here: the callers authorize against the concrete
//! securable first.
//!
//! Neon object storage (an `aws_iam_role` credential with a Neon ARN, see
//! [`neon_arn`](unitycatalog_common::credentials_ext::neon_arn)) is minted by
//! trestle's *trusted* `NeonTrustedMinter`: its keys reach the whole branch. It
//! only mints when the server is built with the `neon-trusted-vending` feature
//! and [`NeonVending`] is enabled; otherwise a vend against such a credential is
//! a `FAILED_PRECONDITION`, while registering one still works.

use std::cell::Cell;
use std::sync::Arc;

use async_trait::async_trait;
use olai_credential_vending::registered::NeonStorage;
use olai_credential_vending::{
    CredentialMinter, CredentialResolver, LocalMinter, MintRequest, RegisteredCredential, Vendor,
    minter_for,
};
use olai_storage_credentials::{
    AccessLevel, StorageCredentials, StorageLocation, StorageScope, VendedCredential,
};
use unitycatalog_common::models::temporary_credentials::v1::TemporaryCredential;
use unitycatalog_handlers::{Principal, RequestContext};
use unitycatalog_storage_access::StorageAccessStore;

use super::location::StorageLocationUrl;
use crate::Result;

/// The metastore's managed storage root and the registered credential governing
/// it, for managed storage no external location covers.
#[derive(Debug, Clone)]
pub(crate) struct ManagedRootCredential {
    pub(crate) scope: StorageScope,
    pub(crate) credential_name: String,
}

/// Resolves a location to the registered credential allowed to mint for it: the
/// governing external location's credential, else the managed root's.
pub(crate) struct CatalogResolver {
    store: Arc<dyn StorageAccessStore>,
    managed_root: Option<ManagedRootCredential>,
}

impl CatalogResolver {
    pub(crate) fn new(
        store: Arc<dyn StorageAccessStore>,
        managed_root: Option<ManagedRootCredential>,
    ) -> Self {
        Self {
            store,
            managed_root,
        }
    }
}

#[async_trait]
impl CredentialResolver for CatalogResolver {
    async fn resolve(
        &self,
        location: &StorageLocation,
    ) -> olai_credential_vending::Result<RegisteredCredential> {
        let not_found = || olai_credential_vending::Error::NotFound {
            location: location.to_string(),
        };
        match self.store.as_ref().resolve(location).await {
            Err(olai_credential_vending::Error::NotFound { .. }) => {}
            other => return other.inspect(record_kind),
        }
        let Some(root) = self
            .managed_root
            .as_ref()
            .filter(|root| root.scope.covers(location))
        else {
            return Err(not_found());
        };
        let credential = self
            .store
            .credential_with_secrets(&root.credential_name)
            .await
            .map_err(|e| match e {
                unitycatalog_common::Error::NotFound => not_found(),
                other => olai_credential_vending::Error::mint(other),
            })?;
        RegisteredCredential::try_from(&credential)
            .map_err(|e| olai_credential_vending::Error::Unsupported {
                location: location.to_string(),
                reason: e.to_string(),
            })
            .inspect(record_kind)
    }
}

tokio::task_local! {
    /// The kind of the registered credential the current [`mint`] resolved to,
    /// for its audit event. The vendor resolves on every vend, inside the mint's
    /// task, so no extra store read is needed.
    static RESOLVED_KIND: Cell<Option<&'static str>>;
}

fn record_kind(credential: &RegisteredCredential) {
    // Outside a `mint` (e.g. the storage proxy's minter) there is nothing to record.
    let _ = RESOLVED_KIND.try_with(|kind| kind.set(Some(credential.kind())));
}

/// The kind recorded by [`RegisteredCredential::kind`] for Neon storage.
const NEON_KIND: &str = "neon_storage";

/// Trusted vending for Neon object storage (`vending.neon` in the server config).
///
/// Carries the defaults a Neon credential's ARN is completed from on
/// registration, and, when enabled in a `neon-trusted-vending` build, the Neon
/// API key and minter options. [`NeonVending::default`] is disabled with no
/// defaults.
#[derive(Clone, Default)]
pub struct NeonVending {
    project_id: Option<String>,
    branch_id: Option<String>,
    #[cfg(feature = "neon-trusted-vending")]
    minter: Option<NeonMinterSetup>,
}

#[cfg(feature = "neon-trusted-vending")]
#[derive(Clone)]
struct NeonMinterSetup {
    api_key: String,
    options: olai_credential_vending::neon::NeonMinterOptions,
}

impl NeonVending {
    /// Disabled vending, completing registered ARNs from `project_id` / `branch_id`.
    pub fn disabled(project_id: Option<String>, branch_id: Option<String>) -> Self {
        Self {
            project_id: project_id.filter(|id| !id.is_empty()),
            branch_id: branch_id.filter(|id| !id.is_empty()),
            ..Self::default()
        }
    }

    /// This configuration with vending enabled: Neon credentials mint with
    /// `api_key` and `options`.
    ///
    /// **Trusted:** a minted key reaches every bucket on its branch. Enable only
    /// behind real authentication and authorization.
    #[cfg(feature = "neon-trusted-vending")]
    pub fn enabled(
        mut self,
        api_key: impl Into<String>,
        options: olai_credential_vending::neon::NeonMinterOptions,
    ) -> Self {
        self.minter = Some(NeonMinterSetup {
            api_key: api_key.into(),
            options,
        });
        self
    }

    /// Whether Neon credentials mint.
    pub fn is_enabled(&self) -> bool {
        #[cfg(feature = "neon-trusted-vending")]
        return self.minter.is_some();
        #[cfg(not(feature = "neon-trusted-vending"))]
        false
    }

    /// The project a registered Neon ARN without one is completed with.
    pub fn project_id(&self) -> Option<&str> {
        self.project_id.as_deref()
    }

    /// The branch a registered Neon ARN without one is completed with.
    pub fn branch_id(&self) -> Option<&str> {
        self.branch_id.as_deref()
    }

    /// The minter for `branch`, or why there is none.
    fn minter(
        &self,
        branch: &NeonStorage,
    ) -> olai_credential_vending::Result<Arc<dyn CredentialMinter>> {
        #[cfg(feature = "neon-trusted-vending")]
        if let Some(setup) = &self.minter {
            let minter = olai_credential_vending::NeonTrustedMinter::new(
                branch,
                setup.api_key.clone(),
                setup.options.clone(),
            )?;
            return Ok(Arc::new(minter));
        }
        let _ = branch;
        let reason = if cfg!(feature = "neon-trusted-vending") {
            "trusted Neon vending is disabled on this server (`vending.neon.enabled: false`)"
        } else {
            "this server is built without trusted Neon vending (the `neon-trusted-vending` feature)"
        };
        Err(olai_credential_vending::Error::NeedsHostConfig {
            kind: NEON_KIND,
            reason: reason.into(),
        })
    }
}

impl std::fmt::Debug for NeonVending {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never the API key.
        let mut f = f.debug_struct("NeonVending");
        f.field("enabled", &self.is_enabled())
            .field("project_id", &self.project_id)
            .field("branch_id", &self.branch_id);
        #[cfg(feature = "neon-trusted-vending")]
        if let Some(setup) = &self.minter {
            f.field("options", &setup.options);
        }
        f.finish_non_exhaustive()
    }
}

/// The vendor a handler mints through.
pub(crate) type CatalogVendor = Vendor<CatalogResolver>;

/// Build the vendor for a handler's stores and managed-root configuration.
/// Neon storage credentials mint as `neon` allows; every other kind through
/// trestle's [`minter_for`].
pub(crate) fn catalog_vendor(
    store: Arc<dyn StorageAccessStore>,
    managed_root: Option<ManagedRootCredential>,
    neon: &NeonVending,
) -> Arc<CatalogVendor> {
    let neon = neon.clone();
    let vendor =
        Vendor::new(CatalogResolver::new(store, managed_root)).with_minters(move |credential| {
            match credential {
                RegisteredCredential::NeonStorage(branch) => neon.minter(branch),
                other => minter_for(other),
            }
        });
    Arc::new(vendor)
}

/// A [`CredentialMinter`] over a shared vendor, for callers (the storage proxy's
/// refreshing store) that re-mint the same request later.
#[derive(Clone)]
pub(crate) struct VendorMinter(pub(crate) Arc<CatalogVendor>);

impl std::fmt::Debug for VendorMinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("VendorMinter").field(&self.0).finish()
    }
}

#[async_trait]
impl CredentialMinter for VendorMinter {
    async fn mint(
        &self,
        request: &MintRequest,
    ) -> olai_credential_vending::Result<VendedCredential> {
        self.0.vend(request).await
    }
}

/// The minter for `location`: the [`LocalMinter`] for local (`file://`) storage,
/// else the vendor.
pub(crate) fn minter_for_location(
    vendor: &Arc<CatalogVendor>,
    location: &StorageLocationUrl,
) -> Arc<dyn CredentialMinter> {
    if location.is_local() {
        Arc::new(LocalMinter)
    } else {
        Arc::new(VendorMinter(vendor.clone()))
    }
}

/// The audit log target: one event per credential vend handed to a client.
pub const AUDIT_TARGET: &str = "unitycatalog::audit";

/// The API a vend was asked for through, for its audit event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VendSurface {
    /// The UC REST `temporary-*-credentials` endpoints.
    UcRest,
    /// The UC Delta API `/credentials` endpoints (and staging-table creation).
    DeltaApi,
}

impl VendSurface {
    fn as_str(self) -> &'static str {
        match self {
            Self::UcRest => "uc_rest",
            Self::DeltaApi => "delta_api",
        }
    }
}

/// Who asked for a vend, and for what: the subject of its audit event.
pub(crate) struct VendAudit<'a> {
    pub(crate) surface: VendSurface,
    /// The securable the caller was authorized for: a full name, or the path of a
    /// path vend.
    pub(crate) securable: String,
    /// The request, for its principal and the policies that allowed it.
    pub(crate) context: &'a RequestContext,
}

impl<'a> VendAudit<'a> {
    pub(crate) fn new(
        surface: VendSurface,
        securable: impl Into<String>,
        context: &'a RequestContext,
    ) -> Self {
        Self {
            surface,
            securable: securable.into(),
            context,
        }
    }
}

/// A short, non-secret id of a vended credential for the audit log: the first
/// characters of the S3 access key id (a Neon key's token id, an STS key id).
/// SAS and bearer tokens are secrets through and through, so they get none.
fn short_credential_id(credentials: &StorageCredentials) -> Option<String> {
    match credentials {
        StorageCredentials::Aws { key_id, .. } => Some(key_id.chars().take(8).collect()),
        _ => None,
    }
}

/// Emits the audit event for one vend. Never formats a secret: only the
/// credential's kind, short id and expiry.
fn audit_vend(
    audit: &VendAudit<'_>,
    location: &StorageLocationUrl,
    access: AccessLevel,
    kind: Option<&'static str>,
    result: &Result<VendedCredential>,
) {
    let principal = match audit.context.recipient() {
        Principal::User(name) => name.as_str(),
        Principal::Anonymous => "anonymous",
    };
    let access = match access {
        AccessLevel::Read => "read",
        AccessLevel::ReadWrite => "read_write",
    };
    let policies = audit.context.decision_trail().policies();
    let kind = kind.unwrap_or(if location.is_local() {
        "local"
    } else {
        "unresolved"
    });
    match result {
        Ok(vended) => {
            let credential_id = short_credential_id(&vended.credentials);
            let expires_at = vended.expires_at.map(|t| t.to_rfc3339());
            tracing::info!(
                target: AUDIT_TARGET,
                event = "credential_vend",
                outcome = "ok",
                surface = audit.surface.as_str(),
                principal,
                securable = %audit.securable,
                location = %location.raw(),
                access,
                credential_kind = kind,
                credential_id = credential_id.as_deref().unwrap_or(""),
                expires_at = expires_at.as_deref().unwrap_or(""),
                ?policies,
                "credential vended"
            );
        }
        Err(err) => {
            tracing::info!(
                target: AUDIT_TARGET,
                event = "credential_vend",
                outcome = "error",
                surface = audit.surface.as_str(),
                principal,
                securable = %audit.securable,
                location = %location.raw(),
                access,
                credential_kind = kind,
                error_code = err.error_code(),
                ?policies,
                "credential vend failed"
            );
        }
    }
}

/// Mints a credential for `location` at `access`, emitting one audit event (see
/// [`AUDIT_TARGET`]) for `audit`, whatever the outcome.
pub(crate) async fn mint(
    vendor: &Arc<CatalogVendor>,
    location: &StorageLocationUrl,
    access: AccessLevel,
    audit: &VendAudit<'_>,
) -> Result<VendedCredential> {
    let request = MintRequest::new(location.scope(), access);
    let minter = minter_for_location(vendor, location);
    let (result, kind) = RESOLVED_KIND
        .scope(Cell::new(None), async {
            let result = minter.mint(&request).await;
            (result, RESOLVED_KIND.with(Cell::get))
        })
        .await;
    // Normalize like the store lookups do, so a location no credential governs is
    // a plain `NotFound`.
    let result = result.map_err(|e| crate::error::from_store(unitycatalog_common::Error::from(e)));
    audit_vend(audit, location, access, kind, &result);
    result
}

/// The UC REST response for `vended`, naming `location` as the client spelled it
/// (emulator and endpoint URLs do not survive the canonical form).
pub(crate) fn to_temporary_credential(
    vended: &VendedCredential,
    location: &StorageLocationUrl,
) -> TemporaryCredential {
    TemporaryCredential {
        url: location.raw().to_string(),
        ..TemporaryCredential::from(vended)
    }
}
