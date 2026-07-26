//! `object_store` integration for Unity Catalog.
//!
//! This crate adapts Unity Catalog's credential-vending APIs to the
//! [`object_store`](https://docs.rs/object_store) trait, so any framework
//! that accepts an `Arc<dyn ObjectStore>` (DataFusion, `delta_kernel`,
//! `parquet`, …) can read and write data governed by Unity Catalog
//! volumes, tables, or external locations with no extra glue.
//!
//! # Quickstart
//!
//! ```no_run
//! use unitycatalog_object_store::{Operation, UnityObjectStoreFactory};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let factory = UnityObjectStoreFactory::builder()
//!     .with_uri("https://my-workspace.cloud.databricks.com/api/2.1/unity-catalog/")
//!     .with_token(std::env::var("DATABRICKS_TOKEN").unwrap())
//!     .build()
//!     .await?;
//!
//! // Address a UC securable directly with a `uc://` URL …
//! let store = factory
//!     .for_url("uc:///Volumes/main/default/landing/raw/", Operation::Read)
//!     .await?;
//!
//! // … and use it like any other `object_store`.
//! let listing = futures::StreamExt::collect::<Vec<_>>(store.as_dyn().list(None)).await;
//! # Ok(()) }
//! ```
//!
//! # URL scheme
//!
//! See [`UCReference`] for the full grammar. In short:
//!
//! - `uc:///Volumes/<catalog>/<schema>/<volume>[/<path>]`
//! - `uc:///Tables/<catalog>/<schema>/<table>`
//! - `s3://`, `gs://`, `abfss://`, `r2://`, … — raw cloud URLs, vended via
//!   `temporary-path-credentials`.
//!
//! The kind segment is **case-insensitive** (so `/Volumes/`, `/volumes/`,
//! and `/VOLUMES/` all work); the capitalised form is canonical because it
//! mirrors the Databricks workspace POSIX path convention. The Databricks
//! `vol+dbfs:/Volumes/...` alias is also accepted.

use std::sync::Arc;

use object_store::azure::MicrosoftAzureBuilder;
use object_store::path::Path;
use object_store::prefix::PrefixStore;
use object_store::{ObjectStore, Result};
use unitycatalog_client::{TemporaryCredentialClient, UnityCatalogClient};

// Native-only object_store backends and I/O handle. On `wasm32` the AWS/GCP
// stores are unsupported (Azure-first), `LocalFileSystem` is absent from
// object_store, and there is no tokio runtime.
#[cfg(not(target_arch = "wasm32"))]
use object_store::aws::AmazonS3Builder;
#[cfg(not(target_arch = "wasm32"))]
use object_store::client::SpawnedReqwestConnector;
#[cfg(not(target_arch = "wasm32"))]
use object_store::gcp::GoogleCloudStorageBuilder;
#[cfg(not(target_arch = "wasm32"))]
use object_store::local::LocalFileSystem;
#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Handle;

/// HTTP transport for credential vending, selected per target — mirrors the
/// alias in `unitycatalog-client` (`crates/client/src/lib.rs`). Native builds
/// use `olai_http::CloudClient`; `wasm32` builds use the browser Fetch
/// `olai_http_wasm::WasmClient`. The per-service clients
/// (`TemporaryCredentialClient`, `UnityCatalogClient`) hold this same type.
#[cfg(not(target_arch = "wasm32"))]
use olai_http::CloudClient as Transport;
#[cfg(not(target_arch = "wasm32"))]
use olai_http::service::{HttpService, ReqwestService};
#[cfg(target_arch = "wasm32")]
use olai_http_wasm::WasmClient as Transport;
use unitycatalog_common::tables::v1::GetTableRequest;
use unitycatalog_common::temporary_credentials::v1::TemporaryCredential;
use unitycatalog_common::volumes::v1::GetVolumeRequest;
use url::Url;

use crate::credential::{SecurableRef, as_aws, as_azure, as_gcp, new_azure};
// AWS/GCP provider constructors + the access-point helper are native-only: on
// wasm the store is Azure-first and these are never constructed.
#[cfg(not(target_arch = "wasm32"))]
use crate::credential::{aws_access_point, new_aws, new_gcp};
pub use crate::error::Error;
pub use unitycatalog_common::UCReference;
// Re-export the reference / operation enums so consumers do not need a direct
// dependency on `unitycatalog-client` for the common case.
pub use unitycatalog_client::{
    PathOperation, TableOperation, TableReference, VolumeOperation, VolumeReference,
};

mod credential;
mod error;
mod proxy;

/// An [`HttpService`] decorator that injects a fixed request header on every
/// outbound request before delegating to an inner service.
///
/// Used by [`UnityObjectStoreFactory::with_forwarded_user`] to forward a trusted
/// reverse-proxy identity header (e.g. `x-forwarded-user`) onto the upstream
/// credential-vending + metadata calls, so the upstream Unity Catalog attributes
/// the vend to the end user rather than the proxy's own service principal. The
/// header is `insert`ed (replacing any existing value of the same name), not
/// appended.
///
/// Native-only: the browser Fetch transport (`wasm32`) forwards identity through
/// its own same-origin session, so this decorator is unused there.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
struct ForwardedHeaderService {
    inner: Arc<dyn HttpService>,
    name: reqwest::header::HeaderName,
    value: reqwest::header::HeaderValue,
}

#[cfg(not(target_arch = "wasm32"))]
impl HttpService for ForwardedHeaderService {
    fn call(
        &self,
        mut request: reqwest::Request,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = olai_http::Result<reqwest::Response>> + Send + '_>,
    > {
        request
            .headers_mut()
            .insert(self.name.clone(), self.value.clone());
        self.inner.call(request)
    }
}

/// Builder for [`UnityObjectStoreFactory`].
#[derive(Debug, Clone, Default)]
pub struct UnityObjectStoreFactoryBuilder {
    /// Base URL of the Unity Catalog REST API
    /// (e.g. `https://<workspace>.cloud.databricks.com/api/2.1/unity-catalog/`).
    uri: Option<String>,
    /// Bearer token used for authentication.
    token: Option<String>,
    /// Permit construction without a token. Useful for local development
    /// against an unauthenticated OSS server; do not use in production.
    allow_unauthenticated: bool,
    /// Optional AWS region hint. Required when the data lives in a region
    /// other than `us-east-1` and the server does not return region info
    /// alongside the vended credential.
    aws_region: Option<String>,
    /// Optional dedicated tokio runtime for HTTP I/O. When set, all
    /// object-store and credential-vending requests are spawned on this
    /// runtime instead of the ambient one. See [`with_io_runtime`].
    ///
    /// [`with_io_runtime`]: UnityObjectStoreFactoryBuilder::with_io_runtime
    #[cfg(not(target_arch = "wasm32"))]
    io_handle: Option<Handle>,
}

impl UnityObjectStoreFactoryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the URI of the Unity Catalog API
    /// (e.g. `https://<workspace>/api/2.1/unity-catalog/`).
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Set the [access token] used for bearer authentication.
    ///
    /// Accepts both `String` and `Option<String>` — pass `None` to clear
    /// a previously-set token (e.g. when reusing the builder).
    ///
    /// [access token]: https://docs.databricks.com/aws/en/dev-tools/auth/pat
    pub fn with_token(mut self, token: impl Into<Option<String>>) -> Self {
        self.token = token.into();
        self
    }

    /// Allow construction without any authentication credentials.
    ///
    /// Only intended for local development against an unauthenticated OSS
    /// Unity Catalog server — there should not be any unauthenticated UC
    /// servers in production deployments.
    pub fn with_allow_unauthenticated(mut self, allow_unauthenticated: bool) -> Self {
        self.allow_unauthenticated = allow_unauthenticated;
        self
    }

    /// Override the AWS region used for vended AWS credentials.
    ///
    /// When unset the factory falls back to (in order):
    /// 1. The `AWS_REGION` environment variable.
    /// 2. The `object_store` default region (`us-east-1`).
    ///
    /// This is a stop-gap until the server reliably returns region info
    /// alongside the credential.
    pub fn with_aws_region(mut self, aws_region: impl Into<Option<String>>) -> Self {
        self.aws_region = aws_region.into();
        self
    }

