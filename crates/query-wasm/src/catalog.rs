//! Async catalog resolve pass: fetch Unity Catalog metadata for a query's table
//! references, vend credentials, register each per-table object store on the
//! shared session, and build the kernel snapshot — populating a query-scoped map
//! the synchronous log UDTFs and data-table registration read.
//!
//! This mirrors the *shape* of the native resolver (`crates/datafusion`'s
//! `UnityCatalogProviderList` chain) without implementing DataFusion's
//! `AsyncCatalogProviderList` traits: those are `Send + Sync` under
//! `#[async_trait]`, but the browser fetch backend (`UcFetchStore`, reqwest-wasm)
//! is `!Send`, so the resolve pass runs off a `?Send` trait of our own. The
//! mechanics carry over unchanged — ad-hoc per-session resolution, store
//! registration *inside* the resolve pass, a routing store behind DataFusion's
//! coarse `scheme://host` registry key.
//!
//! [`UcTableResolver`] is transport-abstracted so native tests drive the full
//! resolve/register path with an [`InMemoryResolver`] instead of the wasm-only
//! [`UcRestResolver`].

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use datafusion::execution::context::SessionContext;
use object_store::ObjectStore;
use object_store::path::Path;
use olai_delta_df::{RoutingObjectStore, SnapshotRef, bucket_key};
use url::Url;

use crate::engine::{TableAddress, open_snapshot};
use crate::error::{Error, Result};

/// A Unity Catalog table resolved to everything the session needs to scan it: the
/// fetchable table URL, the per-table object store, the kernel snapshot, and the
/// version the snapshot pins.
///
/// Cloning is cheap — `store` and `snapshot` are `Arc`-shared, `table_url` is a
/// small owned `Url`.
#[derive(Clone)]
pub struct ResolvedTable {
    /// The table root as a directly fetchable URL (the store is registered under
    /// this URL's `scheme://host` and routed by its path prefix).
    pub table_url: Url,
    /// The vended-credential object store, rooted at the storage origin.
    pub store: Arc<dyn ObjectStore>,
    /// The kernel snapshot, built list-free + engine-free from the discovered
    /// `_delta_log` manifest.
    pub snapshot: SnapshotRef,
    /// The pinned table version the snapshot reads.
    pub table_version: u64,
}

/// The query-scoped resolved-table map: `catalog.schema.table` (lowercased,
/// consistently with `resolve_table_references(.., true)`) → [`ResolvedTable`].
///
/// Populated by the async resolve pass before planning, then read synchronously
/// by the log UDTFs (`call_with_args` is a pure lookup) and by data-table
/// registration. `std::sync::Mutex` suffices — the wasm runtime is
/// single-threaded, so this adds no dependency and never contends.
#[derive(Clone, Default)]
pub struct ResolvedTables {
    inner: Arc<Mutex<HashMap<String, ResolvedTable>>>,
}

impl ResolvedTables {
    /// A fresh, empty map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert `table` under `addr`'s full name (lowercased).
    pub fn insert(&self, addr: &TableAddress, table: ResolvedTable) {
        self.inner
            .lock()
            .expect("ResolvedTables mutex poisoned")
            .insert(lookup_key(addr), table);
    }

    /// Look up the resolved table for `addr`, cloning it out of the map. Returns
    /// `None` when the table was not pre-resolved.
    pub fn get(&self, addr: &TableAddress) -> Option<ResolvedTable> {
        self.inner
            .lock()
            .expect("ResolvedTables mutex poisoned")
            .get(&lookup_key(addr))
            .cloned()
    }
}

/// The map key for `addr`: the full `catalog.schema.table` name lowercased so
/// lookups agree with DataFusion's `resolve_table_references(.., true)`
/// normalization (unquoted identifiers fold to lowercase).
fn lookup_key(addr: &TableAddress) -> String {
    addr.full_name().to_ascii_lowercase()
}

