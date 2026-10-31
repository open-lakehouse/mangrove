use unitycatalog_common::models::tables::v1::Table;
use unitycatalog_common::models::volumes::v1::Volume;

use unitycatalog_storage_access::ProvidesStorageAccessStore;

use super::ProvidesLocalStoragePolicy;
use crate::api::staging_tables::MANAGED_STORAGE_PREFIX;
use crate::error::from_store;
use crate::services::location::StorageLocationUrl;
use crate::store::ResourceStore;
use crate::{Error, Result};

/// All non-empty `storage_location`s currently registered for tables and
/// volumes, parsed into [`StorageLocationUrl`]s.
///
/// Used to enforce that external tables/volumes do not overlap any existing
/// table or volume. Entries without a storage location, or whose location fails
/// to parse, are skipped — matching the tolerance of the other helpers here.
///
/// TODO(roeap): just listing all tables and volumes could be very inefficient; this
/// needs an indexed property/prefix query on the resource store.
pub(crate) async fn list_table_volume_locations(
    handler: &(impl ResourceStore + ?Sized),
) -> Result<Vec<StorageLocationUrl>> {
    let mut out = Vec::new();

    let (tables, _) = handler
        .list(&unitycatalog_common::ObjectLabel::Table, None, None, None)
        .await?;
    for resource in tables {
        let table: Table = resource.try_into()?;
        if let Some(loc) = table.storage_location.filter(|s| !s.is_empty())
            && let Ok(url) = StorageLocationUrl::parse(&loc)
        {
            out.push(url);
        }
    }

    let (volumes, _) = handler
        .list(&unitycatalog_common::ObjectLabel::Volume, None, None, None)
        .await?;
    for resource in volumes {
        let volume: Volume = resource.try_into()?;
        if !volume.storage_location.is_empty()
            && let Ok(url) = StorageLocationUrl::parse(&volume.storage_location)
        {
            out.push(url);
        }
    }

    Ok(out)
}

/// Validate the `storage_location` of an external table or volume.
///
/// Enforces the Unity Catalog rules for external securables:
/// 1. the path must not lie within a reserved managed-storage (`__unitystorage`)
///    region (those are owned by managed catalogs/schemas);
/// 2. the path must be contained within a registered external location; and
/// 3. the path must not overlap any existing table or volume.
///
/// Returns [`Error::invalid_argument`] on a violation.
pub(crate) async fn validate_external_storage_location(
    handler: &(impl ResourceStore + ProvidesLocalStoragePolicy + ProvidesStorageAccessStore + ?Sized),
    location: &StorageLocationUrl,
) -> Result<()> {
    // 0. Local (file://) locations must sit within an allowed host root.
    //    Cloud schemes pass through untouched.
    handler.local_storage_policy().check(location)?;

    // 1. Must not define a path inside a reserved managed-storage region. The
    //    `__unitystorage` segment is owned by managed catalogs/schemas; an
    //    external securable here would collide with managed layout, so it is
    //    rejected regardless of any external location that may also cover it.
    if path_contains_managed_segment(location) {
        return Err(Error::invalid_argument(format!(
            "storage location '{}' lies within a reserved managed-storage \
             ('{}') region; external tables and volumes must use a location \
             outside managed storage",
            location.raw(),
            MANAGED_STORAGE_PREFIX,
        )));
    }

    // 2. Must reside within a registered external location.
    if !is_covered(handler, location).await? {
        return Err(Error::invalid_argument(format!(
            "storage location '{}' is not contained within any registered external location",
            location.raw()
        )));
    }

    // 3. Must not overlap any existing table or volume.
    for existing in list_table_volume_locations(handler).await? {
        if locations_overlap(location, &existing) {
            return Err(Error::invalid_argument(format!(
                "storage location '{}' overlaps existing table or volume location '{}'",
                location.raw(),
                existing.raw()
            )));
        }
    }

    Ok(())
}