    /// Route all HTTP I/O onto a dedicated tokio runtime.
    ///
    /// In production DataFusion deployments it is common to segregate network
    /// I/O onto a separate runtime so that CPU-bound query work on the main
    /// runtime cannot starve object-store requests (and vice versa). When a
    /// handle is supplied here:
    ///
    /// - every cloud object store ([`AmazonS3Builder`], [`MicrosoftAzureBuilder`],
    ///   [`GoogleCloudStorageBuilder`]) is built with a
    ///   [`SpawnedReqwestConnector`] that spawns its requests on this runtime; and
    /// - the credential-vending [`CloudClient`] is configured with the same
    ///   runtime via [`CloudClient::with_runtime`].
    ///
    /// When unset (the default), I/O runs on the ambient runtime — current
    /// behaviour, fully backwards compatible.
    ///
    /// Pass `None` to clear a previously set handle (e.g. when reusing the
    /// builder).
    ///
    /// Not available on `wasm32`: the browser has a single-threaded executor
    /// and no tokio runtime to route I/O onto.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_io_runtime(mut self, handle: impl Into<Option<Handle>>) -> Self {
        self.io_handle = handle.into();
        self
    }

    pub async fn build(self) -> Result<UnityObjectStoreFactory> {
        let url = if let Some(uri) = self.uri.as_ref() {
            url::Url::parse(uri).map_err(Error::from)?
        } else {
            return Err(Error::invalid_config("missing `uri` for Unity Catalog endpoint").into());
        };

        let transport = self.build_transport()?;

        // On wasm, discover the server's storage-access posture once, up front,
        // so every `for_*` call routes consistently. A discovery failure (older
        // server, no proxy) resolves to `Direct` — the historical behavior.
        #[cfg(target_arch = "wasm32")]
        let storage_access = proxy::discover_storage_access(&url, self.token.as_deref()).await;

        let creds = TemporaryCredentialClient::new_with_url(transport.clone(), url.clone());
        let uc = UnityCatalogClient::new(transport, url.clone());
        Ok(UnityObjectStoreFactory {
            creds,
            uc,
            aws_region: self.aws_region,
            #[cfg(not(target_arch = "wasm32"))]
            io_handle: self.io_handle,
            #[cfg(not(target_arch = "wasm32"))]
            base_url: url,
            #[cfg(not(target_arch = "wasm32"))]
            allow_unauthenticated: self.allow_unauthenticated,
            #[cfg(target_arch = "wasm32")]
            storage_access,
            #[cfg(target_arch = "wasm32")]
            token: self.token,
        })
    }

    /// Build the credential-vending transport for the current target.
    ///
    /// Native: the `olai-http` cloud client, optionally routed onto the
    /// dedicated I/O runtime. `wasm32`: the browser Fetch client, carrying a
    /// bearer token via `with_auth` when one was supplied (otherwise it relies
    /// on the ambient browser session — cookies / forwarded auth).
    #[cfg(not(target_arch = "wasm32"))]
    fn build_transport(&self) -> Result<Transport> {
        let cloud_client = if let Some(token) = &self.token {
            Transport::new_with_token(token)
        } else if self.allow_unauthenticated {
            Transport::new_unauthenticated()
        } else {
            return Err(Error::invalid_config(
                "no token and `allow_unauthenticated` not set: cannot build credential client",
            )
            .into());
        };

        // Route credential-vending HTTP onto the dedicated I/O runtime when one
        // was supplied; otherwise leave it on the ambient runtime.
        Ok(match &self.io_handle {
            Some(handle) => cloud_client.with_runtime(handle.clone()),
            None => cloud_client,
        })
    }

    #[cfg(target_arch = "wasm32")]
    fn build_transport(&self) -> Result<Transport> {
        use olai_http_wasm::CredentialsMode;
        use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

        // No `allow_unauthenticated` gate on wasm: a bare browser session is a
        // valid unauthenticated mode (cookies / same-origin forwarded auth).
        let client = Transport::new();
        Ok(match &self.token {
            Some(token) => {
                let token = token.clone();
                // Bearer auth: attach the header per request and stop the
                // browser from shadowing it with a stale cookie.
                client
                    .with_credentials(CredentialsMode::Omit)
                    .with_auth(move || {
                        let mut headers = HeaderMap::new();
                        if let Ok(value) = HeaderValue::from_str(&format!("Bearer {token}")) {
                            headers.insert(AUTHORIZATION, value);
                        }
                        headers
                    })
            }
            None => client,
        })
    }
}

/// A configured Unity Catalog `ObjectStore` ready for use.
///
/// The default [`Self::as_dyn`] returns a store that is automatically
/// prefixed to the credential-scoped sub-path (e.g. just the volume's
/// storage root); paths passed to `list`/`get`/`put` are interpreted
/// relative to that prefix. The unprefixed [`Self::root`] is an escape
/// hatch for callers that need to work at the bucket level.
#[derive(Clone)]
pub struct UCStore {
    /// Bucket-rooted store (credentials may be scoped to a sub-path).
    root: Arc<dyn ObjectStore>,
    /// The full cloud URL of the credential-scoped root.
    url: Url,
    /// Path within `root` the credential is scoped to.
    path: Path,
}

impl UCStore {
    /// Returns the credential-scoped store (prefixed at [`Self::prefix`]).
    ///
    /// This is the common case: callers list / read / write paths inside
    /// the volume or table the credential was vended for.
    pub fn as_dyn(&self) -> Arc<dyn ObjectStore> {
        if self.path.as_ref().is_empty() {
            self.root.clone()
        } else {
            Arc::new(PrefixStore::new(self.root.clone(), self.path.clone()))
        }
    }

    /// Returns the bucket-rooted store.
    ///
    /// The vended credential may not authorise access to siblings of
    /// [`Self::prefix`]; callers using `root()` are responsible for not
    /// accessing paths outside the scoped region.
    pub fn root(&self) -> Arc<dyn ObjectStore> {
        self.root.clone()
    }

    /// The full cloud URL of the credential-scoped root
    /// (e.g. `s3://bucket/prefix/inside/volume/`).
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// The prefix inside [`Self::root`] the credential is scoped to.
    pub fn prefix(&self) -> &Path {
        &self.path
    }
}

/// Factory that mints `object_store` instances backed by Unity Catalog
/// credential vending.
#[derive(Clone)]
pub struct UnityObjectStoreFactory {
    creds: TemporaryCredentialClient,
    uc: UnityCatalogClient,
    // Read only by the native AWS store branch; the wasm build is Azure-first and
    // never consults it, but keeping the field (and `with_aws_region`) uniform
    // across targets avoids splitting the builder API.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    aws_region: Option<String>,
    /// Dedicated runtime for object-store HTTP I/O, if configured via
    /// [`UnityObjectStoreFactoryBuilder::with_io_runtime`]. Absent on `wasm32`.
    #[cfg(not(target_arch = "wasm32"))]
    io_handle: Option<Handle>,
    /// The UC REST base URL, retained so [`with_forwarded_user`] can rebuild the
    /// credential + metadata clients over a derived transport.
    ///
    /// [`with_forwarded_user`]: UnityObjectStoreFactory::with_forwarded_user
    #[cfg(not(target_arch = "wasm32"))]
    base_url: Url,
    /// Whether the factory was built without a token, retained for parity when
    /// [`with_forwarded_user`] derives a transport (the forwarded-identity path
    /// is unauthenticated by design; this field documents that the base factory
    /// also tolerated no token).
    ///
    /// [`with_forwarded_user`]: UnityObjectStoreFactory::with_forwarded_user
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(dead_code)]
    allow_unauthenticated: bool,
    /// Storage-access posture discovered from the server's `/capabilities`
    /// endpoint at build time. `Proxy` routes every byte through the server's
    /// same-origin storage byte-proxy instead of vending + direct cloud IO.
    /// Only meaningful on `wasm32` (native always reads storage directly).
    #[cfg(target_arch = "wasm32")]
    storage_access: proxy::StorageAccess,
    /// Bearer token, retained on `wasm32` so the proxy store can relay it to the
    /// same-origin `/storage-proxy` endpoint (the endpoint may require auth).
    #[cfg(target_arch = "wasm32")]
    token: Option<String>,
}

impl UnityObjectStoreFactory {
    pub fn builder() -> UnityObjectStoreFactoryBuilder {
        UnityObjectStoreFactoryBuilder::default()
    }

