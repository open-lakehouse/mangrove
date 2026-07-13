use itertools::Itertools;
use std::collections::HashMap;

use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::shares::v1::*;
use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};

use super::{RequestContext, SecuredAction};
pub use crate::codegen::shares::ShareHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::store::ResourceStore;
use crate::{Error, Result};

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext>> ShareHandler<RequestContext> for T {
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_share(
        &self,
        request: CreateShareRequest,
        context: RequestContext,
    ) -> Result<Share> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let resource = Share {
            name: request.name,
            comment: request.comment,
            ..Default::default()
        };
        // TODO:
        // - update the share with the current actor as owner
        // - create updated_* relations
        Ok(self.create(resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_share(
        &self,
        request: DeleteShareRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.delete(&request.resource()).await?)
    }

    #[tracing::instrument(skip(self, context))]
    async fn list_shares(
        &self,
        request: ListSharesRequest,
        context: RequestContext,
    ) -> Result<ListSharesResponse> {
        self.check_required(&request, &context).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::Share,
                None,
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        Ok(ListSharesResponse {
            shares: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_share(&self, request: GetShareRequest, context: RequestContext) -> Result<Share> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_share(
        &self,
        request: UpdateShareRequest,
        context: RequestContext,
    ) -> Result<Share> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let ident = request.resource();
        let current: Share = self.get(&ident).await?.0.try_into()?;

        // update the data_objects according to the actions defined in request
        let mut data_objects: HashMap<String, DataObject> = current
            .objects
            .into_iter()
            .map(|d| (d.name.clone(), d))
            .collect();
        for update in request.updates.iter() {
            match update.action.as_known().unwrap_or_default() {
                Action::ADD => {
                    if let Some(obj) = update.data_object.as_option() {
                        if data_objects.contains_key(&obj.name) {
                            return Err(Error::AlreadyExists);
                        }
                        data_objects.insert(obj.name.clone(), obj.clone());
                    }
                }
                Action::REMOVE => {
                    if let Some(obj) = update.data_object.as_option() {
                        data_objects.remove(&obj.name);
                    }
                }
                Action::UPDATE => {
                    if let Some(obj) = update.data_object.as_option() {
                        if let Some(existing) = data_objects.get_mut(&obj.name) {
                            *existing = obj.clone();
                        } else {
                            return Err(Error::NotFound);
                        }
                    }
                }
                Action::ACTION_UNSPECIFIED => {
                    return Err(Error::InvalidArgument("Unspecified action".to_string()));
                }
            }
        }

        let resource = Share {
            name: request.new_name.unwrap_or_else(|| request.name.clone()),
            comment: request.comment.or(current.comment),
            owner: request.owner.or(current.owner),
            objects: data_objects.into_values().collect(),
            ..Default::default()
        };
        // TODO:
        // - add update_* relations
        Ok(self.update(&ident, resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_permissions(
        &self,
        request: GetPermissionsRequest,
        context: RequestContext,
    ) -> Result<GetPermissionsResponse> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        // Permissions for a share are modelled as associations between the share and
        // principal-named resources.  The `AssociationLabel` enum currently does not
        // include a dedicated `SharedWith` / `GrantedTo` variant, so we cannot store
        // per-principal privilege lists in the association graph today.
        //
        // Once a suitable `AssociationLabel` variant is added (e.g. `GrantedTo` /
        // `GrantedBy`) the implementation should:
        //   1. Call `self.list_associations(&share_ident, &AssociationLabel::GrantedTo, …)`
        //   2. For each returned ResourceIdent, read the stored privilege list from the
        //      association properties and build a `PrivilegeAssignment`.
        //
        // For now we return an empty list so that callers receive a valid (non-panicking)
        // response and the endpoint is reachable.
        let _ = self.get(&request.resource()).await?; // verify the share exists
        Ok(GetPermissionsResponse {
            privilege_assignments: vec![],
            next_page_token: None,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_permissions(
        &self,
        request: UpdatePermissionsRequest,
        context: RequestContext,
    ) -> Result<UpdatePermissionsResponse> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        // See the note in `get_permissions`: a dedicated `AssociationLabel` variant is
        // needed before per-principal privilege changes can be persisted in the graph.
        //
        // Once that label exists, the implementation should iterate `request.changes` and:
        //   - For adds:    `self.add_association(&share_ident, &principal_ident, &label, props)`
        //   - For removes: `self.remove_association(&share_ident, &principal_ident, &label)`
        //
        // For now we verify the share exists, acknowledge the request, and return the
        // current (empty) privilege list unless the caller asked to omit it.
        let _ = self.get(&request.resource()).await?; // verify the share exists
        // When omit_permissions_list is true the caller does not want the updated list back.
        // Both cases currently return an empty list; once the association label is added the
        // non-omit path should call get_permissions to build the real list.
        Ok(UpdatePermissionsResponse {
            privilege_assignments: vec![],
            ..Default::default()
        })
    }
}

impl SecuredAction for CreateShareRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListSharesRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetShareRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateShareRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteShareRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for GetPermissionsRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdatePermissionsRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}
