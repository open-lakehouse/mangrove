//! The mangrove [`StorageProxyBackend`] adapter (the **local arm**).
//!
//! Implements [`unitycatalog_storage_proxy::StorageProxyBackend`] for
//! [`ServerHandler<RequestContext>`], expressing the narrow proxy port over the
//! server's existing surfaces: name → resource resolution
//! ([`TableHandler::get_table`] / [`VolumeHandler::get_volume`]),
//! [`Policy::authorize_checked`], the credential-vending pipeline
//! ([`find_external_location_for_url`] → `get_credential_internal` → a minter), and
//! object-store construction from the minted credential ([`build_store`]).
//!
//! All the *proxy* semantics (streaming, `Range`/`If-Match`, the confused-deputy
//! prefix confinement) live in [`unitycatalog_storage_proxy`]; this adapter only
//! authorizes + vends + hands back a scoped store. The store re-mints through the
//! same cached minter should a long request outlive its credential.

use std::sync::Arc;

use async_trait::async_trait;
use object_store::DynObjectStore;
use object_store::prefix::PrefixStore;
use olai_credential_vending::{CredentialMinter, MintRequest};
use olai_storage_credentials::{AccessLevel, StorageLocation, VendedCredential};
use olai_vended_store::{CredentialSource, StoreOptions, build_store};

use unitycatalog_common::models::credentials::v1::GetCredentialRequest;
use unitycatalog_common::models::tables::v1::{GetTableRequest, Table};
use unitycatalog_common::models::volumes::v1::{GetVolumeRequest, Volume};
use unitycatalog_storage_proxy::backend::reject_key_traversal;
use unitycatalog_storage_proxy::{
    ProxyCapabilities, ProxyError, ProxyReq, ProxyResult, Securable, StorageProxyBackend,
};

use crate::Error;
use crate::api::RequestContext;
use crate::api::credentials::CredentialHandlerExt;
use crate::api::tables::TableHandler;
use crate::api::volumes::VolumeHandler;
use crate::policy::{Permission, Policy};
use crate::services::ServerHandler;
use crate::services::location::StorageLocationUrl;
use crate::services::object_store::find_external_location_for_url;
use crate::services::vending::minter_for_location;

/// Map a server [`Error`] onto a [`ProxyError`], preserving status semantics
/// (not-found → 404, not-allowed → 403, …).
fn to_proxy_err(e: Error) -> ProxyError {
    match e {
        Error::NotFound => ProxyError::NotFound("resource not found".into()),
        Error::NotAllowed => ProxyError::PermissionDenied("not authorized".into()),
        other => ProxyError::Internal(other.to_string()),
    }
}

/// Map a [`StorageLocationUrl::parse`] failure (a malformed URL) onto a [`ProxyError`].
fn to_proxy_err_location(e: Error) -> ProxyError {
    match e {
        Error::InvalidArgument(m) => ProxyError::InvalidArgument(m),
        other => to_proxy_err(other),
    }
}

/// The permission a verb requires: reads need [`Permission::Read`], writes
/// [`Permission::Write`].
fn required_permission(write: bool) -> Permission {
    if write {
        Permission::Write
    } else {
        Permission::Read
    }
}

/// The access level for a verb.
fn access_level(write: bool) -> AccessLevel {
    if write {
        AccessLevel::ReadWrite
    } else {
        AccessLevel::Read
    }
}

/// Re-mints the credential a proxied store was built from.
#[derive(Debug)]
struct MintSource {
    minter: Arc<dyn CredentialMinter>,
    request: MintRequest,
}

#[async_trait]
impl CredentialSource for MintSource {
    async fn fetch(&self) -> olai_vended_store::Result<VendedCredential> {
        self.minter
            .mint(&self.request)
            .await
            .map_err(olai_vended_store::Error::vend)
    }
}

/// Resolve a securable to its storage location, having authorized the requested
/// access against the concrete resource. Returns the storage-location URL string.
///
/// Mirrors the resolution chain in
/// [`generate_temporary_table_credentials`](crate::api::temporary_credentials)
/// but keyed by fully-qualified **name** (what the proxy wire carries) rather than
/// a client-supplied UUID.
async fn authorize_and_locate(
    handler: &ServerHandler<RequestContext>,
    securable: &Securable,
    write: bool,
    cx: &RequestContext,
) -> ProxyResult<String> {
    let permission = required_permission(write);
    match securable {
        Securable::Table { full_name } => {
            // Name → Table. `get_table` runs its own read authorization; a write
            // then needs an explicit write check against the concrete table.
            let table: Table = TableHandler::get_table(
                handler,
                GetTableRequest {
                    full_name: full_name.clone(),
                    ..Default::default()
                },
                cx.clone(),
            )
            .await
            .map_err(to_proxy_err)?;
            if write {
                let id = table.table_id.as_deref().ok_or_else(|| {
                    ProxyError::Internal("table has no id to authorize write".into())
                })?;
                authorize_table_write(handler, id, cx).await?;
            }
            table
                .storage_location
                .filter(|s| !s.is_empty())
                .ok_or_else(|| ProxyError::NotFound("table has no storage location".into()))
        }
        Securable::Volume { full_name } => {
            let volume: Volume = VolumeHandler::get_volume(
                handler,
                GetVolumeRequest {
                    name: full_name.clone(),
                    ..Default::default()
                },
                cx.clone(),
            )
            .await
            .map_err(to_proxy_err)?;
            if write {
                authorize_volume_write(handler, &volume.volume_id, cx).await?;
            }
            Ok(volume.storage_location)
        }
        Securable::Path { url } => {
            // A raw path must resolve to a registered external location the caller
            // is authorized on; `find_external_location_for_url` 404s otherwise.
            let storage_url =
                StorageLocationUrl::parse(url.as_str()).map_err(to_proxy_err_location)?;
            let ext_loc = find_external_location_for_url(&storage_url, handler)
                .await
                .map_err(to_proxy_err)?;
            handler
                .authorize_checked(&(&ext_loc).into(), &permission, cx)
                .await
                .map_err(to_proxy_err)?;
            Ok(url.to_string())
        }
    }
}

