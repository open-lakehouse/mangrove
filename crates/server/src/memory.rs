//! In-memory resource store for tests and local development.
//!
//! [`InMemoryResourceStore`] is a thin composition over
//! [`olai_store::InMemoryStore`] wrapped in an
//! [`olai_store::ManagedObjectStore`] (for inline sensitive-field sealing) and
//! lifted to the typed [`ResourceStore`] API by
//! [`ObjectStoreAdapter`](unitycatalog_common::store::ObjectStoreAdapter). It
//! mirrors the durable backends (sqlite/postgres) so the same code paths —
//! object/association storage and inline secret sealing — are exercised in
//! tests, without a database.

use std::str::FromStr;
use std::sync::Arc;

use olai_store::{InMemoryStore, ManagedObjectStore, ResourceRegistry};
use unitycatalog_common::models::AssociationLabel;
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::labels::RESOURCE_DESCRIPTORS;
use unitycatalog_common::services::encryption::EnvelopeEncryptor;
use unitycatalog_common::store::{ObjectStoreAdapter, ProvidesResourceStore, ResourceStore};

/// Map an [`AssociationLabel`] string to its inverse label string, for the
/// generic store's inverse-edge resolver (mirrors the sqlite backend).
fn inverse_resolver(label: &str) -> Option<String> {
    AssociationLabel::from_str(label)
        .ok()
        .and_then(|l| l.inverse())
        .map(|inv| inv.to_string())
}

/// The concrete store stack backing the in-memory backend: a registry-aware,
/// encrypting object store over an in-memory graph, lifted to [`ResourceStore`].
type MemoryAdapter =
    ObjectStoreAdapter<ManagedObjectStore<ObjectLabel, InMemoryStore<ObjectLabel>>>;

/// An in-memory implementation of a resource store.
///
/// Not intended for production use, but useful for testing and development. Like
/// the durable backends, credential secret fields are sealed inline on the
/// object row (see [`ManagedObjectStore`]); there is no separate secret store.
#[derive(Clone)]
pub struct InMemoryResourceStore {
    store: Arc<MemoryAdapter>,
}

impl InMemoryResourceStore {
    pub fn new(encryptor: EnvelopeEncryptor) -> Self {
        let inner = InMemoryStore::<ObjectLabel>::with_inverse(inverse_resolver);
        let registry = ResourceRegistry::from_static(RESOURCE_DESCRIPTORS);
        let managed = ManagedObjectStore::with_encryptor(inner, encryptor, registry);
        Self {
            store: Arc::new(ObjectStoreAdapter::new(managed)),
        }
    }
}

