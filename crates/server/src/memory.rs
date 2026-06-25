use std::sync::Arc;

use dashmap::DashMap;
use uuid::Uuid;

use unitycatalog_common::models::{AssociationLabel, ObjectLabel, PropertyMap, Resource};
use unitycatalog_common::models::{ResourceExt, ResourceIdent, ResourceName, ResourceRef};

use unitycatalog_common::{Error, Result};

use unitycatalog_common::services::encryption::EnvelopeEncryptor;

use crate::services::secrets::SecretManager;
use crate::store::{ResourceStore, ResourceStoreReader};

const MAX_PAGE_SIZE: usize = 10000;

/// Resource rows keyed by `(label, uuid)`. The label is part of the key so two
/// resources of different types can share one logical id — the managed-table flow
/// does exactly this (a `Table` adopts its `StagingTable`'s id, so
/// `table_uuid == staging.id`). A bare-uuid key would let the second create
/// overwrite the first.
type ResourceMap = Arc<DashMap<(ObjectLabel, Uuid), Resource>>;

/// Association edges keyed by label, then by `(from_uuid, to_uuid)` so a single
/// source resource can hold many edges of the same label (e.g. an entity with
/// multiple tags). The value carries the target's [`ObjectLabel`] (so the row can
/// be located in [`ResourceMap`], which is keyed by `(label, uuid)`) and the
/// edge's optional properties (e.g. a tag assignment's value). Mirrors Postgres's
/// `to_label`.
type AssociationMap =
    Arc<DashMap<AssociationLabel, DashMap<(Uuid, Uuid), (ObjectLabel, Option<PropertyMap>)>>>;

/// An in-memory implementation of a resource store.
///
/// This store is not intended for production use, but is useful for testing and development.
#[derive(Debug, Clone)]
pub struct InMemoryResourceStore {
    resources: ResourceMap,
    id_map: Arc<DashMap<ObjectLabel, DashMap<ResourceName, Uuid>>>,
    associations: AssociationMap,
    /// Sealed secret blobs keyed by name. Encryption matches the production path so dev/test
    /// behaviour (and the on-disk format) is exercised here too.
    secrets: Arc<DashMap<String, bytes::Bytes>>,
    encryptor: EnvelopeEncryptor,
}

impl InMemoryResourceStore {
    pub fn new(encryptor: EnvelopeEncryptor) -> Self {
        Self {
            resources: DashMap::new().into(),
            id_map: DashMap::new().into(),
            associations: DashMap::new().into(),
            secrets: DashMap::new().into(),
            encryptor,
        }
    }

    fn get_uuid(&self, label: &ObjectLabel, name: &ResourceName) -> Option<Uuid> {
        self.id_map
            .get(label)
            .and_then(|map| map.value().get(name).map(|uuid| *uuid.value()))
    }

    fn remove_uuid(&self, label: &ObjectLabel, name: &ResourceName) -> Option<Uuid> {
        self.id_map
            .get(label)
            .and_then(|map| map.value().remove(name).map(|(_, uuid)| uuid))
    }

    fn new_uuid(&self, label: &ObjectLabel, name: &ResourceName) -> Result<Uuid> {
        self.insert_uuid(label, name, Uuid::now_v7())
    }

    /// Register `uuid` for `(label, name)`, failing if the name is already taken.
    ///
    /// `new_uuid` generates a fresh v7 id; callers that pre-allocate an id (e.g.
    /// managed volumes and staging tables, which embed the id in their storage
    /// path) reserve that exact id here so the persisted row id matches the path.
    fn insert_uuid(&self, label: &ObjectLabel, name: &ResourceName, uuid: Uuid) -> Result<Uuid> {
        if self.get_uuid(label, name).is_some() {
            return Err(Error::AlreadyExists);
        }
        let map = self.id_map.entry(*label).or_default();
        map.insert(name.clone(), uuid);
        Ok(uuid)
    }
}

