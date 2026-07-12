use std::sync::Arc;

use itertools::Itertools;
use olai_store::store::{StoreExec, Transactional};
use olai_store::{AssociationStore, EdgeQuery, ObjectStore, ObjectStoreReader, SecretObjectReader};
use uuid::Uuid;

use crate::models::{AssociationLabel, ObjectLabel, PropertyMap, Resource};
use crate::{Object, ResourceIdent, ResourceName, ResourceRef, Result};

/// Optimistic-concurrency precondition for versioned writes, re-exported from
/// `olai-store` so callers of [`ResourceStore`] need not depend on it directly.
///
/// [`Precondition::Version`] pins a write to the version observed on a prior read
/// ([`ResourceStoreReader::get_versioned`]); a mismatch is a
/// [`Conflict`](crate::Error::Conflict). [`Precondition::Any`] is an unconditional write.
pub use olai_store::Precondition;

/// Convert a stored association `properties` JSON value into a [`PropertyMap`].
///
/// Association properties are persisted as a JSON object (see `add_association`), so a
/// non-object value yields `None`.
fn json_to_property_map(value: serde_json::Value) -> Option<PropertyMap> {
    match value {
        serde_json::Value::Object(map) => Some(map.into_iter().collect()),
        _ => None,
    }
}

#[async_trait::async_trait]
pub trait ResourceStoreReader: Send + Sync + 'static {
    /// Get a resource by its identifier.
    ///
    /// ## Arguments
    /// - `id`: The identifier of the resource to get.
    ///
    /// ## Returns
    /// The resource with the given identifier.
    async fn get(&self, id: &ResourceIdent) -> Result<(Resource, ResourceRef)>;

    /// Get multiple resources by their identifiers.
    ///
    /// ## Arguments
    /// - `ids`: The identifiers of the resources to get.
    ///
    /// ## Returns
    /// The resources with the given identifiers.
    async fn get_many(&self, ids: &[ResourceIdent]) -> Result<Vec<(Resource, ResourceRef)>> {
        let futures = ids.iter().map(|id| self.get(id)).collect_vec();
        Ok(futures::future::try_join_all(futures).await?)
    }

    /// Get a resource with its sensitive fields decrypted and merged into the typed model.
    ///
    /// Ordinary [`get`](Self::get) redacts a resource's `Sensitive` fields; this returns the
    /// unsealed view for the narrow internal callers that need the secret material (e.g.
    /// credential vending). The default is the redacted `get`; a store backed by
    /// `olai_store::ManagedObjectStore` overrides it to merge the sealed fields back in.
    async fn get_with_secrets(&self, id: &ResourceIdent) -> Result<(Resource, ResourceRef)> {
        self.get(id).await
    }

    /// Get a resource together with its current version (etag).
    ///
    /// The `u64` is the store's monotonic per-object version, bumped on every
    /// mutation. Pass it back as a [`Precondition::Version`] on a later
    /// [`update_checked`](ResourceStore::update_checked) or
    /// [`rename`](ResourceStore::rename) to perform a compare-and-swap.
    ///
    /// The default returns version `0` for stores that don't track versions;
    /// [`ObjectStoreAdapter`] overrides it to surface the real version.
    async fn get_versioned(&self, id: &ResourceIdent) -> Result<(Resource, ResourceRef, u64)> {
        let (resource, reference) = self.get(id).await?;
        Ok((resource, reference, 0))
    }

    /// List resources.
    ///
    /// List resources in the store that are children of the given resource.
    /// If the Reference inside the ResourceIdent is [Undefined](crate::ResourceRef::Undefined),
    /// the root of the store is used and resources of the specified type are listed.
    ///
    /// ## Arguments
    /// - `root`: The root resource to list children of.
    /// - `max_results`: The maximum number of results to return.
    /// - `page_token`: The token to use to get the next page of results.
    async fn list(
        &self,
        label: &ObjectLabel,
        namespace: Option<&ResourceName>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<Resource>, Option<String>)>;
}