/// Authorize a write against a concrete table UUID.
async fn authorize_table_write(
    handler: &ServerHandler<RequestContext>,
    table_id: &str,
    cx: &RequestContext,
) -> ProxyResult<()> {
    use unitycatalog_common::models::{ResourceIdent, ResourceRef};
    let uuid = uuid::Uuid::parse_str(table_id)
        .map_err(|_| ProxyError::Internal("table id is not a valid UUID".into()))?;
    let ident = ResourceIdent::Table(ResourceRef::Uuid(uuid));
    handler
        .authorize_checked(&ident, &Permission::Write, cx)
        .await
        .map_err(to_proxy_err)
}

/// Authorize a write against a concrete volume UUID.
async fn authorize_volume_write(
    handler: &ServerHandler<RequestContext>,
    volume_id: &str,
    cx: &RequestContext,
) -> ProxyResult<()> {
    use unitycatalog_common::models::{ResourceIdent, ResourceRef};
    let uuid = uuid::Uuid::parse_str(volume_id)
        .map_err(|_| ProxyError::Internal("volume id is not a valid UUID".into()))?;
    let ident = ResourceIdent::Volume(ResourceRef::Uuid(uuid));
    handler
        .authorize_checked(&ident, &Permission::Write, cx)
        .await
        .map_err(to_proxy_err)
}

/// Vend a credential for a resolved storage location and build a store from it.
async fn open_store(
    handler: &ServerHandler<RequestContext>,
    location: &str,
    write: bool,
) -> ProxyResult<Arc<DynObjectStore>> {
    let storage_url = StorageLocationUrl::parse(location).map_err(to_proxy_err_location)?;
    let credential = if storage_url.is_local() {
        None
    } else {
        let ext_loc = find_external_location_for_url(&storage_url, handler)
            .await
            .map_err(to_proxy_err)?;
        let credential = handler
            .get_credential_internal(GetCredentialRequest {
                name: ext_loc.credential_name.clone(),
                ..Default::default()
            })
            .await
            .map_err(to_proxy_err)?;
        Some(credential)
    };
    let minter = minter_for_location(credential.as_ref(), &storage_url).map_err(to_proxy_err)?;
    let source = Arc::new(MintSource {
        minter,
        request: MintRequest::new(storage_url.scope(), access_level(write)),
    });
    let mut seed = source
        .fetch()
        .await
        .map_err(|e| ProxyError::Internal(format!("vend: {e}")))?;
    // What the location URL implies (an Azurite endpoint, a regional S3 host) fills
    // what the minter left out.
    let (_, hints) = StorageLocation::parse_with_hints(storage_url.raw())
        .map_err(|e| ProxyError::InvalidArgument(e.to_string()))?;
    seed.hints.fill_from(&hints);
    // The server has no ambient region to forward here; region discovery and the
    // `AWS_REGION` env fallback still apply.
    let store = build_store(&seed, source, &StoreOptions::new())
        .await
        .map_err(|e| ProxyError::Internal(format!("build store: {e}")))?;
    // Root the store at the location itself: the credential may cover more (the
    // emulator grants whole containers).
    Ok(Arc::new(PrefixStore::new(
        store.root(),
        storage_url.path().clone(),
    )))
}

#[async_trait]
impl StorageProxyBackend<RequestContext> for ServerHandler<RequestContext> {
    fn capabilities(&self) -> ProxyCapabilities {
        ProxyCapabilities { enabled: true }
    }

    async fn authorize(&self, req: &ProxyReq, cx: &RequestContext) -> ProxyResult<()> {
        // Layer 1: reject key traversal before anything touches storage.
        reject_key_traversal(&req.key)?;
        // Layers 2-3: authorize the concrete securable for the verb's access.
        // (The resolved location is recomputed in `open`; resolution is cheap and
        // keeps the two port methods independent.)
        authorize_and_locate(self, &req.securable, req.verb.is_write(), cx).await?;
        Ok(())
    }

    async fn open(&self, req: &ProxyReq, cx: &RequestContext) -> ProxyResult<Arc<DynObjectStore>> {
        let write = req.verb.is_write();
        let location = authorize_and_locate(self, &req.securable, write, cx).await?;
        open_store(self, &location, write).await
    }
}
