//! Object store wiring for Unity Catalog backed tables.
//!
//! Unity Catalog vends temporary credentials that are scoped to a *sub-path*
//! of a bucket (the table's storage location), but DataFusion's
//! [`ObjectStoreRegistry`] resolves stores by `scheme://host` only — the path
//! is discarded. Two tables in the same bucket therefore collide on a single
//! registry key. [`RoutingObjectStore`] sits behind that coarse key and
//! dispatches each operation to the correct per-table store based on the full
//! request path.
//!
//! This is a fresh copy of the native `crates/datafusion` routing store, retyped
//! onto [`delta_kernel::object_store`] (the `object_store` re-export the rest of
//! this crate uses — see [`crate::session`]). The native crate does not depend on
//! `olai-delta-df`, so sharing the type would risk an `object_store` version
//! split; two copies keyed on different `object_store` resolution paths is the
//! deliberate choice. The mechanics — longest-prefix routing, register-once per
//! bucket — are identical.
//!
//! [`ObjectStoreRegistry`]: datafusion::execution::object_store::ObjectStoreRegistry

use std::fmt;
use std::ops::Range;
use std::sync::{Arc, RwLock};

use bytes::Bytes;
use delta_kernel::object_store::path::Path;
use delta_kernel::object_store::{
    CopyOptions, GetOptions, GetResult, ListResult, MultipartUpload, ObjectMeta, ObjectStore,
    ObjectStoreExt, PutMultipartOptions, PutOptions, PutPayload, PutResult, Result,
};
use futures::stream::{self, BoxStream, StreamExt};
use url::Url;

/// An [`ObjectStore`] that dispatches operations to per-prefix backing stores.
///
/// DataFusion's `ObjectStoreRegistry` resolves stores by `scheme://host` only,
/// so every table living in the same bucket maps to one registry entry. When
/// those tables are backed by Unity Catalog credentials vended for distinct
/// sub-paths, a single backing store cannot serve them. A `RoutingObjectStore`
/// is registered under the bucket key and routes each request to the store
/// registered for the longest matching path prefix.
///
/// Each backing store is expected to be rooted at the bucket (e.g. a fetch store
/// rooted at the storage origin), so the full request path is forwarded
/// unchanged.
/// A registered route: a path `prefix` and the store that serves paths beneath
/// it. `prefix` is matched on path-segment boundaries.
type Route = (Path, Arc<dyn ObjectStore>);

#[derive(Clone)]
pub struct RoutingObjectStore {
    /// Routes ordered so that lookups can pick the longest matching prefix.
    routes: Arc<RwLock<Vec<Route>>>,
}

impl RoutingObjectStore {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register `store` to serve every path at or beneath `prefix`.
    ///
    /// Re-registering an existing prefix replaces the backing store (e.g. when
    /// a table is resolved again in a later query). Routes are kept sorted by
    /// descending prefix length so the first match is the most specific.
    pub fn register(&self, prefix: Path, store: Arc<dyn ObjectStore>) {
        let mut routes = self.routes.write().unwrap();
        routes.retain(|(p, _)| p != &prefix);
        routes.push((prefix, store));
        // Longest (most segments) first.
        routes.sort_by_key(|(p, _)| std::cmp::Reverse(p.parts_count()));
    }

    /// Resolve the backing store for `location`, returning the most specific
    /// registered prefix match.
    fn route(&self, location: &Path) -> Result<Arc<dyn ObjectStore>> {
        let routes = self.routes.read().unwrap();
        routes
            .iter()
            .find(|(prefix, _)| is_path_prefix(prefix, location))
            .map(|(_, store)| store.clone())
            .ok_or_else(|| delta_kernel::object_store::Error::NotFound {
                path: location.to_string(),
                source: format!("no registered object store routes path '{location}'").into(),
            })
    }