/// Generic store that can be used to store and retrieve resources.
///
/// Any implementation must conform to the following rules:
/// - Id fields are managed by the store and must be globally unique.
///   If the id field is set on a resource, it can be ignored.
#[async_trait::async_trait]
pub trait ResourceStore: ResourceStoreReader + Send + Sync + 'static {
    /// Create a new resource.
    ///
    /// ## Arguments
    /// - `resource`: The resource to create.
    ///
    /// ## Returns
    /// The created resource.
    ///
    /// Convenience for [`create_versioned`](Self::create_versioned), discarding
    /// the created object's initial version.
    async fn create(&self, resource: Resource) -> Result<(Resource, ResourceRef)> {
        let (resource, reference, _version) = self.create_versioned(resource).await?;
        Ok((resource, reference))
    }

    /// Create a resource, returning its initial store version alongside it.
    ///
    /// The `u64` is the store's monotonic per-object version at creation — the
    /// same counter [`get_versioned`](ResourceStoreReader::get_versioned) reads and
    /// [`update_checked`](Self::update_checked) asserts. Callers that turn the
    /// version into an etag (e.g. the Delta `createTable` path) use this instead of
    /// assuming a fresh object's version, so the etag stays correct even if the
    /// store's initial version is ever nonzero.
    async fn create_versioned(&self, resource: Resource) -> Result<(Resource, ResourceRef, u64)>;

    /// Delete a resource and all connected associations by its identifier.
    ///
    /// The implementing store should delete all associations of the resource
    /// before deleting the resource itself.
    ///
    /// ## Arguments
    /// - `id`: The identifier of the resource to delete.
    async fn delete(&self, id: &ResourceIdent) -> Result<()>;

    /// Atomically delete one resource and create another **in one transaction**.
    ///
    /// Used by the managed-table commit flow, where a `StagingTable` reservation
    /// must be replaced by a `Table` **at the same uuid** (the id is fixed at
    /// staging time and the Delta API protocol depends on it). This is a genuine
    /// **relabel** — `StagingTable` and `Table` are distinct, immutable object
    /// labels — so it cannot be a [`rename`](Self::rename) (which changes the name,
    /// not the label). Because the object store keys objects by a single uuid, the
    /// delete and the create cannot both exist at that id: they must land in one
    /// atomic unit of work so a crash between them cannot leave the reservation
    /// gone without the table, and concurrent readers see either the old row or the
    /// new one, never neither.
    ///
    /// ## Arguments
    /// - `delete`: The identifier of the resource to remove.
    /// - `create`: The resource to create (carrying the adopted id).
    ///
    /// ## Returns
    /// The created resource and its reference.
    ///
    /// Convenience for [`replace_atomically_versioned`](Self::replace_atomically_versioned),
    /// discarding the created object's version.
    async fn replace_atomically(
        &self,
        delete: &ResourceIdent,
        create: Resource,
    ) -> Result<(Resource, ResourceRef)> {
        let (resource, reference, _version) =
            self.replace_atomically_versioned(delete, create).await?;
        Ok((resource, reference))
    }

    /// [`replace_atomically`](Self::replace_atomically), returning the created
    /// object's store version alongside it (see [`create_versioned`](Self::create_versioned)).
    ///
    /// This is a required method with no default: every backend store is
    /// transactional ([`olai_store::store::Transactional`]), so a non-atomic
    /// delete-then-create fallback would be a silent correctness footgun.
    async fn replace_atomically_versioned(
        &self,
        delete: &ResourceIdent,
        create: Resource,
    ) -> Result<(Resource, ResourceRef, u64)>;

    /// Update a resource unconditionally.
    ///
    /// Convenience for [`update_checked`](Self::update_checked) with
    /// [`Precondition::Any`], discarding the returned version.
    ///
    /// ## Arguments
    /// - `id`: The identifier of the resource to update.
    /// - `resource`: The updated resource.
    ///
    /// ## Returns
    /// The updated resource.
    async fn update(
        &self,
        id: &ResourceIdent,
        resource: Resource,
    ) -> Result<(Resource, ResourceRef)> {
        let (resource, reference, _version) =
            self.update_checked(id, resource, Precondition::Any).await?;
        Ok((resource, reference))
    }

    /// Update a resource under an optimistic-concurrency precondition.
    ///
    /// With [`Precondition::Version`] this is a compare-and-swap: the write
    /// succeeds only if the stored version still equals the expected one, else it
    /// fails with [`Conflict`](crate::Error::Conflict). With [`Precondition::Any`]
    /// it is an unconditional overwrite. Returns the resource and its new version.
    async fn update_checked(
        &self,
        id: &ResourceIdent,
        resource: Resource,
        precondition: Precondition,
    ) -> Result<(Resource, ResourceRef, u64)>;

    /// Rename a resource to a new (possibly namespaced) name.
    ///
    /// Preserves the resource's id, label, associations, and sealed secrets —
    /// only the name changes. With [`Precondition::Version`] the rename is a
    /// compare-and-swap against the stored version. Returns the renamed resource
    /// and its new version.
    async fn rename(
        &self,
        id: &ResourceIdent,
        new_name: &ResourceName,
        precondition: Precondition,
    ) -> Result<(Resource, ResourceRef, u64)>;

    /// Add an association between two resources.
    ///
    /// Associations are directed edges between resources with a label and optional properties.
    /// Between two resources must be at most one association with a given label.
    /// Associations are bi-directional, meaning that if an association is added from A to B,
    /// there is also an association from B to A with the inverse label. Some labels are symmetric,
    /// meaning that the inverse label is the same as the label.
    ///
    /// ## Arguments
    /// - `from`: The source resource of the association.
    /// - `to`: The target resource of the association.
    /// - `label`: The label of the association.
    /// - `properties`: Optional properties of the association.
    ///
    /// ## Errors
    /// - [AlreadyExists](crate::Error::AlreadyExists) If the association already exists.
    async fn add_association(
        &self,
        from: &ResourceIdent,
        to: &ResourceIdent,
        label: &AssociationLabel,
        properties: Option<PropertyMap>,
    ) -> Result<()>;

    /// Remove an association between two resources.
    ///
    /// Implementations must remove the inverse association as well.
    ///
    /// ## Arguments
    /// - `from`: The source resource of the association.
    /// - `to`: The target resource of the association.
    /// - `label`: The label of the association.
    ///
    /// ## Errors
    /// - [NotFound](crate::Error::NotFound) If the association does not exist.
    async fn remove_association(
        &self,
        from: &ResourceIdent,
        to: &ResourceIdent,
        label: &AssociationLabel,
    ) -> Result<()>;

    /// List associations of a resource.
    ///
    /// List associations of a resource with the given label.
    ///
    /// ## Arguments
    /// - `resource`: The resource to list associations of.
    /// - `label`: The label of the associations to list.
    /// - `target_label`: The label of the target resource of the associations to list.
    /// - `max_results`: The maximum number of results to return.
    /// - `page_token`: The token to use to get the next page of results.
    ///
    /// ## Returns
    /// The list of associations of the resource with the given label.
    /// The token to use to get the next page of results.
    async fn list_associations(
        &self,
        resource: &ResourceIdent,
        label: &AssociationLabel,
        target_label: Option<&ResourceIdent>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<ResourceIdent>, Option<String>)>;

    /// List associations of a resource together with each association's properties.
    ///
    /// Like [`list_associations`](Self::list_associations), but also returns the
    /// [`PropertyMap`] stored on each association edge (e.g. a tag assignment's value).
    ///
    /// The default implementation delegates to `list_associations` and returns `None`
    /// for every property map; stores that persist association properties should override
    /// this to surface them.
    async fn list_associations_with_properties(
        &self,
        resource: &ResourceIdent,
        label: &AssociationLabel,
        target_label: Option<&ResourceIdent>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<(ResourceIdent, Option<PropertyMap>)>, Option<String>)> {
        let (idents, token) = self
            .list_associations(resource, label, target_label, max_results, page_token)
            .await?;
        Ok((idents.into_iter().map(|i| (i, None)).collect(), token))
    }
}