    /// Derive a factory that forwards a trusted reverse-proxy identity header on
    /// every upstream Unity Catalog request (both the metadata lookups and the
    /// credential vend).
    ///
    /// Intended for a service — e.g. the standalone storage byte-proxy — that
    /// sits behind the same reverse proxy as the upstream UC and has already
    /// validated the caller's identity. Forwarding the header verbatim lets UC's
    /// own reverse-proxy authenticator attribute the vend to the real end user.
    ///
    /// Semantics:
    /// - `user == None` (anonymous request) → returns `self` cloned unchanged, so
    ///   the upstream calls use whatever auth the base factory was built with
    ///   (its static token, or unauthenticated). Zero added cost.
    /// - `user == Some(name)` → returns a factory whose upstream transport is
    ///   **unauthenticated** and injects `header: name` on every request. The
    ///   base factory's static token is intentionally dropped for these calls:
    ///   the forwarded identity is the auth, not the proxy's own principal.
    ///
    /// `header` is the outgoing header name (typically the same
    /// `x-forwarded-user` the proxy read the identity from). Returns an error if
    /// `header` or `name` is not a valid HTTP header name / value.
    ///
    /// Native-only: on `wasm32` identity is forwarded by the browser session, so
    /// this method is absent.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_forwarded_user(&self, header: &str, user: Option<&str>) -> Result<Self> {
        use reqwest::header::{HeaderName, HeaderValue};

        let Some(name) = user else {
            // Anonymous: keep the base factory (and its auth) untouched.
            return Ok(self.clone());
        };

        let header_name = HeaderName::from_bytes(header.as_bytes()).map_err(|e| {
            Error::invalid_config(format!(
                "invalid forwarded-user header name `{header}`: {e}"
            ))
        })?;
        let header_value = HeaderValue::from_str(name).map_err(|e| {
            Error::invalid_config(format!("invalid forwarded-user header value: {e}"))
        })?;

        // A fresh unauthenticated transport (no bearer signer): the forwarded
        // header is the auth for these calls. Route I/O onto the dedicated
        // runtime when one is configured, mirroring `build_transport`.
        let reqwest_client = reqwest::Client::new();
        let base_service: Arc<dyn HttpService> =
            Arc::new(ReqwestService::new(reqwest_client.clone()));
        let transport =
            Transport::new_unauthenticated().with_http_service(Arc::new(ForwardedHeaderService {
                inner: base_service,
                name: header_name,
                value: header_value,
            }));
        let transport = match &self.io_handle {
            Some(handle) => transport.with_runtime(handle.clone()),
            None => transport,
        };

