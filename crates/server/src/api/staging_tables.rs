use itertools::Itertools;
use unitycatalog_common::models::ResourceName;
use unitycatalog_common::models::catalogs::v1::Catalog;
use unitycatalog_common::models::schemas::v1::Schema;
use unitycatalog_common::models::staging_tables::v1::*;
use unitycatalog_common::models::{ObjectLabel, ResourceIdent};

use super::{RequestContext, SecuredAction};
pub use crate::codegen::staging_tables::StagingTableHandler;
use crate::policy::{Permission, Policy, Principal};
use crate::services::location::StorageLocationUrl;
use crate::services::{ProvidesLocalStoragePolicy, ProvidesManagedStorageRoot};
use crate::store::ResourceStore;
use crate::{Error, Result};

/// Managed-storage prefix appended to a catalog/schema storage root. Mirrors the
/// Unity Catalog reference implementation's `__unitystorage` segment. A
/// catalog/schema location that already carries the prefix is used as-is.
///
/// This path segment is reserved for managed storage: external securables must
/// never define a location inside a `__unitystorage` region (see
/// `validate_external_storage_location`).
pub(crate) const MANAGED_STORAGE_PREFIX: &str = "__unitystorage";

impl SecuredAction for CreateStagingTableRequest {
    fn resource(&self) -> ResourceIdent {
        // Creating a staging table requires CREATE on the parent schema.
        ResourceIdent::schema(ResourceName::new([
            self.catalog_name.as_str(),
            self.schema_name.as_str(),
        ]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

#[async_trait::async_trait]
impl<
    T: ResourceStore
        + Policy<RequestContext>
        + ProvidesLocalStoragePolicy
        + ProvidesManagedStorageRoot,
> StagingTableHandler<RequestContext> for T
{
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_staging_table(
        &self,
        request: CreateStagingTableRequest,
        context: RequestContext,
    ) -> Result<StagingTable> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;

        // A managed table cannot share a name with an existing table in the schema.
        let table_ident = ResourceIdent::table(ResourceName::new([
            request.catalog_name.as_str(),
            request.schema_name.as_str(),
            request.name.as_str(),
        ]));
        if self.get(&table_ident).await.is_ok() {
            return Err(Error::AlreadyExists);
        }

        // Allocate the immutable id and location. The id is the future managed
        // table's id; the store persists the row under this same id.
        let id = uuid::Uuid::new_v4();
        let parent =
            resolve_managed_parent_location(self, &request.catalog_name, &request.schema_name)
                .await?;
        let staging_location = child_location(&parent, "tables", &id.to_string());

        let created_by = match context.recipient() {
            Principal::User(name) => Some(name.clone()),
            Principal::Anonymous => None,
        };

        let staging_table = StagingTable {
            id: id.hyphenated().to_string(),
            name: request.name,
            catalog_name: request.catalog_name,
            schema_name: request.schema_name,
            staging_location,
            created_by,
            stage_committed: false,
            created_at: None,
        };

        Ok(self.create(staging_table.into()).await?.0.try_into()?)
    }
}

/// Resolve the managed *parent* location that a managed table/volume created in
/// `catalog.schema` appends its `tables/{id}` / `volumes/{id}` segment to.
///
/// Resolution order mirrors the Unity Catalog reference implementation's
/// `getManagedStorageLocation`:
/// 1. the schema's `storage_location` if set (already id-materialized at create
///    time as `<root>/__unitystorage/schemas/<schema_id>`),
/// 2. else the catalog's `storage_location` if set (materialized as
///    `<root>/__unitystorage/catalogs/<catalog_id>`),
/// 3. else the metastore-level managed storage root, with a bare
///    `__unitystorage` prefix appended (no entity-id segment — matches the
///    reference's fallback branch, which has no catalog/schema id to embed),
/// 4. else the single allowed local root (dev-server convenience), likewise
///    with a bare `__unitystorage` prefix.
///
/// The returned location already carries the `__unitystorage` prefix; callers
/// must not add it again. A catalog/schema with no resolvable root cannot host
/// managed tables, so this returns `invalid_argument`.
pub(crate) async fn resolve_managed_parent_location(
    handler: &(impl ResourceStore + ProvidesLocalStoragePolicy + ProvidesManagedStorageRoot + ?Sized),
    catalog_name: &str,
    schema_name: &str,
) -> Result<String> {
    // The schema/catalog `storage_location` is already materialized with the
    // `__unitystorage/{schemas,catalogs}/<id>` segment (see `create_schema` /
    // `create_catalog`); use it verbatim.
    let schema_ident = ResourceIdent::schema(ResourceName::new([catalog_name, schema_name]));
    let schema: Schema = handler.get(&schema_ident).await?.0.try_into()?;
    let location = if let Some(loc) = schema.storage_location.filter(|s| !s.is_empty()) {
        loc
    } else {
        let catalog_ident = ResourceIdent::catalog(ResourceName::new([catalog_name]));
        let catalog: Catalog = handler.get(&catalog_ident).await?.0.try_into()?;
        if let Some(loc) = catalog.storage_location.filter(|s| !s.is_empty()) {
            loc
        } else if let Some(root) = handler.managed_storage_root().filter(|s| !s.is_empty()) {
            // Fall back to the metastore-level managed storage root. The
            // reference's fallback has no catalog/schema id to embed, so we
            // append only the bare `__unitystorage` prefix.
            managed_prefix(root)
        } else if let [sole_root] = handler.local_storage_policy().allowed_roots() {
            // No schema, catalog, nor metastore root. When the server allows
            // exactly one local root, use it as the implicit managed root so a
            // purely-local dev server hosts managed tables without any
            // storage_root configured. More than one allowed root is ambiguous →
            // fall through to the error.
            let root = url::Url::from_directory_path(sole_root)
                .map_err(|_| {
                    Error::invalid_argument(format!(
                        "allowed local storage root '{}' is not an absolute path",
                        sole_root.display()
                    ))
                })?
                .to_string();
            managed_prefix(&root)
        } else {
            return Err(Error::invalid_argument(format!(
                "neither schema '{catalog_name}.{schema_name}', catalog \
                 '{catalog_name}', nor the metastore has a managed storage root \
                 configured; cannot create a managed table"
            )));
        }
    };

    // A local (`file://`) managed root must sit within an allowed host root.
    // Cloud schemes pass through: the managed location is, by definition, the
    // catalog/schema/metastore managed storage region — it is not an external
    // location and is not required to match one. (External securables are the
    // only ones bound to a registered external location; see
    // `validate_external_storage_location`.)
    handler
        .local_storage_policy()
        .check(&StorageLocationUrl::parse(&location)?)?;
    Ok(location)
}

/// Find the staging table reserved at `staging_location`, if any.
///
/// Staging rows are keyed by their (immutable) location; `CreateStagingTable`
/// guarantees uniqueness via a fresh UUID per location. Returns `NotFound` if no
/// staging table was allocated there.
///
/// TODO(perf): this lists all staging tables; once the store supports property
/// filters, query by `staging_location` directly (mirrors the existing TODOs on
/// `list_external_locations` / `list_table_volume_locations`).
pub(crate) async fn find_staging_table_by_location(
    handler: &(impl ResourceStore + ?Sized),
    staging_location: &str,
) -> Result<StagingTable> {
    let (resources, _) = handler
        .list(&ObjectLabel::StagingTable, None, None, None)
        .await?;
    // Match slash-insensitively: the createTable client sends the location with
    // a trailing slash (kernel/Url normalization in create_managed_table) while
    // the staging reservation stored it without one.
    let needle = staging_location.trim_end_matches('/');
    resources
        .into_iter()
        .map(StagingTable::try_from)
        .filter_ok(|st| st.staging_location.trim_end_matches('/') == needle)
        .next()
        .ok_or(Error::NotFound)?
        .map_err(Error::from)
}

/// Append the managed-storage prefix to `root` unless it already ends with it.
///
/// `managed_prefix("s3://b/root") == "s3://b/root/__unitystorage"`. Idempotent.
pub(crate) fn managed_prefix(root: &str) -> String {
    let trimmed = root.trim_end_matches('/');
    if trimmed.ends_with(MANAGED_STORAGE_PREFIX) {
        trimmed.to_string()
    } else {
        format!("{trimmed}/{MANAGED_STORAGE_PREFIX}")
    }
}

/// Materialize a catalog's managed storage location from its `storage_root`:
/// `<root>/__unitystorage/catalogs/<catalog_id>`. Mirrors the reference's
/// `getManagedLocationForCatalog`.
pub(crate) fn catalog_location(storage_root: &str, catalog_id: &str) -> String {
    format!("{}/catalogs/{catalog_id}", managed_prefix(storage_root))
}

/// Materialize a schema's managed storage location from its own storage root:
/// `<root>/__unitystorage/schemas/<schema_id>`. Mirrors the reference's
/// `getManagedLocationForSchema`. Independent of the parent catalog's location.
pub(crate) fn schema_location(storage_root: &str, schema_id: &str) -> String {
    format!("{}/schemas/{schema_id}", managed_prefix(storage_root))
}

/// Append a child entity segment to an already-materialized managed parent
/// location: `<parent>/<kind>/<id>` where `kind` is `"tables"` or `"volumes"`.
///
/// Unlike the catalog/schema helpers this does **not** add `__unitystorage` —
/// the prefix already lives in `parent`. Mirrors the reference's
/// `getManagedLocationForTable` / `getManagedLocationForVolume`.
pub(crate) fn child_location(parent: &str, kind: &str, id: &str) -> String {
    format!("{}/{kind}/{id}", parent.trim_end_matches('/'))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use unitycatalog_common::models::catalogs::v1::CreateCatalogRequest;
    use unitycatalog_common::models::credentials::v1::{
        AwsIamRoleConfig, CreateCredentialRequest, Purpose,
    };
    use unitycatalog_common::models::external_locations::v1::CreateExternalLocationRequest;
    use unitycatalog_common::models::schemas::v1::CreateSchemaRequest;
    use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

    use super::*;
    use crate::api::{CatalogHandler, CredentialHandler, ExternalLocationHandler, SchemaHandler};
    use crate::memory::InMemoryResourceStore;
    use crate::policy::ConstantPolicy;
    use crate::services::ServerHandler;

    fn handler() -> ServerHandler<RequestContext> {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
        ServerHandler::try_new_tokio(policy, store).unwrap()
    }

    fn ctx() -> RequestContext {
        RequestContext {
            recipient: Principal::anonymous(),
        }
    }

    /// Register a credential + external location at `url` so a managed
    /// `storage_root` under it passes the coverage check. `tag` keeps the
    /// credential/location names unique across multiple roots in one test.
    async fn make_covering_location(h: &ServerHandler<RequestContext>, tag: &str, url: &str) {
        h.create_credential(
            CreateCredentialRequest {
                name: format!("{tag}-cred"),
                purpose: Purpose::Storage as i32,
                aws_iam_role: Some(AwsIamRoleConfig {
                    role_arn: "arn:aws:iam::123456789012:role/test".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_external_location(
            CreateExternalLocationRequest {
                name: format!("{tag}-el"),
                url: url.to_string(),
                credential_name: format!("{tag}-cred"),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
    }

    async fn make_catalog(
        h: &ServerHandler<RequestContext>,
        name: &str,
        storage_root: Option<&str>,
    ) {
        // A client-supplied root must be covered by a registered external
        // location; register one so create_catalog's coverage check passes.
        if let Some(root) = storage_root {
            make_covering_location(h, &format!("cat-{name}"), root).await;
        }
        h.create_catalog(
            CreateCatalogRequest {
                name: name.to_string(),
                storage_root: storage_root.map(str::to_string),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
    }

    async fn make_schema(
        h: &ServerHandler<RequestContext>,
        catalog: &str,
        name: &str,
        storage_root: Option<&str>,
    ) {
        if let Some(root) = storage_root {
            make_covering_location(h, &format!("sch-{catalog}-{name}"), root).await;
        }
        h.create_schema(
            CreateSchemaRequest {
                name: name.to_string(),
                catalog_name: catalog.to_string(),
                storage_root: storage_root.map(str::to_string),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
    }

    fn create_req(catalog: &str, schema: &str, name: &str) -> CreateStagingTableRequest {
        CreateStagingTableRequest {
            name: name.to_string(),
            catalog_name: catalog.to_string(),
            schema_name: schema.to_string(),
        }
    }

    #[test]
    fn managed_prefix_is_not_doubled() {
        assert_eq!(managed_prefix("s3://b/root"), "s3://b/root/__unitystorage");
        assert_eq!(managed_prefix("s3://b/root/"), "s3://b/root/__unitystorage");
        assert_eq!(
            managed_prefix("s3://b/root/__unitystorage"),
            "s3://b/root/__unitystorage"
        );
    }

    #[test]
    fn location_helpers_match_reference_layout() {
        assert_eq!(
            catalog_location("s3://b/cat", "cid"),
            "s3://b/cat/__unitystorage/catalogs/cid"
        );
        assert_eq!(
            schema_location("s3://b/sch", "sid"),
            "s3://b/sch/__unitystorage/schemas/sid"
        );
        // child_location does not re-add the managed prefix.
        assert_eq!(
            child_location("s3://b/cat/__unitystorage/catalogs/cid", "tables", "tid"),
            "s3://b/cat/__unitystorage/catalogs/cid/tables/tid"
        );
    }

    #[tokio::test]
    async fn staging_location_derives_from_catalog_location() {
        let h = handler();
        make_catalog(&h, "cat", Some("s3://bucket/cat")).await;
        make_schema(&h, "cat", "sch", None).await;

        // The catalog's materialized location embeds the catalog id.
        let catalog = h
            .get_catalog(
                unitycatalog_common::models::catalogs::v1::GetCatalogRequest {
                    name: "cat".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .unwrap();
        let cat_loc = catalog.storage_location.unwrap();

        let st = h
            .create_staging_table(create_req("cat", "sch", "tbl"), ctx())
            .await
            .unwrap();

        assert!(!st.stage_committed);
        // {catalog.storage_location}/tables/{uuid}
        assert_eq!(
            st.staging_location,
            format!("{cat_loc}/tables/{}", st.id),
            "location should be the catalog location plus tables/<id>"
        );
        assert!(
            st.staging_location
                .starts_with("s3://bucket/cat/__unitystorage/catalogs/"),
            "got {}",
            st.staging_location
        );
    }

    #[tokio::test]
    async fn schema_response_carries_storage_root_and_location() {
        // The create/get response must echo storage_root and the materialized
        // storage_location so clients (e.g. the UI) can display them.
        use unitycatalog_common::models::schemas::v1::GetSchemaRequest;
        let h = handler();
        make_catalog(&h, "cat", Some("s3://bucket/cat")).await;
        make_schema(&h, "cat", "sch", Some("s3://other/sch")).await;

        let got = h
            .get_schema(
                GetSchemaRequest {
                    full_name: "cat.sch".to_string(),
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(got.storage_root.as_deref(), Some("s3://other/sch"));
        let sid = got.schema_id.as_deref().expect("schema id");
        assert_eq!(
            got.storage_location.as_deref(),
            Some(format!("s3://other/sch/__unitystorage/schemas/{sid}").as_str())
        );
    }

    #[tokio::test]
    async fn schema_without_root_has_empty_storage_fields() {
        // A schema created without its own root carries no storage fields; managed
        // securables fall back to the catalog location.
        use unitycatalog_common::models::schemas::v1::GetSchemaRequest;
        let h = handler();
        make_catalog(&h, "cat", Some("s3://bucket/cat")).await;
        make_schema(&h, "cat", "sch", None).await;

        let got = h
            .get_schema(
                GetSchemaRequest {
                    full_name: "cat.sch".to_string(),
                },
                ctx(),
            )
            .await
            .unwrap();
        assert!(got.storage_root.is_none(), "got {:?}", got.storage_root);
        assert!(
            got.storage_location.is_none(),
            "got {:?}",
            got.storage_location
        );
    }

    #[tokio::test]
    async fn schema_storage_location_takes_precedence_over_catalog() {
        let h = handler();
        make_catalog(&h, "cat", Some("s3://bucket/cat")).await;
        make_schema(&h, "cat", "sch", Some("s3://other/sch")).await;

        let st = h
            .create_staging_table(create_req("cat", "sch", "tbl"), ctx())
            .await
            .unwrap();

        // Schema location embeds the schema id and roots under the schema's own
        // storage root, not the catalog's.
        assert!(
            st.staging_location
                .starts_with("s3://other/sch/__unitystorage/schemas/"),
            "got {}",
            st.staging_location
        );
        assert!(
            st.staging_location.ends_with(&format!("/tables/{}", st.id)),
            "got {}",
            st.staging_location
        );
    }

    #[tokio::test]
    async fn missing_storage_root_is_rejected() {
        // No schema/catalog root, no metastore default, and no allowed local
        // roots ⇒ a managed table has nowhere to live.
        let h = handler();
        make_catalog(&h, "cat", Some("s3://bucket/cat")).await;
        // Overwrite the catalog row with one that has no storage_root, bypassing
        // create_catalog's requirement, to exercise the resolution fallback.
        let cat_ident = ResourceIdent::catalog(ResourceName::new(["cat"]));
        let bare = Catalog {
            name: "cat".to_string(),
            ..Default::default()
        };
        h.update(&cat_ident, bare.into()).await.unwrap();
        make_schema(&h, "cat", "sch", None).await;

        let res = h
            .create_staging_table(create_req("cat", "sch", "tbl"), ctx())
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn resolved_local_location_outside_allowlist_is_rejected() {
        // Lazy resolution validates the resolved managed location against the
        // local-storage allowlist. The default handler has an empty allowlist, so
        // a catalog whose stored location is a file:// path (written directly,
        // bypassing create-time validation) is rejected at staging time.
        let h = handler();
        make_catalog(&h, "cat", Some("s3://bucket/cat")).await;
        let cat_ident = ResourceIdent::catalog(ResourceName::new(["cat"]));
        let local = Catalog {
            name: "cat".to_string(),
            storage_location: Some("file:///forbidden/cat/__unitystorage/catalogs/cid".to_string()),
            ..Default::default()
        };
        h.update(&cat_ident, local.into()).await.unwrap();
        make_schema(&h, "cat", "sch", None).await;

        let res = h
            .create_staging_table(create_req("cat", "sch", "tbl"), ctx())
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn resolved_metastore_local_root_outside_allowlist_is_rejected() {
        // The metastore-default fallback is likewise allowlist-checked: a file://
        // metastore root with no allowed local roots is rejected at staging time.
        // The catalog is created with a cloud root (so create_catalog's own check
        // passes) then overwritten with a bare row, forcing resolution to fall
        // through to the file:// metastore default.
        let h = handler().with_managed_storage_root(Some("file:///forbidden/meta"));
        make_catalog(&h, "cat", Some("s3://bucket/cat")).await;
        let cat_ident = ResourceIdent::catalog(ResourceName::new(["cat"]));
        let bare = Catalog {
            name: "cat".to_string(),
            ..Default::default()
        };
        h.update(&cat_ident, bare.into()).await.unwrap();
        make_schema(&h, "cat", "sch", None).await;

        let res = h
            .create_staging_table(create_req("cat", "sch", "tbl"), ctx())
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn resolves_from_metastore_default_when_catalog_has_none() {
        // A catalog with no storage_root (e.g. created before the create-time
        // requirement) still resolves a managed table under the metastore root.
        let h = handler().with_managed_storage_root(Some("s3://bucket/meta"));
        make_catalog(&h, "cat", None).await;
        let cat_ident = ResourceIdent::catalog(ResourceName::new(["cat"]));
        let bare = Catalog {
            name: "cat".to_string(),
            ..Default::default()
        };
        h.update(&cat_ident, bare.into()).await.unwrap();
        make_schema(&h, "cat", "sch", None).await;

        let st = h
            .create_staging_table(create_req("cat", "sch", "tbl"), ctx())
            .await
            .unwrap();
        assert!(
            st.staging_location
                .starts_with("s3://bucket/meta/__unitystorage/tables/"),
            "got {}",
            st.staging_location
        );
    }

    #[tokio::test]
    async fn find_by_location_round_trips() {
        let h = handler();
        make_catalog(&h, "cat", Some("s3://bucket/cat")).await;
        make_schema(&h, "cat", "sch", None).await;

        let st = h
            .create_staging_table(create_req("cat", "sch", "tbl"), ctx())
            .await
            .unwrap();

        let found = find_staging_table_by_location(&h, &st.staging_location)
            .await
            .unwrap();
        assert_eq!(found.id, st.id);
        assert_eq!(found.name, "tbl");

        let missing = find_staging_table_by_location(&h, "s3://bucket/cat/tables/nope").await;
        assert!(matches!(missing, Err(Error::NotFound)), "{missing:?}");
    }
}