pub trait ProvidesResourceStore: Send + Sync + 'static {
    fn store(&self) -> &dyn ResourceStore;
}

/// Adapter that implements [`ResourceStore`] for any store implementing
/// the generic [`ObjectStore`] and [`AssociationStore`] traits.
///
/// This bridges the typed `Resource`/`ResourceIdent` API surface to the
/// generic `Object<ObjectLabel>` layer, using the `TryFrom` conversions
/// generated by `object_conversions!`.
pub struct ObjectStoreAdapter<S> {
    store: S,
}

impl<S> ObjectStoreAdapter<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn into_inner(self) -> S {
        self.store
    }
}

impl<S> ObjectStoreAdapter<S>
where
    S: ObjectStoreReader<ObjectLabel>,
{
    /// Resolve a [`ResourceIdent`] to a UUID, fetching by name if necessary.
    async fn resolve_ident(&self, id: &ResourceIdent) -> Result<Uuid> {
        let (label, reference): (&ObjectLabel, &ResourceRef) = (id.as_ref(), id.as_ref());
        match reference {
            ResourceRef::Uuid(uuid) => Ok(*uuid),
            ResourceRef::Name(name) => {
                let object = self.store.get_by_name(*label, name).await?;
                Ok(object.id)
            }
            ResourceRef::Undefined => {
                Err(crate::Error::generic("Cannot resolve undefined resource"))
            }
        }
    }
}