        // Rebuilding both clients is cheap — each just holds a `Transport` clone
        // (Arc bumps) plus a `Url`.
        let creds =
            TemporaryCredentialClient::new_with_url(transport.clone(), self.base_url.clone());
        let uc = UnityCatalogClient::new(transport, self.base_url.clone());
        Ok(UnityObjectStoreFactory {
            creds,
            uc,
            aws_region: self.aws_region.clone(),
            io_handle: self.io_handle.clone(),
            base_url: self.base_url.clone(),
            allow_unauthenticated: self.allow_unauthenticated,
        })
    }

    /// Borrow the underlying [`UnityCatalogClient`] for catalog metadata
    /// operations (listing volumes, resolving table names, …).
    pub fn unity_client(&self) -> &UnityCatalogClient {
        &self.uc
    }

    /// Borrow the underlying credential-vending client. Most users want
    /// [`for_url`](Self::for_url) / [`for_volume`](Self::for_volume) /
    /// [`for_table`](Self::for_table) / [`for_path`](Self::for_path) instead.
    pub fn credentials_client(&self) -> &TemporaryCredentialClient {
        &self.creds
    }

    /// Build an [`UCStore`] for any supported URL.
    ///
    /// See [`UCReference`] for the supported URL grammar. Raw cloud URLs
    /// (`s3://`, `gs://`, `abfss://`, …) are routed to
    /// [`for_path`](Self::for_path).
    pub async fn for_url(&self, url: &str, op: Operation) -> Result<UCStore> {
        let reference = UCReference::parse(url)
            .map_err(crate::error::Error::from)
            .map_err(object_store::Error::from)?;
        match reference {
            UCReference::Volume {
                catalog,
                schema,
                volume,
                path,
            } => {
                let name = format!("{catalog}.{schema}.{volume}");
                let store = self.for_volume(name, op.into_volume()).await?;
                if path.is_empty() {
                    Ok(store)
                } else {
                    Ok(extend_prefix(store, &path))
                }
            }
            UCReference::Table {
                catalog,
                schema,
                table,
            } => {
                let name = format!("{catalog}.{schema}.{table}");
                self.for_table(name, op.into_table()).await
            }
            UCReference::Path(url) => self.for_path(&url, op.into_path()).await,
        }
    }

    /// Vend credentials for a table and return a prefixed store rooted at
    /// the table's storage location.
    ///
    /// The `table` argument accepts a `Uuid`, a [`String`] / `&str`
    /// containing a three-level `<catalog>.<schema>.<table>` name, or any
    /// [`TableReference`].
    pub async fn for_table(
        &self,
        table: impl Into<TableReference>,
        operation: TableOperation,
    ) -> Result<UCStore> {
        let table = table.into();
        // For name references, resolve the storage location up front (a lookup
        // name-based vending makes anyway). Two branches consume it before
        // vending:
        //   - a `file://` location has no cloud credential to vend → local store;
        //   - on wasm under a proxy posture → route through the byte-proxy.
        // A UUID-only reference skips both (a local-fs or proxied table by UUID
        // is an unsupported edge case — use the three-level name).
        if let TableReference::Name(name) = &table
            && let Some(location) = self.table_storage_location(name).await?
            && let Ok(url) = Url::parse(&location)
        {
            if url.scheme() == "file" {
                let path_op = match operation {
                    TableOperation::Read => PathOperation::Read,
                    TableOperation::ReadWrite => PathOperation::ReadWrite,
                };
                return local_store(&url, path_op);
            }
            #[cfg(target_arch = "wasm32")]
            if matches!(self.storage_access, proxy::StorageAccess::Proxy { .. }) {
                let seg = format!("table:{name}");
                return self.proxy_store(&seg, &url);
            }
        }
        let (credential, table_id) = self
            .creds
            .temporary_table_credential(table, operation)
            .await
            .map_err(Error::from)?;
        let securable = SecurableRef::Table(table_id, operation);
        self.build_store(credential, securable).await
    }

    /// Look up a table's `storage_location` by its three-level name.
    ///
    /// Returns `None` when the table has no storage location set. Used by
    /// [`for_table`](Self::for_table) to detect `file://`-backed tables before
    /// vending.
    async fn table_storage_location(&self, full_name: &str) -> Result<Option<String>> {
        let table = self
            .uc
            .tables_client()
            .get_table(&GetTableRequest {
                full_name: full_name.to_string(),
                include_browse: Some(false),
                include_delta_metadata: Some(false),
                include_manifest_capabilities: Some(false),
                ..Default::default()
            })
            .await
            .map_err(Error::from)?;
        Ok(table.storage_location.filter(|s| !s.is_empty()))
    }

    /// Vend credentials for a volume and return a prefixed store rooted at
    /// the volume's storage location.
    ///
    /// A volume whose storage location is a `file://` path is served by a local
    /// [`LocalFileSystem`] store and never hits the credential-vending API —
    /// local storage has no cloud credential to vend.
    pub async fn for_volume(
        &self,
        volume: impl Into<VolumeReference>,
        operation: VolumeOperation,
    ) -> Result<UCStore> {
        let volume = volume.into();
        // As with `for_table`: a volume backed by local filesystem storage has
        // no cloud credential to vend. Resolve its storage location up front
        // (the same `GetVolume` lookup name-based vending makes) and, when it is
        // `file://`, build a local store directly — skipping vending.
        //
        // Only possible for name references; a caller holding only the volume
        // UUID still vends (a local-fs volume addressed by UUID is unsupported —
        // use the three-level name).
        if let VolumeReference::Name(name) = &volume
            && let Some(location) = self.volume_storage_location(name).await?
            && let Ok(url) = Url::parse(&location)
        {
            if url.scheme() == "file" {
                let path_op = match operation {
                    VolumeOperation::Read => PathOperation::Read,
                    VolumeOperation::ReadWrite => PathOperation::ReadWrite,
                };
                return local_store(&url, path_op);
            }
            // Proxy posture (wasm): route volume bytes (read + write) through
            // the server's storage byte-proxy.
            #[cfg(target_arch = "wasm32")]
            if matches!(self.storage_access, proxy::StorageAccess::Proxy { .. }) {
                let seg = format!("vol:{name}");
                return self.proxy_store(&seg, &url);
            }
        }
        let (credential, volume_id) = self
            .creds
            .temporary_volume_credential(volume, operation)
            .await
            .map_err(Error::from)?;
        let securable = SecurableRef::Volume(volume_id, operation);
        self.build_store(credential, securable).await
    }

    /// Look up a volume's `storage_location` by its three-level name.
    ///
    /// Returns `None` when the volume has no storage location set. Used by
    /// [`for_volume`](Self::for_volume) to detect `file://`-backed volumes
    /// before vending.
    async fn volume_storage_location(&self, name: &str) -> Result<Option<String>> {
        let volume = self
            .uc
            .volumes_client()
            .get_volume(&GetVolumeRequest {
                name: name.to_string(),
                include_browse: Some(false),
                ..Default::default()
            })
            .await
            .map_err(Error::from)?;
        Ok(Some(volume.storage_location).filter(|s| !s.is_empty()))
    }

    /// Vend credentials for a raw cloud URL (`s3://`, `gs://`, `abfss://`,
    /// …). Uses `temporary-path-credentials` under the hood.
    ///
    /// `file://` URLs are served by a local [`LocalFileSystem`] store and
    /// never hit the credential-vending API — local storage has no cloud
    /// credential to vend.
    pub async fn for_path(&self, path: &Url, operation: PathOperation) -> Result<UCStore> {
        if path.scheme() == "file" {
            return local_store(path, operation);
        }
        // Proxy posture (wasm): route bytes for the raw cloud URL through the
        // server's storage byte-proxy (`path:<pct-encoded url>` securable). The
        // URL itself is the securable root, so the routing prefix is its path.
        #[cfg(target_arch = "wasm32")]
        if matches!(self.storage_access, proxy::StorageAccess::Proxy { .. }) {
            let seg = proxy::path_securable(path);
            return self.proxy_store(&seg, path);
        }
        let (credential, _resolved) = self
            .creds
            .temporary_path_credential(path.clone(), operation, false)
            .await
            .map_err(Error::from)?;
        let securable = SecurableRef::Path(path.clone(), operation, Some(false));
        self.build_store(credential, securable).await
    }

    /// Vend credentials for a raw cloud URL with `dry_run` set to true.
    /// The server validates that credentials *could* be issued but the
    /// returned token is not usable for IO; useful for permission probes.
    ///
    /// For `file://` URLs there is nothing to probe — a local store is
    /// returned directly, identical to [`for_path`](Self::for_path).
    pub async fn dry_run_path(&self, path: &Url, operation: PathOperation) -> Result<UCStore> {
        if path.scheme() == "file" {
            return local_store(path, operation);
        }
        let (credential, _resolved) = self
            .creds
            .temporary_path_credential(path.clone(), operation, true)
            .await
            .map_err(Error::from)?;
        let securable = SecurableRef::Path(path.clone(), operation, Some(true));
        self.build_store(credential, securable).await
    }

    /// Build a proxy-backed [`UCStore`] for `securable_seg` (a typed
    /// `{securable}` segment) whose data lives at `location` (the real cloud
    /// URL of the securable root).
    ///
    /// Used on wasm when the server announces `storageAccess: "proxy"`: no
    /// credential is vended here (the server vends internally). `url` stays the
    /// real cloud URL so the read path's routing/log-discovery math is
    /// unchanged; `root` is a proxy `HttpStore` wrapped to strip the bucket
    /// prefix so forwarded bucket-rooted keys become securable-relative.
    #[cfg(target_arch = "wasm32")]
    fn proxy_store(&self, securable_seg: &str, location: &Url) -> Result<UCStore> {
        let proxy::StorageAccess::Proxy { base } = &self.storage_access else {
            // Only called from the proxy branches, which are gated on `Proxy`.
            return Err(Error::invalid_config(
                "proxy_store called without a proxy posture".to_string(),
            )
            .into());
        };
        // The bucket-relative prefix of the securable root — the same value
        // `build_store` records as `UCStore.path`, and exactly what the routing
        // store forwards ahead of the object key.
        let path = match parse_azurite(location) {
            Some(loc) => Path::from(loc.prefix),
            None => Path::from_url_path(location.path())?,
        };
        let root = proxy::build_proxy_store(base, securable_seg, path.clone(), self.token())?;
        Ok(UCStore {
            root,
            url: location.clone(),
            path,
        })
    }

    /// The bearer token this factory authenticates with, if any. On wasm the
    /// proxy store relays it to the same-origin proxy endpoint.
    #[cfg(target_arch = "wasm32")]
    fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    async fn build_store(
        &self,
        credential: TemporaryCredential,
        securable: SecurableRef,
    ) -> Result<UCStore> {
        let url = Url::parse(&credential.url).map_err(Error::from)?;
        // The emulator store is rooted at the container, so the prefix is the
        // blob path *within* the container — not the full URL path (which for
        // path-style Azurite also carries `/<account>/<container>`).
        let path = match parse_azurite(&url) {
            Some(loc) => Path::from(loc.prefix),
            None => Path::from_url_path(url.path())?,
        };
        let store = self.to_store(credential, securable).await?;
        Ok(UCStore {
            root: store,
            url,
            path,
        })
    }

    async fn to_store(
        &self,
        credential: TemporaryCredential,
        securable: SecurableRef,
    ) -> Result<Arc<dyn ObjectStore>> {
        if as_azure(&credential).is_ok() {
            let url = Url::parse(&credential.url).map_err(Error::from)?;

            // Azurite (local Blob emulator): the URL is path-style and
            // `with_url` does not understand it, and in emulator mode the
            // builder ignores a `with_credentials` provider — it only honours
            // an explicit access key / bearer / SAS. So set account + container
            // explicitly and pass the vended SAS via `SasKey`.
            if let Some(loc) = parse_azurite(&url) {
                let sas = azure_sas_token(&credential).ok_or_else(|| {
                    Error::invalid_config(
                        "Azurite store requires a SAS-token credential".to_string(),
                    )
                })?;
                #[allow(unused_mut)]
                let mut builder = MicrosoftAzureBuilder::new()
                    .with_use_emulator(true)
                    .with_account(loc.account)
                    .with_container_name(loc.container)
                    .with_config(object_store::azure::AzureConfigKey::SasKey, sas);
                // Native: route object-store HTTP onto the dedicated I/O runtime
                // when one was supplied. On wasm, use object_store's default
                // browser-Fetch connector (no runtime to route onto).
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(handle) = &self.io_handle {
                    builder =
                        builder.with_http_connector(SpawnedReqwestConnector::new(handle.clone()));
                }
                // Belt-and-braces: object_store's retry path panics on wasm
                // without a timer, so disable retries on the browser store.
                #[cfg(target_arch = "wasm32")]
                {
                    builder = builder.with_retry(object_store::RetryConfig {
                        max_retries: 0,
                        ..Default::default()
                    });
                }
                let store = builder.build()?;
                return Ok(Arc::new(store));
            }

            let provider = new_azure(self.creds.clone(), &credential, securable).await?;
            #[allow(unused_mut)]
            let mut builder = MicrosoftAzureBuilder::new()
                .with_url(url.to_string())
                .with_credentials(Arc::new(provider));
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(handle) = &self.io_handle {
                builder = builder.with_http_connector(SpawnedReqwestConnector::new(handle.clone()));
            }
            #[cfg(target_arch = "wasm32")]
            {
                builder = builder.with_retry(object_store::RetryConfig {
                    max_retries: 0,
                    ..Default::default()
                });
            }
            let store = builder.build()?;
            return Ok(Arc::new(store));
        }

        #[cfg(not(target_arch = "wasm32"))]
        if as_aws(&credential).is_ok() {
            let access_point = aws_access_point(&credential);
            let provider = new_aws(self.creds.clone(), &credential, securable).await?;
            let url = Url::parse(&credential.url).map_err(Error::from)?;
            let mut builder = AmazonS3Builder::new()
                .with_url(url.to_string())
                .with_credentials(Arc::new(provider));
            // Prefer an explicit override; otherwise honour `AWS_REGION`
            // before falling back to the object_store default.
            if let Some(region) = self
                .aws_region
                .clone()
                .or_else(|| std::env::var("AWS_REGION").ok())
            {
                builder = builder.with_region(region);
            }
            // Where the server returns an S3 access-point ARN, use it as the
            // bucket so SigV4 signatures match what STS authorised.
            if let Some(ap) = access_point {
                builder = builder.with_bucket_name(ap);
            }
            if let Some(handle) = &self.io_handle {
                builder = builder.with_http_connector(SpawnedReqwestConnector::new(handle.clone()));
            }
            let store = builder.build()?;
            return Ok(Arc::new(store));
        }

        #[cfg(not(target_arch = "wasm32"))]
        if as_gcp(&credential).is_ok() {
            let provider = new_gcp(self.creds.clone(), &credential, securable).await?;
            let url = Url::parse(&credential.url).map_err(Error::from)?;
            let mut builder = GoogleCloudStorageBuilder::new()
                .with_url(url.to_string())
                .with_credentials(Arc::new(provider));
            if let Some(handle) = &self.io_handle {
                builder = builder.with_http_connector(SpawnedReqwestConnector::new(handle.clone()));
            }
            let store = builder.build()?;
            return Ok(Arc::new(store));
        }

        // wasm is Azure-first: AWS/GCP stores are gated out above, so a
        // non-Azure credential here is unsupported rather than unmatched.
        #[cfg(target_arch = "wasm32")]
        if as_aws(&credential).is_ok() || as_gcp(&credential).is_ok() {
            return Err(Error::invalid_config(
                "AWS/GCP object stores are not supported on wasm (Azure-first)",
            )
            .into());
        }

        Err(
            Error::InvalidCredential("Failed to match credential with storage type".to_string())
                .into(),
        )
    }
}