impl ProvidesResourceStore for InMemoryResourceStore {
    fn store(&self) -> &dyn ResourceStore {
        self.store.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unitycatalog_common::models::{Catalog, ObjectLabel, ResourceExt, ResourceRef};
    use unitycatalog_common::services::encryption::LocalKeyProvider;
    use unitycatalog_common::store::{ResourceStore, ResourceStoreReader};
    use unitycatalog_common::{Error, Result};
    use uuid::Uuid;

    fn test_store() -> InMemoryResourceStore {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        InMemoryResourceStore::new(encryptor)
    }

    #[tokio::test]
    async fn test_create_get_delete() -> Result<()> {
        let store = test_store();
        let resource: unitycatalog_common::models::Resource = Catalog {
            name: "new_catalog".into(),
            ..Default::default()
        }
        .into();
        let (created, reference) = store.create(resource.clone()).await?;
        assert_eq!(created.resource_name(), resource.resource_name());

        let ident = ObjectLabel::Catalog.to_ident(reference);
        let (retrieved, _) = store.get(&ident).await?;
        assert_eq!(retrieved, created);

        store.delete(&ident).await?;
        let result = store.get(&ident).await;
        assert!(matches!(result.unwrap_err(), Error::NotFound));
        Ok(())
    }

    #[tokio::test]
    async fn create_honors_pre_allocated_id() -> Result<()> {
        use unitycatalog_common::models::volumes::v1::Volume;

        let store = test_store();
        let id = Uuid::new_v4();
        let resource: unitycatalog_common::models::Resource = Volume {
            name: "vol".into(),
            catalog_name: "cat".into(),
            schema_name: "sch".into(),
            volume_id: id.hyphenated().to_string(),
            ..Default::default()
        }
        .into();

        let (_, reference) = store.create(resource).await?;
        // The store persists under the supplied id rather than minting a new one.
        assert_eq!(reference, ResourceRef::Uuid(id));
        Ok(())
    }

    #[tokio::test]
    async fn create_generates_id_when_absent() -> Result<()> {
        // A resource with no id set (the common case) still gets a fresh minted id.
        let store = test_store();
        let resource: unitycatalog_common::models::Resource = Catalog {
            name: "cat".into(),
            ..Default::default()
        }
        .into();
        let (_, reference) = store.create(resource).await?;
        let ResourceRef::Uuid(id) = reference else {
            panic!("expected a uuid reference, got {reference:?}");
        };
        assert!(!id.is_nil(), "store should mint a non-nil id");
        Ok(())
    }

    #[tokio::test]
    async fn create_at_same_uuid_is_rejected() -> Result<()> {
        use unitycatalog_common::models::staging_tables::v1::StagingTable;
        use unitycatalog_common::models::tables::v1::Table;

        // The object store keys objects by a single uuid: one object per uuid is the
        // store's contract. The managed-table flow therefore does not let a Table
        // coexist with its StagingTable at the same id — it *replaces* the staging
        // reservation atomically (see `ResourceStore::replace_atomically`). A raw
        // create of a second object at an already-occupied uuid must be rejected.
        let store = test_store();
        let id = Uuid::new_v4();
        let id_str = id.hyphenated().to_string();

        store
            .create(
                StagingTable {
                    name: "t".into(),
                    catalog_name: "cat".into(),
                    schema_name: "sch".into(),
                    id: id_str.clone(),
                    ..Default::default()
                }
                .into(),
            )
            .await?;
        // Creating a Table at the same uuid (a different label) must fail: the id is
        // already occupied by the staging reservation.
        let res = store
            .create(
                Table {
                    name: "t".into(),
                    catalog_name: "cat".into(),
                    schema_name: "sch".into(),
                    table_id: Some(id_str.clone()),
                    ..Default::default()
                }
                .into(),
            )
            .await;
        assert!(matches!(res, Err(Error::AlreadyExists)), "{res:?}");

        // The staging reservation is untouched and still readable at its uuid.
        let staging_ident = ObjectLabel::StagingTable.to_ident(ResourceRef::Uuid(id));
        let staging: StagingTable = store.get(&staging_ident).await?.0.try_into()?;
        assert_eq!(staging.id, id_str);
        Ok(())
    }

    #[tokio::test]
    async fn create_rejects_duplicate_pre_allocated_id() -> Result<()> {
        use unitycatalog_common::models::volumes::v1::Volume;

        let store = test_store();
        let id = Uuid::new_v4();
        let volume = |name: &str| -> unitycatalog_common::models::Resource {
            Volume {
                name: name.into(),
                catalog_name: "cat".into(),
                schema_name: "sch".into(),
                volume_id: id.hyphenated().to_string(),
                ..Default::default()
            }
            .into()
        };
        store.create(volume("a")).await?;
        // A different name but the same pre-allocated id must not overwrite the
        // existing row (the id primary key enforces this).
        let res = store.create(volume("b")).await;
        assert!(matches!(res, Err(Error::AlreadyExists)), "{res:?}");
        Ok(())
    }

    #[tokio::test]
    async fn test_list() -> Result<()> {
        let store = test_store();
        let resource: unitycatalog_common::models::Resource = Catalog {
            name: "new_catalog".into(),
            ..Default::default()
        }
        .into();
        let (created, _) = store.create(resource.clone()).await?;

        let (resources, next) = store.list(&ObjectLabel::Catalog, None, None, None).await?;
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0], created);
        assert!(next.is_none());

        // add more resources
        for name in ["new_catalog2", "new_catalog3"] {
            let resource: unitycatalog_common::models::Resource = Catalog {
                name: name.into(),
                ..Default::default()
            }
            .into();
            store.create(resource).await?;
        }

        let (resources, next) = store
            .list(&ObjectLabel::Catalog, None, Some(2), None)
            .await?;
        assert_eq!(resources.len(), 2);
        assert!(next.is_some());