#[async_trait::async_trait]
impl<S> ResourceStoreReader for ObjectStoreAdapter<S>
where
    S: SecretObjectReader<ObjectLabel> + Send + Sync + 'static,
{
    async fn get(&self, id: &ResourceIdent) -> Result<(Resource, ResourceRef)> {
        let (label, reference): (&ObjectLabel, &ResourceRef) = (id.as_ref(), id.as_ref());
        match reference {
            ResourceRef::Uuid(uuid) => {
                let object = self.store.get(uuid).await?;
                Ok((object.try_into()?, ResourceRef::from(id)))
            }
            ResourceRef::Name(name) => {
                let object = self.store.get_by_name(*label, name).await?;
                let id_new = ResourceRef::Uuid(object.id);
                Ok((object.try_into()?, id_new))
            }
            ResourceRef::Undefined => Err(crate::Error::generic("Cannot get undefined resource")),
        }
    }

    /// Surface the stored object's version alongside the resource, so a caller can
    /// pin a later CAS write to it (see [`ResourceStoreReader::get_versioned`]).
    async fn get_versioned(&self, id: &ResourceIdent) -> Result<(Resource, ResourceRef, u64)> {
        let uuid = self.resolve_ident(id).await?;
        let object = self.store.get(&uuid).await?;
        let version = object.version;
        Ok((object.try_into()?, ResourceRef::Uuid(uuid), version))
    }

    async fn list(
        &self,
        label: &ObjectLabel,
        namespace: Option<&ResourceName>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<Resource>, Option<String>)> {
        let (objects, token) = self
            .store
            .list(*label, namespace, max_results, page_token)
            .await?;
        Ok((
            objects
                .into_iter()
                .map(|object| object.try_into())
                .try_collect()?,
            token,
        ))
    }

    /// Override the redacting default: read the object with its sensitive fields decrypted
    /// (via the inner store's [`SecretObjectReader`]) so the typed model is fully hydrated.
    async fn get_with_secrets(&self, id: &ResourceIdent) -> Result<(Resource, ResourceRef)> {
        let uuid = self.resolve_ident(id).await?;
        let object = self.store.get_with_secrets(&uuid).await?;
        Ok((object.try_into()?, ResourceRef::Uuid(uuid)))
    }
}