/// The per-`scheme://host` [`RoutingObjectStore`]s a single resolve pass has
/// registered on its session, so a second table sharing a bucket routes into the
/// same router rather than clobbering the first.
///
/// The resolve pass owns this (mirroring the native resolver's
/// `UnityContext.routers` `DashMap`) instead of reading routers back off the
/// DataFusion runtime — the runtime getter would force an `ObjectStoreUrl` round
/// trip and a fragile `Arc` downcast. `std::sync::Mutex` suffices: the wasm
/// runtime is single-threaded.
#[derive(Clone, Default)]
pub struct SessionRouters {
    routers: Arc<Mutex<HashMap<String, RoutingObjectStore>>>,
}

impl SessionRouters {
    /// An empty router set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register `store` on `ctx`'s runtime behind a [`RoutingObjectStore`] keyed
    /// on `table_url`'s `scheme://host`, routing `table_url`'s path prefix to
    /// `store`.
    ///
    /// This is the store-registration step the resolve pass owns (the native
    /// resolver does the equivalent in `UnityContext::register_table_store`). It
    /// fixes the latent same-origin clobber: `delta_engine_session` registers
    /// exactly one store per origin, so two tables in one bucket with distinct
    /// credentials would overwrite each other. Routing behind the coarse
    /// `scheme://host` key lets each table's store serve only its own path
    /// prefix. Single-table never triggers the clobber, but the resolver is built
    /// correct for the multi-table follow-up.
    ///
    /// Registering a second table under an origin already seen this session
    /// reuses that origin's router (adding a route), so the runtime keeps one
    /// routing store per bucket with every table's prefix registered on it.
    pub fn register(&self, ctx: &SessionContext, table_url: &Url, store: Arc<dyn ObjectStore>) {
        let host_key = bucket_key(table_url);
        let bucket_url = match Url::parse(&format!("{host_key}/")) {
            Ok(url) => url,
            // A `scheme://host` that fails to re-parse is not a URL we can
            // register under; leave the (single) store `delta_engine_session`
            // registered in place so single-table still works.
            Err(_) => return,
        };

        let mut routers = self.routers.lock().expect("SessionRouters mutex poisoned");
        let router = routers.entry(host_key).or_insert_with(|| {
            let router = RoutingObjectStore::new();
            // Register the router once per origin; the runtime keys on
            // scheme://host, so re-registering the same instance is harmless but
            // unnecessary.
            ctx.runtime_env()
                .register_object_store(&bucket_url, Arc::new(router.clone()));
            router
        });
        router.register(
            Path::from_url_path(table_url.path()).unwrap_or_default(),
            store,
        );
    }
}

/// Resolves a Unity Catalog [`TableAddress`] to a [`ResolvedTable`] against a
/// shared session.
///
/// `#[async_trait(?Send)]` because the wasm transport (`UcFetchStore` /
/// reqwest-wasm) is `!Send`; the native [`InMemoryResolver`] is `Send` but the
/// trait stays `?Send` so both fit one signature. Implementations MUST register
/// the per-table store on `ctx` (via `routers.register`) before building the
/// snapshot, since snapshot construction reads the log through that store.
#[async_trait::async_trait(?Send)]
pub trait UcTableResolver {
    /// Resolve `addr`, registering its object store on `ctx` (through `routers`)
    /// and returning the snapshot + store + pinned version.
    async fn resolve(
        &self,
        ctx: &SessionContext,
        routers: &SessionRouters,
        addr: &TableAddress,
    ) -> Result<ResolvedTable>;
}

// =====================================================================
// Native test resolver
// =====================================================================

/// A resolver backed by pre-built in-memory stores, for native tests.
///
/// Maps `catalog.schema.table` (lowercased) to a `(store, table_url,
/// latest_version)` triple; `resolve` registers the routed store on the session,
/// discovers the log, and builds the snapshot — exercising the full
/// resolve/register/open path that [`UcRestResolver`] runs in the browser,
/// without any wasm-only dependency.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct InMemoryResolver {
    tables: HashMap<String, InMemoryTable>,
}

#[cfg(not(target_arch = "wasm32"))]
struct InMemoryTable {
    store: Arc<dyn ObjectStore>,
    table_url: Url,
    /// `Some` for catalog-managed tables (drives `max_catalog_version` +
    /// bounds log discovery), `None` for filesystem/external tables.
    latest_version: Option<u64>,
}