/// Unified read/write operation used by [`UnityObjectStoreFactory::for_url`].
///
/// The factory translates this to the operation enum expected by each
/// vending endpoint:
///
/// | `Operation`  | Volume        | Table       | Path             |
/// |--------------|---------------|-------------|------------------|
/// | `Read`       | `READ_VOLUME` | `READ`      | `PATH_READ`      |
/// | `ReadWrite`  | `WRITE_VOLUME`| `READ_WRITE`| `PATH_READ_WRITE`|
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Read,
    ReadWrite,
}

impl Operation {
    fn into_volume(self) -> VolumeOperation {
        match self {
            Operation::Read => VolumeOperation::Read,
            Operation::ReadWrite => VolumeOperation::ReadWrite,
        }
    }

    fn into_table(self) -> TableOperation {
        match self {
            Operation::Read => TableOperation::Read,
            Operation::ReadWrite => TableOperation::ReadWrite,
        }
    }

    fn into_path(self) -> PathOperation {
        match self {
            Operation::Read => PathOperation::Read,
            Operation::ReadWrite => PathOperation::ReadWrite,
        }
    }
}

impl From<Operation> for TableOperation {
    fn from(op: Operation) -> Self {
        op.into_table()
    }
}

impl From<Operation> for VolumeOperation {
    fn from(op: Operation) -> Self {
        op.into_volume()
    }
}

impl From<Operation> for PathOperation {
    fn from(op: Operation) -> Self {
        op.into_path()
    }
}

/// Build a [`UCStore`] backed by the local filesystem for a `file://` URL,
/// bypassing credential vending entirely.
///
/// Mirrors the bucket-root + prefix model of vended cloud stores: `root` is an
/// unrooted [`LocalFileSystem`] (paths are absolute, relative to the filesystem
/// root) and `path` is the URL's directory. This keeps both consumption modes
/// correct:
///
/// - [`UCStore::as_dyn`] wraps `root` in a `PrefixStore` at the directory, so
///   callers address paths *relative* to the `file://` directory; and
/// - [`UCStore::root`] is the unrooted store, so the DataFusion routing store
///   — which forwards the full request path unchanged — resolves absolute
///   table paths.
///
/// For [`PathOperation::ReadWrite`] / [`PathOperation::CreateTable`] the target
/// directory is created if missing, so writes to a fresh local root succeed.
///
/// # Platform support
///
/// Local `file://` storage is **POSIX-only** for now. On Windows it returns an
/// error: `object_store::path::Path` cannot represent a Windows absolute path
/// (the drive-letter colon is percent-encoded), so an unrooted
/// [`LocalFileSystem`] addressed by full path does not resolve back to the real
/// on-disk location. Tracked for a follow-up; cloud schemes are unaffected.
///
/// Not available on `wasm32`: object_store has no `LocalFileSystem` there and a
/// browser has no local filesystem — a `file://` reference returns an error.
#[cfg(not(target_arch = "wasm32"))]
fn local_store(url: &Url, operation: PathOperation) -> Result<UCStore> {
    if cfg!(windows) {
        return Err(Error::invalid_config(format!(
            "local (file://) storage is not supported on Windows: {url}"
        ))
        .into());
    }

    let dir = url
        .to_file_path()
        .map_err(|_| Error::invalid_url(format!("not a valid local file path: {url}")))?;

    if matches!(
        operation,
        PathOperation::ReadWrite | PathOperation::CreateTable
    ) {
        std::fs::create_dir_all(&dir).map_err(|e| {
            Error::invalid_config(format!("failed to create local directory {dir:?}: {e}"))
        })?;
    }

    // Unrooted: object_store `Path`s are relative to the filesystem root, so a
    // full path like `tmp/data/part-0` resolves to `/tmp/data/part-0`. The
    // directory is carried as the `UCStore` prefix instead (see `build_store`).
    let store = LocalFileSystem::new();
    let path = Path::from_url_path(url.path())?;
    Ok(UCStore {
        root: Arc::new(store),
        url: url.clone(),
        path,
    })
}

/// A [`CredentialProvider`] that always hands back a fixed, already-materialized
/// credential. Unlike `object_store::StaticCredentialProvider` (which takes the
/// credential by value), this holds an `Arc<T>` directly — the shape
/// [`as_azure`]/[`as_aws`]/[`as_gcp`] already produce — so no `Clone` on the
/// (non-`Clone`) cloud credential types is required.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
struct StaticArcProvider<T>(Arc<T>);

#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
impl<T: std::fmt::Debug + Send + Sync> object_store::CredentialProvider for StaticArcProvider<T> {
    type Credential = T;
    async fn get_credential(&self) -> Result<Arc<T>> {
        Ok(self.0.clone())
    }
}