#[async_trait::async_trait]
impl<S> ResourceStore for ObjectStoreAdapter<S>
where
    S: ObjectStore<ObjectLabel>
        + AssociationStore<ObjectLabel>
        + SecretObjectReader<ObjectLabel>
        + Transactional<ObjectLabel>
        + Send
        + Sync
        + 'static,
{
    async fn create_versioned(&self, resource: Resource) -> Result<(Resource, ResourceRef, u64)> {
        let object: Object = resource.try_into()?;
        // A non-nil id means the caller pre-allocated it (e.g. a managed table
        // adopting its staging reservation's id, or a managed volume embedding
        // the id in its storage path); a nil id lets the store mint a fresh v7.
        // API request types carry no id field, so callers cannot force an id
        // through this path. Mirrors the Postgres backend's `create`.
        let supplied_id = (!object.id.is_nil()).then_some(object.id);
        // Secrets still live in the backend's local secrets table for now, so no
        // sensitive blob is threaded through the generic store here (see the
        // sensitive-field adoption follow-up).
        let created = self
            .store
            .create(
                object.label,
                &object.name,
                object.properties,
                supplied_id,
                None,
            )
            .await?;
        let id = ResourceRef::Uuid(created.id);
        let version = created.version;
        Ok((created.try_into()?, id, version))
    }

    async fn delete(&self, id: &ResourceIdent) -> Result<()> {
        let uuid = self.resolve_ident(id).await?;
        self.store.delete(&uuid).await?;
        Ok(())
    }

    /// Run the delete + create in one backend transaction so the two land atomically
    /// (see the trait method's contract). The transaction closure operates on the raw
    /// `StoreExec` surface, so the delete ident is resolved to a uuid *before* the
    /// transaction begins and the create `Resource` is converted to an `Object` up front.
    /// Neither `StagingTable` nor `Table` (the sole in-tree caller's resources) carries a
    /// sensitive field, so bypassing the managed sealing layer inside the tx is sound.
    async fn replace_atomically_versioned(
        &self,
        delete: &ResourceIdent,
        create: Resource,
    ) -> Result<(Resource, ResourceRef, u64)> {
        let del_uuid = self.resolve_ident(delete).await?;
        let object: Object = create.try_into()?;
        // A non-nil id means the caller pre-allocated it (here: the table adopting the
        // staging reservation's id); mirrors `create`.
        let supplied_id = (!object.id.is_nil()).then_some(object.id);
        let created = self
            .store
            .transaction(Box::new(move |exec: &dyn StoreExec<ObjectLabel>| {
                Box::pin(async move {
                    exec.delete(&del_uuid).await?;
                    exec.create(
                        object.label,
                        &object.name,
                        object.properties,
                        supplied_id,
                        None,
                    )
                    .await
                })
            }))
            .await?;
        let id = ResourceRef::Uuid(created.id);
        let version = created.version;
        Ok((created.try_into()?, id, version))
    }

    async fn update_checked(
        &self,
        id: &ResourceIdent,
        resource: Resource,
        precondition: Precondition,
    ) -> Result<(Resource, ResourceRef, u64)> {
        let uuid = self.resolve_ident(id).await?;
        let object: Object = resource.try_into()?;
        // `precondition` carries any optimistic-concurrency guard: `Version(v)`
        // makes this a compare-and-swap (mismatch → `olai_store::Error::Conflict`),
        // `Any` an unconditional overwrite. Secrets are not threaded here (they are
        // sealed inline by the managed store on create; an update leaves them).
        let updated = self
            .store
            .update(&uuid, object.properties, precondition, None)
            .await?;
        let version = updated.version;
        Ok((updated.try_into()?, uuid.into(), version))
    }

    async fn rename(
        &self,
        id: &ResourceIdent,
        new_name: &ResourceName,
        precondition: Precondition,
    ) -> Result<(Resource, ResourceRef, u64)> {
        let uuid = self.resolve_ident(id).await?;
        // The authoritative name lives in two places that must stay in sync: the
        // object's `name` key (the store's lookup index) and the `name` field
        // inside the serialized `properties` blob (what the typed `Resource`
        // deserializes). olai-store's `rename` re-keys only the former, so we also
        // patch the leaf name in `properties` and write both in one transaction.
        let leaf = new_name
            .path()
            .last()
            .cloned()
            .ok_or_else(|| crate::Error::generic("rename target name is empty"))?;
        let new_name = new_name.clone();
        let renamed = self
            .store
            .transaction(Box::new(move |exec: &dyn StoreExec<ObjectLabel>| {
                Box::pin(async move {
                    // Re-key the object first so a later property write inside the
                    // same tx observes the new name; the precondition guards this
                    // read-modify-write against a concurrent mutation.
                    let obj = exec.rename(&uuid, &new_name, precondition).await?;
                    let mut props = obj
                        .properties
                        .clone()
                        .unwrap_or_else(|| serde_json::Value::Object(Default::default()));
                    if let serde_json::Value::Object(map) = &mut props {
                        map.insert("name".into(), serde_json::Value::String(leaf));
                    }
                    // Unconditional here: the version was already asserted by the
                    // rename above, and this is the same logical write.
                    exec.update(&obj.id, Some(props), Precondition::Any, None)
                        .await
                })
            }))
            .await?;
        let version = renamed.version;
        Ok((renamed.try_into()?, uuid.into(), version))
    }

    async fn add_association(
        &self,
        from: &ResourceIdent,
        to: &ResourceIdent,
        label: &AssociationLabel,
        properties: Option<PropertyMap>,
    ) -> Result<()> {
        let from_id = self.resolve_ident(from).await?;
        let to_id = self.resolve_ident(to).await?;
        let props = properties.map(|p| serde_json::Value::Object(p.into_iter().collect()));
        self.store
            .add(from_id, to_id, label.as_ref(), props)
            .await?;
        Ok(())
    }

    async fn remove_association(
        &self,
        from: &ResourceIdent,
        to: &ResourceIdent,
        label: &AssociationLabel,
    ) -> Result<()> {
        let from_id = self.resolve_ident(from).await?;
        let to_id = self.resolve_ident(to).await?;
        self.store.remove(from_id, to_id, label.as_ref()).await?;
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
        let (associations, token) = self
            .edge_query(resource, label, target_label, max_results, page_token)
            .await?;
        let idents = associations
            .into_iter()
            .map(|assoc| assoc.to_label.to_ident(assoc.to_id))
            .collect();
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
        let (associations, token) = self
            .edge_query(resource, label, target_label, max_results, page_token)
            .await?;
        let entries = associations
            .into_iter()
            .map(|assoc| {
                let props = assoc.properties.and_then(json_to_property_map);
                (assoc.to_label.to_ident(assoc.to_id), props)
            })
            .collect();
        Ok((entries, token))
    }
}