/// Validate a client-provided managed `storage_root` for a catalog or schema.
///
/// Mirrors the Unity Catalog reference (`CatalogService`/`SchemaService`
/// `AuthorizeExpression`): a managed storage root set by the client must
/// 1. pass the local-storage policy (a `file://` root must sit within an
///    allowed host root; cloud schemes pass through);
/// 2. NOT lie within a reserved `__unitystorage` region — the server appends
///    that segment itself when materializing the storage location, so a client
///    root already inside one is a layout collision; and
/// 3. be contained within a registered external location.
///
/// Unlike [`validate_external_storage_location`] this does **not** check overlap
/// with existing table/volume locations: the managed root is a *parent prefix*
/// under which managed leaves (`__unitystorage/.../tables/<id>`) are later
/// minted, so a prefix-overlap with an existing managed leaf is expected, not a
/// violation.
///
/// Applies only to a client-supplied root. The metastore-level default root and
/// the dev-server single-local-root fallback are server configuration and are
/// not required to be covered by an external location.
pub(crate) async fn validate_managed_storage_root(
    handler: &(impl ProvidesLocalStoragePolicy + ProvidesStorageAccessStore + ?Sized),
    location: &StorageLocationUrl,
) -> Result<()> {
    // 1. Local (file://) roots must sit within an allowed host root.
    handler.local_storage_policy().check(location)?;

    // 2. Must not already lie within a reserved managed-storage region; the
    //    server owns the `__unitystorage` layout and appends it itself.
    if path_contains_managed_segment(location) {
        return Err(Error::invalid_argument(format!(
            "managed storage_root '{}' must not lie within a reserved \
             managed-storage ('{}') region",
            location.raw(),
            MANAGED_STORAGE_PREFIX,
        )));
    }

    // 3. Must be contained within a registered external location.
    if !is_covered(handler, location).await? {
        return Err(Error::invalid_argument(format!(
            "managed storage_root '{}' is not contained within any registered external location",
            location.raw()
        )));
    }

    Ok(())
}

/// Whether a registered external location covers `location`.
async fn is_covered(
    handler: &(impl ProvidesStorageAccessStore + ?Sized),
    location: &StorageLocationUrl,
) -> Result<bool> {
    Ok(handler
        .storage_access()
        .covering_location(location.location())
        .await
        .map_err(from_store)?
        .is_some())
}

/// Whether a storage location lies within a reserved managed-storage region,
/// i.e. any path segment equals the `__unitystorage` prefix.
///
/// Segment-aware (not a substring match): a directory literally named
/// `__unitystorage` triggers it, but a name that merely contains the text — e.g.
/// `my__unitystorage_data` — does not. The canonical path is checked, so every
/// spelling of a cloud location is caught.
pub(crate) fn path_contains_managed_segment(location: &StorageLocationUrl) -> bool {
    location
        .path()
        .parts()
        .any(|part| part.as_ref() == MANAGED_STORAGE_PREFIX)
}

/// Whether two storage locations overlap: one is equal to, or nested within,
/// the other. Unity Catalog forbids overlapping governed storage regions
/// (external locations, tables, volumes), so this is the core guard used when
/// registering them. The check is symmetric, respects path-segment boundaries
/// and compares canonical locations, so that, e.g., `abfss://` and `https://`
/// spellings of the same Azure location are caught.
pub(crate) fn locations_overlap(a: &StorageLocationUrl, b: &StorageLocationUrl) -> bool {
    a.overlaps(b)
}

#[cfg(test)]
mod tests {
    use super::{StorageLocationUrl, locations_overlap};

    #[test]
    fn locations_overlap_is_symmetric_on_nesting() {
        let parse = |s| StorageLocationUrl::parse(s).unwrap();
        // Descendant overlaps ancestor, in both argument orders.
        assert!(locations_overlap(
            &parse("s3://b/data"),
            &parse("s3://b/data/sub")
        ));
        assert!(locations_overlap(
            &parse("s3://b/data/sub"),
            &parse("s3://b/data")
        ));
        // Exact-equal paths overlap.
        assert!(locations_overlap(
            &parse("s3://b/data"),
            &parse("s3://b/data")
        ));
        // Disjoint paths and different buckets do not overlap.
        assert!(!locations_overlap(
            &parse("s3://b/data"),
            &parse("s3://b/other")
        ));
        assert!(!locations_overlap(
            &parse("s3://b/data"),
            &parse("s3://c/data")
        ));
    }

    #[test]
    fn managed_segment_is_detected_segment_aware() {
        use super::path_contains_managed_segment;
        let parse = |s| StorageLocationUrl::parse(s).unwrap();
        // A literal __unitystorage segment is detected, anywhere in the path.
        assert!(path_contains_managed_segment(&parse(
            "s3://b/uc/__unitystorage/catalogs/cid"
        )));
        assert!(path_contains_managed_segment(&parse(
            "s3://b/__unitystorage/tables/tid"
        )));
        // A name that merely contains the text is NOT a match (segment-aware).
        assert!(!path_contains_managed_segment(&parse(
            "s3://b/my__unitystorage_data/x"
        )));
        assert!(!path_contains_managed_segment(&parse(
            "s3://b/uc/external/x"
        )));
    }

    #[test]
    fn locations_overlap_via_storage_location_url() {
        let parse = |s| StorageLocationUrl::parse(s).unwrap();
        assert!(locations_overlap(
            &parse("s3://b/data"),
            &parse("s3://b/data/sub")
        ));
        assert!(!locations_overlap(
            &parse("s3://b/data"),
            &parse("s3://b/data-secret")
        ));
    }
}