/// Build a [`UCStore`] from a credential that has **already been vended**
/// out-of-band, using a *static* credential provider (no Unity Catalog client,
/// no auto-refresh).
///
/// This is the escape hatch for an in-process caller — e.g. a server-side storage
/// byte-proxy — that authorizes and vends the credential itself and then only
/// needs it turned into an [`ObjectStore`]. Unlike [`UnityObjectStoreFactory`]'s
/// `for_*` entry points, this takes no factory and constructs no
/// [`UnityCatalogClient`]: the store is bound to the exact `credential` passed,
/// valid for that credential's lifetime. A single short-lived proxy request never
/// outlives the vended credential, so the lack of refresh is fine; a caller that
/// needs long-lived refresh should use the factory `for_*` methods instead.
///
/// The returned [`UCStore`] is scoped exactly as [`UnityObjectStoreFactory::for_path`]
/// would produce it; call [`UCStore::as_dyn`] for a store prefixed at the
/// credential-scoped root. `io_handle` (when `Some`) routes object-store HTTP onto
/// a dedicated I/O runtime; `aws_region` overrides the S3 region (else `AWS_REGION`
/// / the object_store default).
///
/// A `file://` URL yields a [`LocalFileSystem`] store and needs no credential.
#[cfg(not(target_arch = "wasm32"))]
pub fn store_from_vended_credential(
    credential: &TemporaryCredential,
    io_handle: Option<tokio::runtime::Handle>,
    aws_region: Option<String>,
) -> Result<UCStore> {
    let url = Url::parse(&credential.url).map_err(Error::from)?;

    // Local storage: no cloud credential to apply.
    if url.scheme() == "file" {
        let store = LocalFileSystem::new();
        let path = Path::from_url_path(url.path())?;
        return Ok(UCStore {
            root: Arc::new(store),
            url,
            path,
        });
    }

    // The credential-scoped prefix within the bucket/container root. For
    // path-style Azurite the store is rooted at the container, so the prefix is
    // the blob path only (not the full `/<account>/<container>/...` URL path).
    let path = match parse_azurite(&url) {
        Some(loc) => Path::from(loc.prefix),
        None => Path::from_url_path(url.path())?,
    };

    let root: Arc<dyn ObjectStore> = if as_azure(credential).is_ok() {
        // Azurite: the emulator ignores a credentials provider — set account +
        // container explicitly and pass the vended SAS via `SasKey` (identical to
        // the factory path).
        if let Some(loc) = parse_azurite(&url) {
            let sas = azure_sas_token(credential).ok_or_else(|| {
                Error::invalid_config("Azurite store requires a SAS-token credential".to_string())
            })?;
            let mut builder = MicrosoftAzureBuilder::new()
                .with_use_emulator(true)
                .with_account(loc.account)
                .with_container_name(loc.container)
                .with_config(object_store::azure::AzureConfigKey::SasKey, sas);
            if let Some(handle) = &io_handle {
                builder = builder.with_http_connector(SpawnedReqwestConnector::new(handle.clone()));
            }
            Arc::new(builder.build()?)
        } else {
            let token = as_azure(credential)?.token;
            let mut builder = MicrosoftAzureBuilder::new()
                .with_url(url.to_string())
                .with_credentials(Arc::new(StaticArcProvider(token)));
            if let Some(handle) = &io_handle {
                builder = builder.with_http_connector(SpawnedReqwestConnector::new(handle.clone()));
            }
            Arc::new(builder.build()?)
        }
    } else if as_aws(credential).is_ok() {
        let access_point = aws_access_point(credential);
        let token = as_aws(credential)?.token;
        let mut builder = AmazonS3Builder::new()
            .with_url(url.to_string())
            .with_credentials(Arc::new(StaticArcProvider(token)));
        if let Some(region) = aws_region.or_else(|| std::env::var("AWS_REGION").ok()) {
            builder = builder.with_region(region);
        }
        if let Some(ap) = access_point {
            builder = builder.with_bucket_name(ap);
        }
        if let Some(handle) = &io_handle {
            builder = builder.with_http_connector(SpawnedReqwestConnector::new(handle.clone()));
        }
        Arc::new(builder.build()?)
    } else if as_gcp(credential).is_ok() {
        let token = as_gcp(credential)?.token;
        let mut builder = GoogleCloudStorageBuilder::new()
            .with_url(url.to_string())
            .with_credentials(Arc::new(StaticArcProvider(token)));
        if let Some(handle) = &io_handle {
            builder = builder.with_http_connector(SpawnedReqwestConnector::new(handle.clone()));
        }
        Arc::new(builder.build()?)
    } else {
        return Err(Error::InvalidCredential(
            "Failed to match credential with storage type".to_string(),
        )
        .into());
    };

    Ok(UCStore { root, url, path })
}

/// wasm has no local filesystem; a `file://` reference is unsupported.
#[cfg(target_arch = "wasm32")]
fn local_store(url: &Url, _operation: PathOperation) -> Result<UCStore> {
    Err(Error::invalid_config(format!(
        "local (file://) storage is not supported on wasm: {url}"
    ))
    .into())
}

/// Parsed components of an Azurite (Azure Blob emulator) storage URL.
///
/// Azurite is path-style — the account, container, and blob prefix are all
/// segments of the path rather than the host (real Azure encodes the account in
/// the host: `https://<account>.blob.core.windows.net/<container>/<prefix>`).
struct AzuriteLocation {
    account: String,
    container: String,
    prefix: String,
}

/// Detect and parse an Azurite endpoint from a storage URL.
///
/// Recognised forms (mirroring the server's `is_azurite` / `StorageLocationUrl`):
/// - `http://127.0.0.1:10000/<account>/<container>/<prefix>` (default Azurite
///   blob endpoint on localhost / 127.0.0.1, port 10000), and
/// - `azurite://<container>/<prefix>` (custom scheme; the account is implicit,
///   defaulting to the well-known emulator account).
///
/// Returns `None` for any non-Azurite URL so the caller falls through to the
/// real-Azure path. `object_store`'s `MicrosoftAzureBuilder::with_url` does not
/// understand either form, so the Azurite branch must set account/container
/// explicitly rather than going through `with_url`.
fn parse_azurite(url: &Url) -> Option<AzuriteLocation> {
    const EMULATOR_ACCOUNT: &str = "devstoreaccount1";

    if url.scheme() == "azurite" {
        // azurite://<container>/<prefix>
        let container = url.host_str().filter(|s| !s.is_empty())?.to_owned();
        let prefix = url.path().trim_start_matches('/').to_owned();
        return Some(AzuriteLocation {
            account: EMULATOR_ACCOUNT.to_owned(),
            container,
            prefix,
        });
    }

    let is_localhost = matches!(url.host_str(), Some("localhost") | Some("127.0.0.1"));
    if url.scheme() == "http" && is_localhost && url.port() == Some(10000) {
        // http://127.0.0.1:10000/<account>/<container>/<prefix>
        let mut segments = url.path_segments()?;
        let account = segments.next().filter(|s| !s.is_empty())?.to_owned();
        let container = segments.next().filter(|s| !s.is_empty())?.to_owned();
        let prefix = segments.collect::<Vec<_>>().join("/");
        return Some(AzuriteLocation {
            account,
            container,
            prefix,
        });
    }

    None
}

/// Extract the raw SAS query string (`k1=v1&k2=v2&…`) from a vended credential,
/// if it carries an Azure User Delegation / service SAS.
fn azure_sas_token(credential: &TemporaryCredential) -> Option<String> {
    use unitycatalog_common::temporary_credentials::v1::temporary_credential::Credentials;
    match credential.credentials.as_ref()? {
        Credentials::AzureUserDelegationSas(sas) => Some(sas.sas_token.clone()),
        _ => None,
    }
}

/// Returns a [`UCStore`] whose prefix is `store.prefix() + extra`, leaving
/// the underlying bucket-rooted store untouched.
fn extend_prefix(store: UCStore, extra: &str) -> UCStore {
    let mut url = store.url.clone();
    // Append the extra path component(s), keeping the trailing slash.
    {
        let mut segs = url.path_segments_mut().expect("cloud URL has a path");
        segs.pop_if_empty();
        for part in extra.split('/').filter(|p| !p.is_empty()) {
            segs.push(part);
        }
    }
    let new_path = if store.path.as_ref().is_empty() {
        Path::from(extra)
    } else {
        let base = store.path.as_ref().trim_end_matches('/');
        let extra = extra.trim_start_matches('/');
        Path::from(format!("{base}/{extra}"))
    };
    UCStore {
        root: store.root,
        url,
        path: new_path,
    }
}