impl<S> ObjectStoreAdapter<S>
where
    S: AssociationStore<ObjectLabel> + ObjectStoreReader<ObjectLabel> + Send + Sync + 'static,
{
    /// Build and run the outgoing-edge query shared by `list_associations` and
    /// `list_associations_with_properties`.
    ///
    /// The `target` ident restricts the far endpoint: its label always filters,
    /// and when it carries a concrete reference (a name or uuid) it is resolved to
    /// a `target_id` so a query addressed at one specific target does not fan out
    /// to every object of the same label (e.g. selecting one tag among several).
    async fn edge_query(
        &self,
        resource: &ResourceIdent,
        label: &AssociationLabel,
        target: Option<&ResourceIdent>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<olai_store::Association<ObjectLabel>>, Option<String>)> {
        let resource_id = self.resolve_ident(resource).await?;
        let mut query = EdgeQuery::from(resource_id, label.as_ref()).page(max_results, page_token);
        if let Some(target) = target {
            query = query.target_label(*target.label());
            // A concrete reference narrows to that one target; an `Undefined`
            // reference is a label-only filter.
            let reference: &ResourceRef = target.as_ref();
            if !matches!(reference, ResourceRef::Undefined) {
                let target_id = self.resolve_ident(target).await?;
                query = query.target_id(target_id);
            }
        }
        Ok(self.store.query_edges(query).await?)
    }
}

#[async_trait::async_trait]
impl<T: ResourceStoreReader> ResourceStoreReader for Arc<T> {
    async fn get(&self, id: &ResourceIdent) -> Result<(Resource, ResourceRef)> {
        T::get(self, id).await
    }

    async fn get_many(&self, ids: &[ResourceIdent]) -> Result<Vec<(Resource, ResourceRef)>> {
        T::get_many(self, ids).await
    }

    async fn get_with_secrets(&self, id: &ResourceIdent) -> Result<(Resource, ResourceRef)> {
        T::get_with_secrets(self, id).await
    }

    async fn get_versioned(&self, id: &ResourceIdent) -> Result<(Resource, ResourceRef, u64)> {
        T::get_versioned(self, id).await
    }

    async fn list(
        &self,
        label: &ObjectLabel,
        namespace: Option<&ResourceName>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<Resource>, Option<String>)> {
        T::list(self, label, namespace, max_results, page_token).await
    }
}

#[async_trait::async_trait]
impl<T: ResourceStore> ResourceStore for Arc<T> {
    async fn create_versioned(&self, resource: Resource) -> Result<(Resource, ResourceRef, u64)> {
        T::create_versioned(self, resource).await
    }

    async fn delete(&self, id: &ResourceIdent) -> Result<()> {
        T::delete(self, id).await
    }

    async fn replace_atomically_versioned(
        &self,
        delete: &ResourceIdent,
        create: Resource,
    ) -> Result<(Resource, ResourceRef, u64)> {
        T::replace_atomically_versioned(self, delete, create).await
    }

    async fn update_checked(
        &self,
        id: &ResourceIdent,
        resource: Resource,
        precondition: Precondition,
    ) -> Result<(Resource, ResourceRef, u64)> {
        T::update_checked(self, id, resource, precondition).await
    }

    async fn rename(
        &self,
        id: &ResourceIdent,
        new_name: &ResourceName,
        precondition: Precondition,
    ) -> Result<(Resource, ResourceRef, u64)> {
        T::rename(self, id, new_name, precondition).await
    }