#[cfg(not(target_arch = "wasm32"))]
impl InMemoryResolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a fixture table under `full_name` (`catalog.schema.table`).
    pub fn with_table(
        mut self,
        full_name: &str,
        store: Arc<dyn ObjectStore>,
        table_url: Url,
        latest_version: Option<u64>,
    ) -> Self {
        self.tables.insert(
            full_name.to_ascii_lowercase(),
            InMemoryTable {
                store,
                table_url,
                latest_version,
            },
        );
        self
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait(?Send)]
impl UcTableResolver for InMemoryResolver {
    async fn resolve(
        &self,
        ctx: &SessionContext,
        routers: &SessionRouters,
        addr: &TableAddress,
    ) -> Result<ResolvedTable> {
        let table = self
            .tables
            .get(&addr.full_name().to_ascii_lowercase())
            .ok_or_else(|| {
                Error::UnityCatalog(format!("no fixture table for `{}`", addr.full_name()))
            })?;

        // Register the per-table store on the shared session before building the
        // snapshot (which reads the log through it), exactly as the wasm path does.
        routers.register(ctx, &table.table_url, Arc::clone(&table.store));

        let table_path = Path::from_url_path(table.table_url.path())
            .map_err(|e| Error::InvalidUrl(format!("table path: {e}")))?;
        let log =
            crate::resolve::discover_log(&table.store, &table_path, table.latest_version).await?;
        let version = log.version;
        let snapshot = open_snapshot(ctx, &table.table_url, log, table.latest_version).await?;
        Ok(ResolvedTable {
            table_url: table.table_url.clone(),
            store: Arc::clone(&table.store),
            snapshot,
            table_version: version,
        })
    }
}

// =====================================================================
// wasm REST resolver
// =====================================================================

/// The browser resolver: runs the UC REST resolve → credentials → store → open
/// pipeline (the body of the old `open_for`) against a live Unity Catalog.
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub struct UcRestResolver {
    base_url: Url,
    auth_token: Option<String>,
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
impl UcRestResolver {
    /// Create a resolver talking to the UC REST API at `base_url`.
    pub fn new(base_url: Url, auth_token: Option<String>) -> Self {
        Self {
            base_url,
            auth_token,
        }
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[async_trait::async_trait(?Send)]
impl UcTableResolver for UcRestResolver {
    async fn resolve(
        &self,
        ctx: &SessionContext,
        routers: &SessionRouters,
        addr: &TableAddress,
    ) -> Result<ResolvedTable> {
        use crate::fetch_store::UcFetchStore;
        use crate::resolve::{discover_log, plan_table};
        use crate::uc::UcClient;

        // 1. Resolve the table through Unity Catalog and gate on the v1 envelope.
        let uc = UcClient::new(self.base_url.clone(), self.auth_token.clone());
        let loaded = uc
            .load_table(&addr.catalog, &addr.schema, &addr.table)
            .await?;
        let plan = plan_table(&loaded)?;
        let credential = uc.read_table_credentials(&plan.table_uuid).await?;

        // 2. Vended credential → browser-fetchable store.
        let storage = crate::creds::resolve_storage(&plan.location, &credential)?;
        let table_path = Path::from_url_path(storage.table_url.path())
            .map_err(|e| Error::InvalidUrl(format!("table path: {e}")))?;
        let store: Arc<dyn ObjectStore> = Arc::new(UcFetchStore::try_new(
            storage.table_url.clone(),
            &storage.headers,
        )?);

        // 3. Register the routed store on the shared session, discover the log,
        //    and build the snapshot async-native (no prime). `latest_version` is
        //    `Some` only for catalog-managed tables — exactly when the kernel
        //    needs it as `max_catalog_version`.
        routers.register(ctx, &storage.table_url, Arc::clone(&store));
        let log = discover_log(&store, &table_path, plan.latest_version).await?;
        let version = log.version;
        let snapshot = open_snapshot(ctx, &storage.table_url, log, plan.latest_version).await?;
        Ok(ResolvedTable {
            table_url: storage.table_url,
            store,
            snapshot,
            table_version: version,
        })
    }
}
