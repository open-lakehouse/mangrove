use unitycatalog_common::models::{ObjectLabel, Resource, ResourceExt};
use unitycatalog_storage_access::{LocationKey, ProvidesStorageAccessStore};

use super::ProvidesLocalStoragePolicy;
use crate::api::staging_tables::MANAGED_STORAGE_PREFIX;
use crate::error::from_store;
use crate::services::location::StorageLocationUrl;
use crate::store::ResourceStore;
use crate::{Error, Result};

/// The location of a table or volume overlapping `location`, if there is one.
///
/// Used to enforce that external tables/volumes do not overlap any existing
/// table or volume. The store matches the canonical [`LocationKey`] it keeps with
/// each resource, so every spelling of a location is caught.
pub(crate) async fn find_overlapping_location(
    handler: &(impl ResourceStore + ?Sized),
    location: &StorageLocationUrl,
) -> Result<Option<String>> {
    let filter = LocationKey::new(location.location()).overlapping();
    for label in [ObjectLabel::Table, ObjectLabel::Volume] {
        let (found, _) = handler.search(&label, None, &filter, Some(1), None).await?;
        if let Some(resource) = found.into_iter().next() {
            return Ok(Some(storage_location(resource)));
        }
    }
    Ok(None)
}

/// The names of up to `limit` tables, volumes and agent skills stored at or below
/// the external location URL `url`.
///
/// Used to refuse deleting an external location that still governs securables.
pub(crate) async fn find_location_dependents(
    handler: &(impl ResourceStore + ?Sized),
    url: &str,
    limit: usize,
) -> Result<Vec<String>> {
    let filter = LocationKey::parse(url)?.covered_by();
    let mut names = Vec::new();
    for label in [
        ObjectLabel::Table,
        ObjectLabel::Volume,
        ObjectLabel::AgentSkill,
    ] {
        let remaining = limit - names.len();
        if remaining == 0 {
            break;
        }
        let (found, _) = handler
            .search(&label, None, &filter, Some(remaining), None)
            .await?;
        names.extend(found.iter().map(|r| r.resource_name().to_string()));
    }
    Ok(names)
}

/// The `storage_location` of a stored table, volume or agent skill.
fn storage_location(resource: Resource) -> String {
    match resource {
        Resource::Table(table) => table.storage_location.unwrap_or_default(),
        Resource::Volume(volume) => volume.storage_location,
        Resource::AgentSkill(skill) => skill.storage_location,
        _ => String::new(),
    }
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
    if let Some(existing) = find_overlapping_location(handler, location).await? {
        return Err(Error::invalid_argument(format!(
            "storage location '{}' overlaps existing table or volume location '{existing}'",
            location.raw(),
        )));
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

#[cfg(test)]
mod tests {
    use super::StorageLocationUrl;

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
}