#[async_trait::async_trait]
impl ResourceStoreReader for InMemoryResourceStore {
    async fn get(&self, id: &ResourceIdent) -> Result<(Resource, ResourceRef)> {
        let resource = match id.as_ref() {
            ResourceRef::Uuid(uuid) => self.resources.get(&(*id.label(), *uuid)),
            ResourceRef::Name(name) => {
                let uuid = self.get_uuid(id.label(), name).ok_or(Error::NotFound)?;
                self.resources.get(&(*id.label(), uuid))
            }
            ResourceRef::Undefined => return Err(Error::NotFound),
        };
        match resource {
            Some(resource) => Ok((
                resource.value().clone(),
                ResourceRef::Uuid(resource.key().1),
            )),
            None => Err(Error::NotFound),
        }
    }

    async fn list(
        &self,
        label: &ObjectLabel,
        namespace: Option<&ResourceName>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<Resource>, Option<String>)> {
        let page_token = page_token.map(|t| Uuid::parse_str(&t)).transpose()?;
        let mut resource_ids = self
            .id_map
            .get(label)
            .map(|map| {
                map.value()
                    .iter()
                    .filter(|entry| {
                        namespace.is_none_or(|ns| entry.key().prefix_matches(ns))
                            && page_token.is_none_or(|t| &t > entry.value())
                    })
                    .map(|entry| *entry.value())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if resource_ids.is_empty() {
            return Ok((Vec::new(), None));
        }
        resource_ids.sort_unstable();

        let max_page_size = usize::min(max_results.unwrap_or(MAX_PAGE_SIZE), MAX_PAGE_SIZE);
        let mut resources = Vec::new();
        let mut last_id = &Uuid::nil();
        for uuid in resource_ids.iter().rev().take(max_page_size) {
            let resource = self
                .resources
                .get(&(*label, *uuid))
                .ok_or(Error::NotFound)?
                .clone();
            last_id = uuid;
            resources.push(resource);
        }
        let next_page_token = (resources.len() == max_page_size).then(|| last_id.to_string());
        Ok((resources, next_page_token))
    }
}

#[async_trait::async_trait]
impl ResourceStore for InMemoryResourceStore {
    async fn create(&self, resource: Resource) -> Result<(Resource, ResourceRef)> {
        // Honor an id the caller pre-allocated (a resource whose id field is set
        // resolves to `ResourceRef::Uuid`); otherwise mint a fresh v7 id. This
        // lets managed volumes and staging tables persist under the same id they
        // embed in their storage path. API callers cannot reach this — request
        // types carry no id field.
        let label = resource.resource_label();
        let name = resource.resource_name();
        let uuid = match resource.resource_ref() {
            ResourceRef::Uuid(id) => {
                // Guard the id's uniqueness within this label too; otherwise a
                // colliding pre-set id would silently overwrite an existing row of
                // the same type. (A different label sharing the id is allowed — see
                // the `resources` keying.)
                if self.resources.contains_key(&(*label, id)) {
                    return Err(Error::AlreadyExists);
                }
                self.insert_uuid(label, &name, id)?
            }
            _ => self.new_uuid(label, &name)?,
        };
        self.resources.insert((*label, uuid), resource.clone());
        Ok((resource, ResourceRef::Uuid(uuid)))
    }

    async fn delete(&self, id: &ResourceIdent) -> Result<()> {
        let uuid = match id.as_ref() {
            ResourceRef::Uuid(uuid) => *uuid,
            ResourceRef::Name(name) => self.get_uuid(id.label(), name).ok_or(Error::NotFound)?,
            ResourceRef::Undefined => return Err(Error::NotFound),
        };
        match self.resources.remove(&(*id.label(), uuid)) {
            Some((_, resource)) => self.remove_uuid(id.label(), &resource.resource_name()),
            None => None,
        };
        Ok(())
    }