    async fn add_association(
        &self,
        from: &ResourceIdent,
        to: &ResourceIdent,
        label: &AssociationLabel,
        properties: Option<PropertyMap>,
    ) -> Result<()> {
        T::add_association(self, from, to, label, properties).await
    }

    async fn remove_association(
        &self,
        from: &ResourceIdent,
        to: &ResourceIdent,
        label: &AssociationLabel,
    ) -> Result<()> {
        T::remove_association(self, from, to, label).await
    }

    async fn list_associations(
        &self,
        resource: &ResourceIdent,
        label: &AssociationLabel,
        target_label: Option<&ResourceIdent>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<ResourceIdent>, Option<String>)> {
        T::list_associations(self, resource, label, target_label, max_results, page_token).await
    }

    async fn list_associations_with_properties(
        &self,
        resource: &ResourceIdent,
        label: &AssociationLabel,
        target_label: Option<&ResourceIdent>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<(ResourceIdent, Option<PropertyMap>)>, Option<String>)> {
        T::list_associations_with_properties(
            self,
            resource,
            label,
            target_label,
            max_results,
            page_token,
        )
        .await
    }
}

#[async_trait::async_trait]
impl<T: ProvidesResourceStore> ResourceStoreReader for T {
    async fn get(&self, id: &ResourceIdent) -> Result<(Resource, ResourceRef)> {
        self.store().get(id).await
    }

    async fn get_many(&self, ids: &[ResourceIdent]) -> Result<Vec<(Resource, ResourceRef)>> {
        self.store().get_many(ids).await
    }

    async fn get_with_secrets(&self, id: &ResourceIdent) -> Result<(Resource, ResourceRef)> {
        self.store().get_with_secrets(id).await
    }

    async fn get_versioned(&self, id: &ResourceIdent) -> Result<(Resource, ResourceRef, u64)> {
        self.store().get_versioned(id).await
    }

    async fn list(
        &self,
        label: &ObjectLabel,
        namespace: Option<&ResourceName>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<Resource>, Option<String>)> {
        self.store()
            .list(label, namespace, max_results, page_token)
            .await
    }
}

#[async_trait::async_trait]
impl<T: ProvidesResourceStore> ResourceStore for T {
    async fn create_versioned(&self, resource: Resource) -> Result<(Resource, ResourceRef, u64)> {
        self.store().create_versioned(resource).await
    }

    async fn delete(&self, id: &ResourceIdent) -> Result<()> {
        self.store().delete(id).await
    }

    async fn replace_atomically_versioned(
        &self,
        delete: &ResourceIdent,
        create: Resource,
    ) -> Result<(Resource, ResourceRef, u64)> {
        self.store()
            .replace_atomically_versioned(delete, create)
            .await
    }

    async fn update_checked(
        &self,
        id: &ResourceIdent,
        resource: Resource,
        precondition: Precondition,
    ) -> Result<(Resource, ResourceRef, u64)> {
        self.store()
            .update_checked(id, resource, precondition)
            .await
    }

    async fn rename(
        &self,
        id: &ResourceIdent,
        new_name: &ResourceName,
        precondition: Precondition,
    ) -> Result<(Resource, ResourceRef, u64)> {
        self.store().rename(id, new_name, precondition).await
    }

    async fn add_association(
        &self,
        from: &ResourceIdent,
        to: &ResourceIdent,
        label: &AssociationLabel,
        properties: Option<PropertyMap>,
    ) -> Result<()> {
        self.store()
            .add_association(from, to, label, properties)
            .await
    }

    async fn remove_association(
        &self,
        from: &ResourceIdent,
        to: &ResourceIdent,
        label: &AssociationLabel,
    ) -> Result<()> {
        self.store().remove_association(from, to, label).await
    }

    async fn list_associations(
        &self,
        resource: &ResourceIdent,
        label: &AssociationLabel,
        target_label: Option<&ResourceIdent>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<ResourceIdent>, Option<String>)> {
        self.store()
            .list_associations(resource, label, target_label, max_results, page_token)
            .await
    }

    async fn list_associations_with_properties(
        &self,
        resource: &ResourceIdent,
        label: &AssociationLabel,
        target_label: Option<&ResourceIdent>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<(ResourceIdent, Option<PropertyMap>)>, Option<String>)> {
        self.store()
            .list_associations_with_properties(
                resource,
                label,
                target_label,
                max_results,
                page_token,
            )
            .await
    }
}