    /// Resolve the backing store for a list `prefix`. A `None` prefix (list
    /// everything) has no meaningful route here, so it errors — callers always
    /// scan within a known table location.
    fn route_prefix(&self, prefix: Option<&Path>) -> Result<Arc<dyn ObjectStore>> {
        match prefix {
            Some(prefix) => self.route(prefix),
            None => Err(not_implemented("list without a prefix")),
        }
    }
}

fn not_implemented(operation: &str) -> delta_kernel::object_store::Error {
    delta_kernel::object_store::Error::NotImplemented {
        operation: operation.to_string(),
        implementer: "RoutingObjectStore".to_string(),
    }
}

impl Default for RoutingObjectStore {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for RoutingObjectStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let routes = self.routes.read().unwrap();
        f.debug_struct("RoutingObjectStore")
            .field(
                "prefixes",
                &routes
                    .iter()
                    .map(|(p, _)| p.to_string())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl fmt::Display for RoutingObjectStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RoutingObjectStore")
    }
}

/// `scheme://host` portion of a cloud URL, matching how DataFusion's
/// `ObjectStoreRegistry` keys stores. This is the key a [`RoutingObjectStore`] is
/// registered under on the session runtime; the per-table stores routed within it
/// are distinguished by path prefix.
pub fn bucket_key(url: &Url) -> String {
    format!(
        "{}://{}",
        url.scheme(),
        &url[url::Position::BeforeHost..url::Position::AfterPort]
    )
}

/// Returns true when `path` is equal to, or lives beneath, `prefix`, matching
/// only on full path-segment boundaries so that `db/t1` does not match
/// `db/t10`. An empty prefix matches everything.
fn is_path_prefix(prefix: &Path, path: &Path) -> bool {
    let mut prefix_parts = prefix.parts();
    let mut path_parts = path.parts();
    loop {
        match (prefix_parts.next(), path_parts.next()) {
            // Consumed the whole prefix: `path` is at or below it.
            (None, _) => return true,
            // Prefix still has segments but path ran out, or a segment differs.
            (Some(_), None) => return false,
            (Some(a), Some(b)) if a != b => return false,
            (Some(_), Some(_)) => continue,
        }
    }
}

#[async_trait::async_trait]
impl ObjectStore for RoutingObjectStore {
    async fn put_opts(
        &self,
        location: &Path,
        payload: PutPayload,
        opts: PutOptions,
    ) -> Result<PutResult> {
        self.route(location)?
            .put_opts(location, payload, opts)
            .await
    }

    async fn put_multipart_opts(
        &self,
        location: &Path,
        opts: PutMultipartOptions,
    ) -> Result<Box<dyn MultipartUpload>> {
        self.route(location)?
            .put_multipart_opts(location, opts)
            .await
    }

    async fn get_opts(&self, location: &Path, options: GetOptions) -> Result<GetResult> {
        self.route(location)?.get_opts(location, options).await
    }

    async fn get_ranges(&self, location: &Path, ranges: &[Range<u64>]) -> Result<Vec<Bytes>> {
        self.route(location)?.get_ranges(location, ranges).await
    }

