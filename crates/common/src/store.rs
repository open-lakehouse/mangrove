use std::sync::Arc;

use itertools::Itertools;
use olai_store::store::{StoreExec, Transactional};
use olai_store::{
    AssociationStore, EdgeQuery, ObjectStore, ObjectStoreReader, Precondition, SecretObjectReader,
};
use uuid::Uuid;

use crate::models::{AssociationLabel, ObjectLabel, PropertyMap, Resource};
use crate::{Object, ResourceIdent, ResourceName, ResourceRef, Result};

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
    async fn create(&self, resource: Resource) -> Result<(Resource, ResourceRef)>;

    /// Delete a resource and all connected associations by its identifier.
    ///
    /// The implementing store should delete all associations of the resource
    /// before deleting the resource itself.
    ///
    /// ## Arguments
    /// - `id`: The identifier of the resource to delete.
    async fn delete(&self, id: &ResourceIdent) -> Result<()>;

    /// Atomically delete one resource and create another.
    ///
    /// Used by the managed-table commit flow, where a `StagingTable` reservation
    /// must be replaced by a `Table` **at the same uuid** (the id is fixed at
    /// staging time and the Delta API protocol depends on it). Because the object
    /// store keys objects by a single uuid, the delete and the create cannot both
    /// exist at that id: they must land in one atomic unit of work so a crash
    /// between them cannot leave the reservation gone without the table, and
    /// concurrent readers see either the old row or the new one, never neither.
    ///
    /// ## Arguments
    /// - `delete`: The identifier of the resource to remove.
    /// - `create`: The resource to create (carrying the adopted id).
    ///
    /// ## Returns
    /// The created resource and its reference.
    ///
    /// The default implementation runs the two operations separately (delete then
    /// create) so non-transactional stores still function; stores backed by a
    /// transactional object store override it to run both in one transaction.
    async fn replace_atomically(
        &self,
        delete: &ResourceIdent,
        create: Resource,
    ) -> Result<(Resource, ResourceRef)> {
        self.delete(delete).await?;
        self.create(create).await
    }

    /// Update a resource.
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
    ) -> Result<(Resource, ResourceRef)>;

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

/// Provides access to the generic, untyped [`ObjectStore`] for code that wants
/// to work at the `Object<ObjectLabel>` level rather than the typed `Resource` level.
pub trait ProvidesObjectStore: Send + Sync + 'static {
    fn object_store(&self) -> &dyn olai_store::ObjectStore<ObjectLabel>;
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
    async fn create(&self, resource: Resource) -> Result<(Resource, ResourceRef)> {
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
        Ok((created.try_into()?, id))
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
    async fn replace_atomically(
        &self,
        delete: &ResourceIdent,
        create: Resource,
    ) -> Result<(Resource, ResourceRef)> {
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
        Ok((created.try_into()?, id))
    }

    async fn update(
        &self,
        id: &ResourceIdent,
        resource: Resource,
    ) -> Result<(Resource, ResourceRef)> {
        let uuid = self.resolve_ident(id).await?;
        let object: Object = resource.try_into()?;
        // `ResourceStore::update` carries no etag, so this is an unconditional
        // overwrite (`Precondition::Any`); optimistic concurrency at this layer
        // is a follow-up. Secrets are not threaded here (see `create`).
        let updated = self
            .store
            .update(&uuid, object.properties, Precondition::Any, None)
            .await?;
        Ok((updated.try_into()?, uuid.into()))
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
    async fn create(&self, resource: Resource) -> Result<(Resource, ResourceRef)> {
        T::create(self, resource).await
    }

    async fn delete(&self, id: &ResourceIdent) -> Result<()> {
        T::delete(self, id).await
    }

    async fn replace_atomically(
        &self,
        delete: &ResourceIdent,
        create: Resource,
    ) -> Result<(Resource, ResourceRef)> {
        T::replace_atomically(self, delete, create).await
    }

    async fn update(
        &self,
        id: &ResourceIdent,
        resource: Resource,
    ) -> Result<(Resource, ResourceRef)> {
        T::update(self, id, resource).await
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
    async fn create(&self, resource: Resource) -> Result<(Resource, ResourceRef)> {
        self.store().create(resource).await
    }

    async fn delete(&self, id: &ResourceIdent) -> Result<()> {
        self.store().delete(id).await
    }

    async fn replace_atomically(
        &self,
        delete: &ResourceIdent,
        create: Resource,
    ) -> Result<(Resource, ResourceRef)> {
        self.store().replace_atomically(delete, create).await
    }

    async fn update(
        &self,
        id: &ResourceIdent,
        resource: Resource,
    ) -> Result<(Resource, ResourceRef)> {
        self.store().update(id, resource).await
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