    async fn update(
        &self,
        id: &ResourceIdent,
        resource: Resource,
    ) -> Result<(Resource, ResourceRef)> {
        let uuid = match id.as_ref() {
            ResourceRef::Uuid(uuid) => *uuid,
            ResourceRef::Name(name) => self.get_uuid(id.label(), name).ok_or(Error::NotFound)?,
            ResourceRef::Undefined => return Err(Error::NotFound),
        };
        // Need to clone to avoid locking the map while holding a reference to the value
        let existing = self
            .resources
            .get(&(*id.label(), uuid))
            .ok_or(Error::NotFound)?
            .value()
            .clone();
        if existing.resource_label() != resource.resource_label() {
            self.id_map
                .get(existing.resource_label())
                .and_then(|map| map.value().remove(&existing.resource_name()));
            self.id_map
                .entry(*resource.resource_label())
                .or_default()
                .insert(resource.resource_name().clone(), uuid);
        } else if existing.resource_name() != resource.resource_name() {
            self.id_map
                .get(existing.resource_label())
                .and_then(|map| map.value().remove(&existing.resource_name()));
            self.id_map
                .get(existing.resource_label())
                .and_then(|map| map.value().insert(resource.resource_name(), uuid));
        }
        // The label may have changed; drop the row at the old key and write it
        // under the new one. When the label is unchanged these are the same key,
        // collapsing to a plain overwrite.
        self.resources.remove(&(*existing.resource_label(), uuid));
        self.resources
            .insert((*resource.resource_label(), uuid), resource.clone());
        Ok((resource, ResourceRef::Uuid(uuid)))
    }

    async fn add_association(
        &self,
        from: &ResourceIdent,
        to: &ResourceIdent,
        label: &AssociationLabel,
        properties: Option<PropertyMap>,
    ) -> Result<()> {
        let from_uuid = match from.as_ref() {
            ResourceRef::Uuid(uuid) => *uuid,
            ResourceRef::Name(name) => self.get_uuid(from.label(), name).ok_or(Error::NotFound)?,
            ResourceRef::Undefined => return Err(Error::NotFound),
        };
        let to_uuid = match to.as_ref() {
            ResourceRef::Uuid(uuid) => *uuid,
            ResourceRef::Name(name) => self.get_uuid(to.label(), name).ok_or(Error::NotFound)?,
            ResourceRef::Undefined => return Err(Error::NotFound),
        };
        // Each edge stores the label of *its* target so the row can be located in
        // `resources` (keyed by `(label, uuid)`). The forward edge targets `to`;
        // the inverse edge targets `from`.
        let map = self.associations.entry(label.clone()).or_default();
        map.insert((from_uuid, to_uuid), (*to.label(), properties.clone()));
        if let Some(inverse) = label.inverse() {
            let inverse_map = self.associations.entry(inverse).or_default();
            inverse_map.insert((to_uuid, from_uuid), (*from.label(), properties.clone()));
        }
        Ok(())
    }

    async fn remove_association(
        &self,
        from: &ResourceIdent,
        to: &ResourceIdent,
        label: &AssociationLabel,
    ) -> Result<()> {
        let from_uuid = match from.as_ref() {
            ResourceRef::Uuid(uuid) => *uuid,
            ResourceRef::Name(name) => self.get_uuid(from.label(), name).ok_or(Error::NotFound)?,
            ResourceRef::Undefined => return Err(Error::NotFound),
        };
        let to_uuid = match to.as_ref() {
            ResourceRef::Uuid(uuid) => *uuid,
            ResourceRef::Name(name) => self.get_uuid(to.label(), name).ok_or(Error::NotFound)?,
            ResourceRef::Undefined => return Err(Error::NotFound),
        };
        let map = self.associations.get(label).ok_or(Error::NotFound)?;
        map.remove(&(from_uuid, to_uuid));
        if let Some(inverse) = label.inverse() {
            let inverse_map = self.associations.get(&inverse).ok_or(Error::NotFound)?;
            inverse_map.remove(&(to_uuid, from_uuid));
        }
        Ok(())
    }

    async fn list_associations(
        &self,
        resource: &ResourceIdent,
        label: &AssociationLabel,
        target_label: Option<&ResourceIdent>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<ResourceIdent>, Option<String>)> {
        let (page, token) = self
            .collect_associations(resource, label, target_label, max_results, page_token)
            .await?;
        let idents = page.into_iter().map(|(ident, _)| ident).collect();
        Ok((idents, token))
    }

