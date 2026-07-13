use bytes::Bytes;
use itertools::Itertools;

use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::shares::v1::{
    DataObjectType, GetShareRequest as SharesGetShareRequest, Share,
};
use unitycatalog_sharing_client::models::open_sharing::v1::{Share as SharingShare, *};

use crate::Result;
use crate::api::{RequestContext, SecuredAction, ShareHandler};
use crate::policy::{Permission, Policy, process_resources};
pub use crate::sharing::SharingHandler;
use crate::store::ResourceStore;

#[async_trait::async_trait]
pub trait SharingQueryHandler<Cx = RequestContext>: Send + Sync + 'static {
    async fn get_table_version(
        &self,
        request: GetTableVersionRequest,
        context: Cx,
    ) -> Result<GetTableVersionResponse>;

    async fn get_table_metadata(
        &self,
        request: GetTableMetadataRequest,
        context: Cx,
    ) -> Result<Bytes>;

    async fn query_table(&self, request: QueryTableRequest, context: Cx) -> Result<Bytes>;
}

#[async_trait::async_trait]
impl<T, Cx> SharingHandler<Cx> for T
where
    T: ResourceStore + Policy<Cx> + ShareHandler<Cx>,
    Cx: Clone + Send + Sync + 'static,
{
    async fn list_shares(
        &self,
        request: ListSharesRequest,
        context: Cx,
    ) -> Result<ListSharesResponse> {
        self.check_required(&request, &context).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::Share,
                None,
                request.max_results.map(|v| v as usize),
                request.page_token.clone(),
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;

        // if all resources gor filtered, but there are more pages, try again
        if resources.is_empty() && next_page_token.is_some() {
            return SharingHandler::list_shares(self, request, context).await;
        }

        let shares: Vec<Share> = resources.into_iter().map(|r| r.try_into()).try_collect()?;

        Ok(ListSharesResponse {
            items: shares
                .into_iter()
                .map(|r| SharingShare {
                    name: r.name,
                    id: r.id,
                    comment: r.comment,
                    ..Default::default()
                })
                .collect(),
            next_page_token,
            ..Default::default()
        })
    }

    async fn get_share(&self, request: GetShareRequest, context: Cx) -> Result<SharingShare> {
        self.check_required(&request, &context).await?;
        let shares_request = SharesGetShareRequest {
            name: request.name,
            include_shared_data: Some(false),
            ..Default::default()
        };
        let share: Share = self.get(&shares_request.resource()).await?.0.try_into()?;
        Ok(SharingShare {
            name: share.name,
            id: share.id,
            comment: share.comment,
            ..Default::default()
        })
    }

    async fn list_schemas(
        &self,
        request: ListSchemasRequest,
        context: Cx,
    ) -> Result<ListSchemasResponse> {
        self.check_required(&request, &context).await?;
        let shares_request = SharesGetShareRequest {
            name: request.share,
            include_shared_data: Some(true),
            ..Default::default()
        };
        let share: Share = self.get(&shares_request.resource()).await?.0.try_into()?;
        Ok(ListSchemasResponse {
            items: share
                .objects
                .into_iter()
                .filter_map(|a| {
                    if a.data_object_type == DataObjectType::Table {
                        Some(Schema {
                            name: a
                                .shared_as
                                .as_deref()
                                .unwrap_or_default()
                                .split_once(".")?
                                .0
                                .to_string(),
                            share: share.name.clone(),
                            ..Default::default()
                        })
                    } else {
                        None
                    }
                })
                .dedup()
                .collect(),
            next_page_token: None,
            ..Default::default()
        })
    }

    async fn list_tables(
        &self,
        request: ListTablesRequest,
        context: Cx,
    ) -> Result<ListTablesResponse> {
        self.check_required(&request, &context).await?;
        let shares_request = SharesGetShareRequest {
            name: request.share,
            include_shared_data: Some(true),
            ..Default::default()
        };
        let share: Share = self.get(&shares_request.resource()).await?.0.try_into()?;
        let items = share
            .objects
            .into_iter()
            .filter_map(|a| {
                if a.data_object_type == DataObjectType::Table {
                    let shared_as = a.shared_as.as_deref().unwrap_or_default();
                    let (schema, name) = shared_as.split_once(".")?;
                    if schema == request.name {
                        Some(Table {
                            name: name.to_string(),
                            share: share.name.clone(),
                            share_id: share.id.clone(),
                            schema: schema.to_string(),
                            ..Default::default()
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        Ok(ListTablesResponse {
            items,
            next_page_token: None,
            ..Default::default()
        })
    }

    async fn list_all_tables(
        &self,
        request: ListAllTablesRequest,
        context: Cx,
    ) -> Result<ListAllTablesResponse> {
        self.check_required(&request, &context).await?;
        let shares_request = SharesGetShareRequest {
            name: request.name,
            include_shared_data: Some(true),
            ..Default::default()
        };
        let share: Share = self.get(&shares_request.resource()).await?.0.try_into()?;
        let items = share
            .objects
            .into_iter()
            .filter_map(|a| {
                if a.data_object_type == DataObjectType::Table {
                    let shared_as = a.shared_as.as_deref().unwrap_or_default();
                    let (schema, name) = shared_as.split_once(".")?;
                    Some(Table {
                        name: name.to_string(),
                        share: share.name.clone(),
                        share_id: share.id.clone(),
                        schema: schema.to_string(),
                        ..Default::default()
                    })
                } else {
                    None
                }
            })
            .collect();
        Ok(ListAllTablesResponse {
            items,
            next_page_token: None,
            ..Default::default()
        })
    }
}