        let (resources, next) = store
            .list(&ObjectLabel::Catalog, None, Some(2), next)
            .await?;
        assert_eq!(resources.len(), 1);
        assert!(next.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_provider_round_trip() -> Result<()> {
        use unitycatalog_common::models::providers::v1::{Provider, ProviderAuthenticationType};

        let store = test_store();
        let resource: unitycatalog_common::models::Resource = Provider {
            name: "acme".into(),
            authentication_type: ProviderAuthenticationType::Token as i32,
            comment: Some("inbound share from acme".into()),
            ..Default::default()
        }
        .into();

        // Create exercises the Resource::Provider -> Object conversion.
        let (created, reference) = store.create(resource.clone()).await?;
        assert_eq!(created.resource_name(), resource.resource_name());

        // Get exercises the Object -> Resource::Provider conversion and the
        // hand-written ObjectLabel::Provider -> ResourceIdent mapping.
        let ident = ObjectLabel::Provider.to_ident(reference);
        let (retrieved, _) = store.get(&ident).await?;
        assert_eq!(retrieved, created);
        let provider: Provider = retrieved.try_into()?;
        assert_eq!(provider.name, "acme");
        assert_eq!(provider.comment.as_deref(), Some("inbound share from acme"));

        // List by the Provider label.
        let (resources, _) = store.list(&ObjectLabel::Provider, None, None, None).await?;
        assert_eq!(resources.len(), 1);

        store.delete(&ident).await?;
        assert!(matches!(
            store.get(&ident).await.unwrap_err(),
            Error::NotFound
        ));
        Ok(())
    }

    #[tokio::test]
    async fn update_checked_cas_succeeds_on_match_conflicts_on_stale() -> Result<()> {
        use unitycatalog_common::store::Precondition;

        let store = test_store();
        let (_, reference) = store
            .create(
                Catalog {
                    name: "cat".into(),
                    ..Default::default()
                }
                .into(),
            )
            .await?;
        let ident = ObjectLabel::Catalog.to_ident(reference);

        // Read the current version (etag) to pin the CAS against.
        let (resource, _, version) = store.get_versioned(&ident).await?;
        let mut catalog: Catalog = resource.try_into()?;
        catalog.comment = Some("first".into());

        // CAS with the observed version succeeds and bumps the version.
        let (_, _, new_version) = store
            .update_checked(
                &ident,
                catalog.clone().into(),
                Precondition::Version(version),
            )
            .await?;
        assert!(new_version > version, "version must advance on update");

        // A second CAS against the now-stale original version must conflict.
        catalog.comment = Some("second".into());
        let res = store
            .update_checked(&ident, catalog.into(), Precondition::Version(version))
            .await;
        assert!(matches!(res, Err(Error::Conflict)), "{res:?}");
        Ok(())
    }

    #[tokio::test]
    async fn rename_preserves_id_and_associations() -> Result<()> {
        use unitycatalog_common::models::AssociationLabel;
        use unitycatalog_common::models::ResourceName;
        use unitycatalog_common::store::Precondition;

        let store = test_store();
        // Two catalogs, linked by an association edge.
        let (_, from_ref) = store
            .create(
                Catalog {
                    name: "old_name".into(),
                    ..Default::default()
                }
                .into(),
            )
            .await?;
        let (_, to_ref) = store
            .create(
                Catalog {
                    name: "peer".into(),
                    ..Default::default()
                }
                .into(),
            )
            .await?;
        let from_ident = ObjectLabel::Catalog.to_ident(from_ref.clone());
        let to_ident = ObjectLabel::Catalog.to_ident(to_ref);
        store
            .add_association(&from_ident, &to_ident, &AssociationLabel::ParentOf, None)
            .await?;

        // Rename the catalog; the id is preserved and the edge still resolves.
        let (_, renamed_ref, _) = store
            .rename(
                &from_ident,
                &ResourceName::new(["new_name"]),
                Precondition::Any,
            )
            .await?;
        assert_eq!(renamed_ref, from_ref, "rename must preserve the id");

        let renamed: Catalog = store.get(&from_ident).await?.0.try_into()?;
        assert_eq!(renamed.name, "new_name");

        let (edges, _) = store
            .list_associations(&from_ident, &AssociationLabel::ParentOf, None, None, None)
            .await?;
        assert_eq!(edges.len(), 1, "association must survive the rename");
        Ok(())
    }
}