// Tests use tokio/tempfile/mockito and the native transport; there is no wasm
// test runner in this crate's CI (the wasm build is proven in the query-wasm
// workspace).
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use futures::TryStreamExt;
    // `put`/`PutPayload` are only used by the POSIX-only local-storage tests.
    #[cfg(not(windows))]
    use object_store::{ObjectStoreExt, PutPayload};

    use super::*;

    /// A factory whose UC endpoint points nowhere reachable. Any code path that
    /// tries to vend a credential will fail to connect — so a successful local
    /// operation proves vending was skipped entirely.
    async fn offline_factory() -> UnityObjectStoreFactory {
        UnityObjectStoreFactory::builder()
            // An unroutable, non-listening endpoint: a vend attempt cannot succeed.
            .with_uri("http://127.0.0.1:0/api/2.1/unity-catalog/")
            .with_allow_unauthenticated(true)
            .build()
            .await
            .unwrap()
    }

    /// `file://` URLs are served by a local store and round-trip read/write/list
    /// without any credential-vending call (the factory points at a dead endpoint).
    // Local file:// storage is POSIX-only for now (see `local_store`).
    #[cfg(not(windows))]
    #[tokio::test]
    async fn for_url_file_roundtrips_without_vending() {
        let dir = tempfile::tempdir().unwrap();
        let url = Url::from_directory_path(dir.path()).unwrap();

        let factory = offline_factory().await;
        let store = factory
            .for_url(url.as_str(), Operation::ReadWrite)
            .await
            .unwrap();

        let dyn_store = store.as_dyn();
        let path = object_store::path::Path::from("hello.txt");
        dyn_store
            .put(&path, PutPayload::from_static(b"world"))
            .await
            .unwrap();

        let listing: Vec<_> = dyn_store.list(None).try_collect().await.unwrap();
        assert_eq!(listing.len(), 1, "expected exactly one object");
        assert_eq!(listing[0].location, path);

        let got = dyn_store.get(&path).await.unwrap().bytes().await.unwrap();
        assert_eq!(&got[..], b"world");
    }

    /// `for_path` with a `file://` URL returns a usable store with no network call.
    #[cfg(not(windows))]
    #[tokio::test]
    async fn for_path_file_skips_vending() {
        let dir = tempfile::tempdir().unwrap();
        let url = Url::from_directory_path(dir.path()).unwrap();

        let factory = offline_factory().await;
        // Read-only: the directory already exists, nothing is created.
        let store = factory.for_path(&url, PathOperation::Read).await.unwrap();
        // Empty directory lists to nothing — and crucially, no vend was attempted.
        let listing: Vec<_> = store.as_dyn().list(None).try_collect().await.unwrap();
        assert!(listing.is_empty());
        assert_eq!(store.url(), &url);
    }

    /// A read-write local store auto-creates a missing target directory.
    #[cfg(not(windows))]
    #[tokio::test]
    async fn local_store_read_write_creates_dir() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("not-yet-here");
        let url = Url::from_directory_path(&missing).unwrap();

        let store = local_store(&url, PathOperation::ReadWrite).unwrap();
        assert!(missing.exists(), "ReadWrite must create the root directory");

        let path = object_store::path::Path::from("a.bin");
        store
            .as_dyn()
            .put(&path, PutPayload::from_static(b"x"))
            .await
            .unwrap();
        assert!(missing.join("a.bin").exists());
    }

    /// The unrooted `root()` store resolves the **full** path (the DataFusion
    /// routing-store contract): a write addressed by the full absolute path
    /// round-trips through that same path, and lands on disk under the table
    /// directory.
    ///
    /// On-disk placement is checked with a real `read_dir` of the table
    /// directory, not a native `PathBuf::join` against an object_store `Path`.
    /// POSIX-only: local file:// is gated off on Windows (see `local_store`),
    /// where an unrooted store cannot round-trip a drive-letter absolute path.
    #[cfg(not(windows))]
    #[tokio::test]
    async fn local_store_root_resolves_full_path() {
        let dir = tempfile::tempdir().unwrap();
        let table_dir = dir.path().join("mytable");
        let url = Url::from_directory_path(&table_dir).unwrap();

        let store = local_store(&url, PathOperation::ReadWrite).unwrap();

        // `prefix()` is the table directory; the routing store registers and
        // forwards the full path beneath it unchanged.
        let full = Path::from(format!("{}/part-0.parquet", store.prefix()));
        store
            .root()
            .put(&full, PutPayload::from_static(b"data"))
            .await
            .unwrap();

        // Reading back the same full path proves the unrooted store resolves it
        // consistently — exactly what the DataFusion routing store relies on.
        let got = store
            .root()
            .get(&full)
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();
        assert_eq!(&got[..], b"data");

        // And it landed on disk under the table directory. Check the real
        // filesystem directly (not via an object_store `Path`) so the assertion
        // is OS-agnostic.
        let entries: Vec<_> = std::fs::read_dir(&table_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(entries, vec![std::ffi::OsString::from("part-0.parquet")]);
    }

    /// A non-`file` URL handed to the local helper is a clean error, not a panic.
    /// POSIX-only: on Windows `local_store` rejects every call up front, so the
    /// specific "not a valid local file path" message does not apply.
    #[cfg(not(windows))]
    #[test]
    fn local_store_rejects_non_file_url() {
        let url = Url::parse("s3://bucket/prefix/").unwrap();
        match local_store(&url, PathOperation::Read) {
            Ok(_) => panic!("expected an error for a non-file URL"),
            Err(e) => assert!(
                e.to_string().contains("not a valid local file path"),
                "unexpected error: {e}"
            ),
        }
    }

    /// Building a factory with a dedicated I/O runtime handle succeeds and the
    /// handle is carried through to the factory. The `None` path (no handle) is
    /// exercised everywhere else and must remain the default.
    ///
    /// A plain `#[test]` (not `#[tokio::test]`) so the dedicated I/O `Runtime`
    /// can be dropped here — dropping a `Runtime` inside an async context panics.
    #[test]
    fn build_with_io_runtime_carries_handle() {
        // A dedicated I/O runtime — the canonical segregation pattern (mirrors
        // object_store's own spawn-connector test).
        let io_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let handle = io_runtime.handle().clone();

        // Drive `build()` on a separate, lightweight runtime so this test owns
        // (and can safely drop) `io_runtime` itself.
        let driver = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        driver.block_on(async {
            let factory = UnityObjectStoreFactory::builder()
                .with_uri("http://localhost:8080/api/2.1/unity-catalog/")
                .with_allow_unauthenticated(true)
                .with_io_runtime(handle.clone())
                .build()
                .await
                .unwrap();
            assert!(factory.io_handle.is_some());

            // Clearing via `None` is honoured.
            let factory = UnityObjectStoreFactory::builder()
                .with_uri("http://localhost:8080/api/2.1/unity-catalog/")
                .with_allow_unauthenticated(true)
                .with_io_runtime(handle.clone())
                .with_io_runtime(None)
                .build()
                .await
                .unwrap();
            assert!(factory.io_handle.is_none());
        });
    }

    /// Live test against a Databricks workspace. Requires
    /// `DATABRICKS_HOST` + `DATABRICKS_TOKEN` to be set. Marked `#[ignore]`
    /// because CI shouldn't hit a real workspace.
    ///
    /// The calling runtime is a current-thread runtime with **I/O disabled**
    /// (no `enable_all`); a successful `list` therefore proves every request
    /// was spawned onto the separate, I/O-enabled runtime via the connector —
    /// exactly the assertion object_store's own spawn-connector test makes.
    #[test]
    #[ignore]
    fn list_store_via_temp_credential_on_io_runtime() {
        let databricks_host = std::env::var("DATABRICKS_HOST").unwrap();
        let databricks_token = std::env::var("DATABRICKS_TOKEN").unwrap();

        // Dedicated, I/O-enabled runtime on its own thread.
        let io_runtime = std::thread::spawn(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
        })
        .join()
        .unwrap();
        let io_handle = io_runtime.handle().clone();

        // Calling runtime: current-thread, no I/O driver enabled. Any request
        // not spawned onto `io_handle` would panic ("no reactor running").
        let main_runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        main_runtime.block_on(async move {
            let factory = UnityObjectStoreFactory::builder()
                .with_uri(format!("{databricks_host}/api/2.1/unity-catalog/"))
                .with_token(databricks_token)
                .with_aws_region("eu-north-1".to_string())
                .with_io_runtime(io_handle)
                .build()
                .await
                .unwrap();

            let volume_path = url::Url::parse("s3://open-lakehouse-dev/volumes/").unwrap();
            let store = factory
                .for_path(&volume_path, PathOperation::Read)
                .await
                .unwrap();
            let files: Vec<_> = store.as_dyn().list(None).try_collect().await.unwrap();
            println!("files: {files:?}");
        });
    }

    #[test]
    fn parse_azurite_path_style() {
        let url =
            Url::parse("http://127.0.0.1:10000/devstoreaccount1/mycontainer/some/prefix").unwrap();
        let loc = parse_azurite(&url).expect("should parse path-style azurite URL");
        assert_eq!(loc.account, "devstoreaccount1");
        assert_eq!(loc.container, "mycontainer");
        assert_eq!(loc.prefix, "some/prefix");
    }

    #[test]
    fn parse_azurite_custom_scheme() {
        let url = Url::parse("azurite://mycontainer/some/prefix").unwrap();
        let loc = parse_azurite(&url).expect("should parse azurite:// URL");
        assert_eq!(loc.account, "devstoreaccount1");
        assert_eq!(loc.container, "mycontainer");
        assert_eq!(loc.prefix, "some/prefix");
    }

    #[test]
    fn parse_azurite_localhost_alias() {
        let url = Url::parse("http://localhost:10000/acct/cont/p").unwrap();
        let loc = parse_azurite(&url).expect("localhost is also an azurite host");
        assert_eq!(loc.account, "acct");
        assert_eq!(loc.container, "cont");
        assert_eq!(loc.prefix, "p");
    }

    #[test]
    fn parse_azurite_rejects_real_cloud_urls() {
        // Real Azure, S3, and a non-Azurite localhost port must all be `None`.
        for raw in [
            "https://acct.blob.core.windows.net/container/path",
            "s3://bucket/prefix",
            "http://127.0.0.1:9000/acct/cont/p", // not the Azurite port
        ] {
            let url = Url::parse(raw).unwrap();
            assert!(parse_azurite(&url).is_none(), "should not match: {raw}");
        }
    }

    /// Building a store from an Azurite SAS credential targets the emulator and
    /// roots the prefix at the blob path within the container (not the full URL
    /// path, which also carries `/<account>/<container>`).
    #[tokio::test]
    async fn azurite_store_targets_emulator_and_roots_prefix() {
        use unitycatalog_common::temporary_credentials::v1::AzureUserDelegationSas;

        let url = "http://127.0.0.1:10000/devstoreaccount1/mycontainer/tbl/data";
        let credential = TemporaryCredential {
            expiration_time: now_epoch_millis() + 3_600_000,
            url: url.to_string(),
            credentials: Some(
                AzureUserDelegationSas {
                    // A minimal, well-formed SAS query string. `split_sas` parses it
                    // into query pairs; the emulator never sees it in this test.
                    sas_token: "sv=2021-08-06&ss=b&srt=co&sp=rl&se=2999-01-01T00:00:00Z&sig=AAAA"
                        .to_string(),
                    ..Default::default()
                }
                .into(),
            ),
            ..Default::default()
        };

        let factory = offline_factory().await;
        let securable =
            SecurableRef::Path(Url::parse(url).unwrap(), PathOperation::Read, Some(false));
        // `build_store` does no network I/O — it only constructs the emulator
        // store and computes the prefix — so the dead-endpoint factory is fine.
        let store = factory.build_store(credential, securable).await.unwrap();

        // The container-relative blob prefix, with `/<account>/<container>` stripped.
        assert_eq!(store.prefix(), &Path::from("tbl/data"));
    }

    /// `now_epoch_millis` is defined in the credential-vending crate, not here;
    /// the store crate just needs a future timestamp for the test credential.
    fn now_epoch_millis() -> i64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    // ---- forwarded-identity header forwarding ------------------------------

    /// A JSON `temporary-path-credentials` response carrying a minimal, well-formed
    /// Azurite SAS credential — enough for `for_path` to build an emulator store
    /// with no real cloud I/O. `url` is a path-style Azurite endpoint.
    fn azurite_credential_body() -> String {
        format!(
            r#"{{"expiration_time":{},"url":"http://127.0.0.1:10000/devstoreaccount1/mycontainer/prefix/","azure_user_delegation_sas":{{"sas_token":"sv=2021-08-06&ss=b&srt=co&sp=rl&se=2999-01-01T00:00:00Z&sig=AAAA"}}}}"#,
            now_epoch_millis() + 3_600_000
        )
    }

    /// A factory pointed at the given (mock) UC base URL, authenticated with a
    /// static token — the token the forwarded-identity path must *drop*.
    async fn factory_at(base_url: &str) -> UnityObjectStoreFactory {
        UnityObjectStoreFactory::builder()
            .with_uri(format!("{base_url}/api/2.1/unity-catalog/"))
            .with_token("proxy-service-token".to_string())
            .build()
            .await
            .unwrap()
    }

    /// With a forwarded user, the upstream vend carries `x-forwarded-user` and
    /// drops the proxy's own bearer token (header-replaces-token).
    #[tokio::test]
    async fn forwarded_user_injects_header_and_drops_token() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/2.1/unity-catalog/temporary-path-credentials")
            .match_header("x-forwarded-user", "alice")
            // The static bearer token must NOT ride along on a forwarded call.
            .match_header("authorization", mockito::Matcher::Missing)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(azurite_credential_body())
            .expect(1)
            .create_async()
            .await;

        let factory = factory_at(&server.url()).await;
        let scoped = factory
            .with_forwarded_user("x-forwarded-user", Some("alice"))
            .unwrap();
        let url = Url::parse("abfss://mycontainer@devstoreaccount1/prefix/").unwrap();
        scoped
            .for_path(&url, PathOperation::Read)
            .await
            .expect("vend + store build should succeed against the mock");

        mock.assert_async().await;
    }

    /// An anonymous request (no forwarded user) sends NO `x-forwarded-user` header
    /// and still authenticates with the proxy's static bearer token — unchanged
    /// from today.
    #[tokio::test]
    async fn anonymous_sends_no_forwarded_header_and_keeps_token() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/2.1/unity-catalog/temporary-path-credentials")
            .match_header("x-forwarded-user", mockito::Matcher::Missing)
            .match_header("authorization", "Bearer proxy-service-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(azurite_credential_body())
            .expect(1)
            .create_async()
            .await;

        let factory = factory_at(&server.url()).await;
        // `None` returns the base factory unchanged.
        let scoped = factory
            .with_forwarded_user("x-forwarded-user", None)
            .unwrap();
        let url = Url::parse("abfss://mycontainer@devstoreaccount1/prefix/").unwrap();
        scoped
            .for_path(&url, PathOperation::Read)
            .await
            .expect("vend + store build should succeed against the mock");

        mock.assert_async().await;
    }

    /// The header name is configurable; a custom outgoing header carries the user.
    #[tokio::test]
    async fn forwarded_user_honors_custom_header_name() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/2.1/unity-catalog/temporary-path-credentials")
            .match_header("x-remote-user", "bob")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(azurite_credential_body())
            .expect(1)
            .create_async()
            .await;

        let factory = factory_at(&server.url()).await;
        let scoped = factory
            .with_forwarded_user("x-remote-user", Some("bob"))
            .unwrap();
        let url = Url::parse("abfss://mycontainer@devstoreaccount1/prefix/").unwrap();
        scoped.for_path(&url, PathOperation::Read).await.unwrap();

        mock.assert_async().await;
    }

    /// An invalid header name / value is a clean config error, not a panic.
    #[tokio::test]
    async fn forwarded_user_rejects_invalid_header() {
        let factory = factory_at("http://127.0.0.1:0").await;
        // Space is not a legal header-name character.
        assert!(
            factory
                .with_forwarded_user("bad header", Some("alice"))
                .is_err()
        );
        // A newline is not a legal header-value character.
        assert!(
            factory
                .with_forwarded_user("x-forwarded-user", Some("a\nb"))
                .is_err()
        );
    }

    /// The `ForwardedHeaderService` decorator inserts the header (replacing any
    /// prior value of the same name) and delegates to its inner service.
    #[tokio::test]
    async fn forwarded_header_service_inserts_and_delegates() {
        use olai_http::service::{HttpService, ReqwestService};
        use reqwest::header::{HeaderName, HeaderValue};

        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/echo")
            .match_header("x-forwarded-user", "carol")
            .with_status(204)
            .expect(1)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let svc = ForwardedHeaderService {
            inner: Arc::new(ReqwestService::new(client.clone())),
            name: HeaderName::from_static("x-forwarded-user"),
            value: HeaderValue::from_static("carol"),
        };
        // A pre-existing value of the same header must be overwritten, not appended.
        let mut request = client
            .get(format!("{}/echo", server.url()))
            .build()
            .unwrap();
        request
            .headers_mut()
            .insert("x-forwarded-user", HeaderValue::from_static("stale"));

        let resp = svc.call(request).await.unwrap();
        assert_eq!(resp.status(), 204);
        mock.assert_async().await;
    }
}