    async fn list_associations_with_properties(
        &self,
        resource: &ResourceIdent,
        label: &AssociationLabel,
        target_label: Option<&ResourceIdent>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<(ResourceIdent, Option<PropertyMap>)>, Option<String>)> {
        self.collect_associations(resource, label, target_label, max_results, page_token)
            .await
    }
}

impl InMemoryResourceStore {
    /// Shared association-listing logic returning each edge's target ident and properties,
    /// with pagination over the target uuid. Used by both `list_associations` and
    /// `list_associations_with_properties`.
    async fn collect_associations(
        &self,
        resource: &ResourceIdent,
        label: &AssociationLabel,
        target_label: Option<&ResourceIdent>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<(ResourceIdent, Option<PropertyMap>)>, Option<String>)> {
        let resource_uuid = match resource.as_ref() {
            ResourceRef::Uuid(uuid) => *uuid,
            ResourceRef::Name(name) => self
                .get_uuid(resource.label(), name)
                .ok_or(Error::NotFound)?,
            ResourceRef::Undefined => {
                return Err(Error::invalid_argument("resource must not be undefined"));
            }
        };
        let target_uuid = target_label
            .map(|tl| match tl.as_ref() {
                ResourceRef::Uuid(uuid) => Ok(*uuid),
                ResourceRef::Name(name) => self.get_uuid(tl.label(), name).ok_or(Error::NotFound),
                ResourceRef::Undefined => Err(Error::invalid_argument(
                    "target resource must not be undefined",
                )),
            })
            .transpose()?;
        let page_token = page_token.map(|t| Uuid::parse_str(&t)).transpose()?;
        // Collect (to_uuid, to_label, properties) for every edge whose source is
        // `resource_uuid`, applying the optional target filter and the pagination
        // cursor. The edge carries the target's label, so we can build the target
        // ident directly without a `resources` lookup.
        let mut edges = self
            .associations
            .get(label)
            .map(|map| {
                map.value()
                    .iter()
                    .filter_map(|entry| {
                        let (from, to) = *entry.key();
                        let (to_label, props) = entry.value().clone();
                        (from == resource_uuid).then_some((to, to_label, props))
                    })
                    .filter(|(to, _, _)| {
                        target_uuid.is_none_or(|uuid| *to == uuid)
                            && page_token.is_none_or(|t| t > *to)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if edges.is_empty() {
            return Ok((Vec::new(), None));
        }
        edges.sort_unstable_by_key(|(to, _, _)| *to);

        let max_page_size = usize::min(max_results.unwrap_or(MAX_PAGE_SIZE), MAX_PAGE_SIZE);
        let mut results = Vec::new();
        let mut last_id = Uuid::nil();
        for (to_uuid, to_label, props) in edges.into_iter().rev().take(max_page_size) {
            last_id = to_uuid;
            results.push((to_label.to_ident(ResourceRef::Uuid(to_uuid)), props));
        }
        let next_page_token = (results.len() == max_page_size).then(|| last_id.to_string());
        Ok((results, next_page_token))
    }
}

#[async_trait::async_trait]
impl SecretManager for InMemoryResourceStore {
    async fn get_secret(&self, secret_name: &str) -> Result<bytes::Bytes> {
        let blob = self
            .secrets
            .get(secret_name)
            .map(|entry| entry.value().clone())
            .ok_or(Error::NotFound)?;
        let plaintext = self.encryptor.open(secret_name, &blob).await?;

        // Lazy KEK rotation: re-wrap under the active KEK if sealed under a retired one (mirrors
        // the Postgres store). Best-effort; never fails the read.
        if let Ok(Some(rewrapped)) = self.encryptor.rewrap(&blob).await {
            self.secrets
                .insert(secret_name.to_string(), bytes::Bytes::from(rewrapped));
        }

        Ok(bytes::Bytes::from(plaintext))
    }

    async fn put_secret(&self, secret_name: &str, secret_value: bytes::Bytes) -> Result<()> {
        let blob = self.encryptor.seal(secret_name, &secret_value).await?;
        self.secrets
            .insert(secret_name.to_string(), bytes::Bytes::from(blob));
        Ok(())
    }

    async fn delete_secret(&self, secret_name: &str) -> Result<()> {
        self.secrets.remove(secret_name).ok_or(Error::NotFound)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unitycatalog_common::models::{Catalog, ObjectLabel};
    use unitycatalog_common::services::encryption::LocalKeyProvider;

    fn test_store() -> InMemoryResourceStore {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        InMemoryResourceStore::new(encryptor)
    }

    #[tokio::test]
    async fn test_get_secret_lazily_rewraps_after_rotation() {
        // Seal a value under v1.
        let v1 = EnvelopeEncryptor::local(LocalKeyProvider::single("v1", vec![0x01; 32]).unwrap());
        let sealed_v1 = v1.seal("cred", b"value").await.unwrap();

        // Build a store whose active KEK is v2 (v1 retired) and plant the v1-sealed blob.
        let rotated = EnvelopeEncryptor::local(
            LocalKeyProvider::new(
                "v2",
                [("v1".into(), vec![0x01; 32]), ("v2".into(), vec![0x02; 32])],
            )
            .unwrap(),
        );
        let store = InMemoryResourceStore::new(rotated);
        store
            .secrets
            .insert("cred".to_string(), bytes::Bytes::from(sealed_v1.clone()));

        // Reading returns the correct plaintext...
        assert_eq!(&store.get_secret("cred").await.unwrap()[..], b"value");
        // ...and the stored blob has been re-wrapped (it differs from the v1 blob, and a
        // v2-only encryptor can now open it).
        let after = store.secrets.get("cred").unwrap().value().clone();
        assert_ne!(&after[..], &sealed_v1[..]);
        let v2_only =
            EnvelopeEncryptor::local(LocalKeyProvider::single("v2", vec![0x02; 32]).unwrap());
        assert_eq!(v2_only.open("cred", &after).await.unwrap(), b"value");
    }

    #[tokio::test]
    async fn test_secret_round_trip() {
        let store = test_store();
        store
            .put_secret("cred", bytes::Bytes::from_static(b"top secret"))
            .await
            .unwrap();
        // Stored blob must be ciphertext, not the plaintext.
        let blob = store.secrets.get("cred").unwrap().value().clone();
        assert!(
            !blob
                .windows(b"top secret".len())
                .any(|w| w == b"top secret")
        );

        let got = store.get_secret("cred").await.unwrap();
        assert_eq!(&got[..], b"top secret");

        // put_secret is an idempotent overwrite.
        store
            .put_secret("cred", bytes::Bytes::from_static(b"rotated"))
            .await
            .unwrap();
        assert_eq!(&store.get_secret("cred").await.unwrap()[..], b"rotated");

        store.delete_secret("cred").await.unwrap();
        assert!(matches!(
            store.get_secret("cred").await.unwrap_err(),
            Error::NotFound
        ));
    }

    #[tokio::test]
    async fn test_create_get_delete() {
        let store = test_store();
        let resource: Resource = Catalog {
            name: "new_catalog".into(),
            ..Default::default()
        }
        .into();
        let (created, reference) = store.create(resource.clone()).await.unwrap();
        assert_eq!(created.resource_name(), resource.resource_name());

        let ident = ObjectLabel::Catalog.to_ident(reference);
        let (retrieved, _) = store.get(&ident).await.unwrap();
        assert_eq!(retrieved, created);

        store.delete(&ident).await.unwrap();
        let result = store.get(&ident).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NotFound));
    }

    #[tokio::test]
    async fn create_honors_pre_allocated_id() {
        use unitycatalog_common::models::volumes::v1::Volume;

        let store = test_store();
        let id = Uuid::new_v4();
        let resource: Resource = Volume {
            name: "vol".into(),
            catalog_name: "cat".into(),
            schema_name: "sch".into(),
            volume_id: id.hyphenated().to_string(),
            ..Default::default()
        }
        .into();

        let (_, reference) = store.create(resource).await.unwrap();
        // The store persists under the supplied id rather than minting a new one.
        assert_eq!(reference, ResourceRef::Uuid(id));
    }

    #[tokio::test]
    async fn create_generates_id_when_absent() {
        // A resource with no id set (the common case) still gets a fresh v7 id.
        let store = test_store();
        let resource: Resource = Catalog {
            name: "cat".into(),
            ..Default::default()
        }
        .into();
        let (_, reference) = store.create(resource).await.unwrap();
        let ResourceRef::Uuid(id) = reference else {
            panic!("expected a uuid reference, got {reference:?}");
        };
        assert_eq!(id.get_version_num(), 7, "store should mint a v7 id");
    }

    #[tokio::test]
    async fn staging_table_and_table_share_one_id() {
        use unitycatalog_common::models::staging_tables::v1::StagingTable;
        use unitycatalog_common::models::tables::v1::Table;

        // The managed-table flow has a Table adopt its StagingTable's id, so both
        // rows live at the same uuid under different labels. They must coexist.
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
            .await
            .unwrap();
        store
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
            .await
            .unwrap();

        // Both are retrievable by their own label-scoped uuid; neither clobbered
        // the other.
        let staging_ident = ObjectLabel::StagingTable.to_ident(ResourceRef::Uuid(id));
        let table_ident = ObjectLabel::Table.to_ident(ResourceRef::Uuid(id));
        let staging: StagingTable = store
            .get(&staging_ident)
            .await
            .unwrap()
            .0
            .try_into()
            .unwrap();
        let table: Table = store.get(&table_ident).await.unwrap().0.try_into().unwrap();
        assert_eq!(staging.id, id_str);
        assert_eq!(table.table_id.as_deref(), Some(id_str.as_str()));
    }

    #[tokio::test]
    async fn create_rejects_duplicate_pre_allocated_id() {
        use unitycatalog_common::models::volumes::v1::Volume;

        let store = test_store();
        let id = Uuid::new_v4();
        let volume = |name: &str| -> Resource {
            Volume {
                name: name.into(),
                catalog_name: "cat".into(),
                schema_name: "sch".into(),
                volume_id: id.hyphenated().to_string(),
                ..Default::default()
            }
            .into()
        };
        store.create(volume("a")).await.unwrap();
        // A different name but the same pre-allocated id must not overwrite the
        // existing row (Postgres relies on the id primary key for this).
        let res = store.create(volume("b")).await;
        assert!(matches!(res, Err(Error::AlreadyExists)), "{res:?}");
    }

    #[tokio::test]
    async fn test_list() {
        let store = test_store();
        let resource: Resource = Catalog {
            name: "new_catalog".into(),
            ..Default::default()
        }
        .into();
        let (created, _) = store.create(resource.clone()).await.unwrap();

        let (resources, next) = store
            .list(&ObjectLabel::Catalog, None, None, None)
            .await
            .unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0], created);
        assert!(next.is_none());

        // add more resources
        let resource: Resource = Catalog {
            name: "new_catalog2".into(),
            ..Default::default()
        }
        .into();
        store.create(resource).await.unwrap();
        let resource: Resource = Catalog {
            name: "new_catalog3".into(),
            ..Default::default()
        }
        .into();
        store.create(resource).await.unwrap();

        let (resources, next) = store
            .list(&ObjectLabel::Catalog, None, Some(2), None)
            .await
            .unwrap();
        assert_eq!(resources.len(), 2);
        assert!(next.is_some());

        let (resources, next) = store
            .list(&ObjectLabel::Catalog, None, Some(2), next)
            .await
            .unwrap();
        assert_eq!(resources.len(), 1);
        assert!(next.is_none());
    }

    #[tokio::test]
    async fn test_provider_round_trip() {
        use unitycatalog_common::models::providers::v1::{Provider, ProviderAuthenticationType};

        let store = test_store();
        let resource: Resource = Provider {
            name: "acme".into(),
            authentication_type: ::buffa::EnumValue::Known(ProviderAuthenticationType::Token),
            comment: Some("inbound share from acme".into()),
            ..Default::default()
        }
        .into();

        // Create exercises the Resource::Provider -> Object conversion.
        let (created, reference) = store.create(resource.clone()).await.unwrap();
        assert_eq!(created.resource_name(), resource.resource_name());

        // Get exercises the Object -> Resource::Provider conversion and the
        // hand-written ObjectLabel::Provider -> ResourceIdent mapping.
        let ident = ObjectLabel::Provider.to_ident(reference);
        let (retrieved, _) = store.get(&ident).await.unwrap();
        assert_eq!(retrieved, created);
        let provider: Provider = retrieved.try_into().unwrap();
        assert_eq!(provider.name, "acme");
        assert_eq!(provider.comment.as_deref(), Some("inbound share from acme"));

        // List by the Provider label.
        let (resources, _) = store
            .list(&ObjectLabel::Provider, None, None, None)
            .await
            .unwrap();
        assert_eq!(resources.len(), 1);

        store.delete(&ident).await.unwrap();
        assert!(matches!(
            store.get(&ident).await.unwrap_err(),
            Error::NotFound
        ));
    }
}