    fn list(&self, prefix: Option<&Path>) -> BoxStream<'static, Result<ObjectMeta>> {
        match self.route_prefix(prefix) {
            Ok(store) => store.list(prefix),
            Err(e) => stream::once(async move { Err(e) }).boxed(),
        }
    }

    async fn list_with_delimiter(&self, prefix: Option<&Path>) -> Result<ListResult> {
        self.route_prefix(prefix)?.list_with_delimiter(prefix).await
    }

    async fn copy_opts(&self, from: &Path, to: &Path, options: CopyOptions) -> Result<()> {
        let store = self.route(from)?;
        // Cross-store copies are not supported; both ends must route to the
        // same backing store.
        if !Arc::ptr_eq(&store, &self.route(to)?) {
            return Err(not_implemented("copy across distinct routed stores"));
        }
        store.copy_opts(from, to, options).await
    }

    fn delete_stream(
        &self,
        locations: BoxStream<'static, Result<Path>>,
    ) -> BoxStream<'static, Result<Path>> {
        let this = self.clone();
        locations
            .map(move |location| {
                let this = this.clone();
                async move {
                    let location = location?;
                    this.route(&location)?.delete(&location).await?;
                    Ok(location)
                }
            })
            .buffered(10)
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use delta_kernel::object_store::memory::InMemory;

    fn store_with(path: &str, body: &str) -> Arc<dyn ObjectStore> {
        let store = InMemory::new();
        let path = Path::from(path);
        let payload = PutPayload::from(body.to_string());
        futures::executor::block_on(store.put(&path, payload)).unwrap();
        Arc::new(store)
    }

    #[test]
    fn is_path_prefix_matches_on_segment_boundaries() {
        let prefix = Path::from("db/t1");
        assert!(is_path_prefix(&prefix, &Path::from("db/t1")));
        assert!(is_path_prefix(
            &prefix,
            &Path::from("db/t1/_delta_log/0.json")
        ));
        // must NOT match a sibling whose name shares a textual prefix
        assert!(!is_path_prefix(&prefix, &Path::from("db/t10/part.parquet")));
        assert!(!is_path_prefix(&prefix, &Path::from("db/t2/part.parquet")));
        // empty prefix matches everything
        assert!(is_path_prefix(
            &Path::from(""),
            &Path::from("anything/here")
        ));
    }

    #[tokio::test]
    async fn routes_to_longest_matching_prefix() {
        let routing = RoutingObjectStore::new();
        let t1 = store_with("db/t1/data", "one");
        let t2 = store_with("db/t2/data", "two");
        routing.register(Path::from("db/t1"), t1);
        routing.register(Path::from("db/t2"), t2);

        let got_one = routing.get(&Path::from("db/t1/data")).await.unwrap();
        assert_eq!(&got_one.bytes().await.unwrap()[..], b"one");

        let got_two = routing.get(&Path::from("db/t2/data")).await.unwrap();
        assert_eq!(&got_two.bytes().await.unwrap()[..], b"two");
    }

    #[tokio::test]
    async fn unmatched_path_is_not_found() {
        let routing = RoutingObjectStore::new();
        routing.register(Path::from("db/t1"), store_with("db/t1/data", "one"));
        let err = routing.get(&Path::from("db/other/data")).await.unwrap_err();
        assert!(matches!(
            err,
            delta_kernel::object_store::Error::NotFound { .. }
        ));
    }

    #[tokio::test]
    async fn re_register_replaces_backing_store() {
        let routing = RoutingObjectStore::new();
        routing.register(Path::from("db/t1"), store_with("db/t1/data", "old"));
        routing.register(Path::from("db/t1"), store_with("db/t1/data", "new"));
        let got = routing.get(&Path::from("db/t1/data")).await.unwrap();
        assert_eq!(&got.bytes().await.unwrap()[..], b"new");
    }

    /// Two tables sharing one bucket origin (`db/`) but distinct sub-paths route
    /// independently — the case DataFusion's coarse `scheme://host` registry key
    /// cannot serve with a single backing store, and the reason this store exists
    /// on the multi-table resolve path.
    #[tokio::test]
    async fn two_tables_one_origin_route_independently() {
        let routing = RoutingObjectStore::new();
        // Both tables live under the same bucket; only the path distinguishes them.
        routing.register(
            Path::from("db/sales/orders"),
            store_with("db/sales/orders/_delta_log/0.json", "orders-log"),
        );
        routing.register(
            Path::from("db/sales/returns"),
            store_with("db/sales/returns/_delta_log/0.json", "returns-log"),
        );

        let orders = routing
            .get(&Path::from("db/sales/orders/_delta_log/0.json"))
            .await
            .unwrap();
        assert_eq!(&orders.bytes().await.unwrap()[..], b"orders-log");

        let returns = routing
            .get(&Path::from("db/sales/returns/_delta_log/0.json"))
            .await
            .unwrap();
        assert_eq!(&returns.bytes().await.unwrap()[..], b"returns-log");
    }
}
