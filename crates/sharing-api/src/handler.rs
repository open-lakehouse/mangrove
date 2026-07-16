//! The sharing handler surface: the generated discovery/asset handler traits are
//! implemented here as a **blanket impl** over any [`SharingBackend`], and the
//! hand-written NDJSON query path is a separate [`SharingQueryHandler`] trait
//! (also blanket-implemented) whose streaming response contract the generated,
//! JSON-only handlers do not model.
//!
//! All sharing business logic lives here, expressed purely in terms of the
//! [`SharingBackend`] port — a server serves the identical surface by implementing
//! the port and nothing else.

use bytes::Bytes;

use unitycatalog_sharing_client::models::open_sharing::v1::{
    Schema, Share as SharingShare, SharingSkill, SharingVolume, Table, *,
};
use unitycatalog_sharing_client::models::{
    MetadataResponse, MetadataResponseData, ProtocolResponseData,
};

use crate::backend::{
    SharedAssetKind, SharingAction, SharingBackend, SharingTableReference, SharingVolumeReference,
};
use crate::codegen::sharing::SharingHandler;
use crate::codegen::sharing_skill::SharingSkillHandler;
use crate::codegen::sharing_volume::SharingVolumeHandler;
use crate::error::{Error, Result};

// ---------------------------------------------------------------------------
// Discovery: the generated `SharingHandler`, blanket-implemented over the port.
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl<B, Cx> SharingHandler<Cx> for B
where
    B: SharingBackend<Cx>,
    Cx: Clone + Send + Sync + 'static,
{
    async fn list_shares(
        &self,
        request: ListSharesRequest,
        context: Cx,
    ) -> Result<ListSharesResponse> {
        self.authorize(SharingAction::ListShares, &context).await?;
        let (shares, next_page_token) = self
            .list_shares(
                request.max_results.map(|v| v as usize),
                request.page_token.clone(),
                &context,
            )
            .await?;
        Ok(ListSharesResponse {
            items: shares.into_iter().map(SharingShare::from).collect(),
            next_page_token,
            ..Default::default()
        })
    }

    async fn get_share(&self, request: GetShareRequest, context: Cx) -> Result<SharingShare> {
        self.authorize(
            SharingAction::ReadShare {
                share: &request.name,
            },
            &context,
        )
        .await?;
        let share = SharingBackend::get_share(self, &request.name, &context).await?;
        Ok(share.into())
    }

    async fn list_schemas(
        &self,
        request: ListSchemasRequest,
        context: Cx,
    ) -> Result<ListSchemasResponse> {
        self.authorize(
            SharingAction::ReadShare {
                share: &request.share,
            },
            &context,
        )
        .await?;
        let objects = self
            .list_share_objects(
                &request.share,
                crate::backend::ShareObjectKind::Table,
                &context,
            )
            .await?;
        let mut seen = std::collections::HashSet::new();
        let items = objects
            .into_iter()
            .filter(|o| seen.insert(o.schema.clone()))
            .map(|o| Schema {
                name: o.schema,
                share: request.share.clone(),
                ..Default::default()
            })
            .collect();
        Ok(ListSchemasResponse {
            items,
            next_page_token: None,
            ..Default::default()
        })
    }

    async fn list_tables(
        &self,
        request: ListTablesRequest,
        context: Cx,
    ) -> Result<ListTablesResponse> {
        self.authorize(
            SharingAction::ReadShare {
                share: &request.share,
            },
            &context,
        )
        .await?;
        let share = SharingBackend::get_share(self, &request.share, &context).await?;
        let objects = self
            .list_share_objects(
                &request.share,
                crate::backend::ShareObjectKind::Table,
                &context,
            )
            .await?;
        let items = objects
            .into_iter()
            .filter(|o| o.schema == request.name)
            .map(|o| Table {
                name: o.name,
                share: share.name.clone(),
                share_id: share.id.clone(),
                schema: o.schema,
                ..Default::default()
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
        self.authorize(
            SharingAction::ReadShare {
                share: &request.name,
            },
            &context,
        )
        .await?;
        let share = SharingBackend::get_share(self, &request.name, &context).await?;
        let objects = self
            .list_share_objects(
                &request.name,
                crate::backend::ShareObjectKind::Table,
                &context,
            )
            .await?;
        let items = objects
            .into_iter()
            .map(|o| Table {
                name: o.name,
                share: share.name.clone(),
                share_id: share.id.clone(),
                schema: o.schema,
                ..Default::default()
            })
            .collect();
        Ok(ListAllTablesResponse {
            items,
            next_page_token: None,
            ..Default::default()
        })
    }
}

// ---------------------------------------------------------------------------
// Assets (volumes): the generated `SharingVolumeHandler`, blanket-implemented.
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl<B, Cx> SharingVolumeHandler<Cx> for B
where
    B: SharingBackend<Cx>,
    Cx: Clone + Send + Sync + 'static,
{
    async fn list_volumes(
        &self,
        request: ListVolumesRequest,
        context: Cx,
    ) -> Result<ListVolumesResponse> {
        self.authorize(
            SharingAction::ReadShare {
                share: &request.share,
            },
            &context,
        )
        .await?;
        let share = SharingBackend::get_share(self, &request.share, &context).await?;
        let objects = self
            .list_share_objects(
                &request.share,
                crate::backend::ShareObjectKind::Volume,
                &context,
            )
            .await?;
        let items = objects
            .into_iter()
            .filter(|o| o.schema == request.schema)
            .map(|o| SharingVolume {
                name: o.name,
                schema: o.schema,
                share: share.name.clone(),
                share_id: share.id.clone(),
                ..Default::default()
            })
            .collect();
        Ok(ListVolumesResponse {
            items,
            next_page_token: None,
            ..Default::default()
        })
    }

    async fn list_all_volumes(
        &self,
        request: ListAllVolumesRequest,
        context: Cx,
    ) -> Result<ListAllVolumesResponse> {
        self.authorize(
            SharingAction::ReadShare {
                share: &request.share,
            },
            &context,
        )
        .await?;
        let share = SharingBackend::get_share(self, &request.share, &context).await?;
        let objects = self
            .list_share_objects(
                &request.share,
                crate::backend::ShareObjectKind::Volume,
                &context,
            )
            .await?;
        let items = objects
            .into_iter()
            .map(|o| SharingVolume {
                name: o.name,
                schema: o.schema,
                share: share.name.clone(),
                share_id: share.id.clone(),
                ..Default::default()
            })
            .collect();
        Ok(ListAllVolumesResponse {
            items,
            next_page_token: None,
            ..Default::default()
        })
    }

    async fn get_volume(&self, request: GetVolumeRequest, context: Cx) -> Result<SharingVolume> {
        self.authorize(
            SharingAction::ReadShare {
                share: &request.share,
            },
            &context,
        )
        .await?;
        let asset = SharingVolumeReference::new(&request.share, &request.schema, &request.name);
        let location = self
            .resolve_asset_location(&asset, SharedAssetKind::Volume, &context)
            .await?;
        Ok(SharingVolume {
            name: request.name,
            schema: request.schema,
            share: request.share,
            storage_location: Some(location.raw),
            ..Default::default()
        })
    }

    async fn generate_temporary_volume_credentials(
        &self,
        request: GenerateTemporaryVolumeCredentialsRequest,
        context: Cx,
    ) -> Result<SharingTemporaryCredentials> {
        self.authorize(
            SharingAction::ReadShare {
                share: &request.share,
            },
            &context,
        )
        .await?;
        let asset = SharingVolumeReference::new(&request.share, &request.schema, &request.name);
        let location = self
            .resolve_asset_location(&asset, SharedAssetKind::Volume, &context)
            .await?;
        self.vend_read_credential(&location, &context).await
    }
}

// ---------------------------------------------------------------------------
// Assets (agent skills): the generated `SharingSkillHandler`, blanket-implemented.
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl<B, Cx> SharingSkillHandler<Cx> for B
where
    B: SharingBackend<Cx>,
    Cx: Clone + Send + Sync + 'static,
{
    async fn list_skills(
        &self,
        request: ListSkillsRequest,
        context: Cx,
    ) -> Result<ListSkillsResponse> {
        self.authorize(
            SharingAction::ReadShare {
                share: &request.share,
            },
            &context,
        )
        .await?;
        let share = SharingBackend::get_share(self, &request.share, &context).await?;
        let objects = self
            .list_share_objects(
                &request.share,
                crate::backend::ShareObjectKind::AgentSkill,
                &context,
            )
            .await?;
        let items = objects
            .into_iter()
            .filter(|o| o.schema == request.schema)
            .map(|o| SharingSkill {
                name: o.name,
                schema: o.schema,
                share: share.name.clone(),
                share_id: share.id.clone(),
                ..Default::default()
            })
            .collect();
        Ok(ListSkillsResponse {
            items,
            next_page_token: None,
            ..Default::default()
        })
    }

    async fn list_all_skills(
        &self,
        request: ListAllSkillsRequest,
        context: Cx,
    ) -> Result<ListAllSkillsResponse> {
        self.authorize(
            SharingAction::ReadShare {
                share: &request.share,
            },
            &context,
        )
        .await?;
        let share = SharingBackend::get_share(self, &request.share, &context).await?;
        let objects = self
            .list_share_objects(
                &request.share,
                crate::backend::ShareObjectKind::AgentSkill,
                &context,
            )
            .await?;
        let items = objects
            .into_iter()
            .map(|o| SharingSkill {
                name: o.name,
                schema: o.schema,
                share: share.name.clone(),
                share_id: share.id.clone(),
                ..Default::default()
            })
            .collect();
        Ok(ListAllSkillsResponse {
            items,
            next_page_token: None,
            ..Default::default()
        })
    }

    async fn get_skill(&self, request: GetSkillRequest, context: Cx) -> Result<SharingSkill> {
        self.authorize(
            SharingAction::ReadShare {
                share: &request.share,
            },
            &context,
        )
        .await?;
        let asset = SharingVolumeReference::new(&request.share, &request.schema, &request.name);
        let location = self
            .resolve_asset_location(&asset, SharedAssetKind::AgentSkill, &context)
            .await?;
        Ok(SharingSkill {
            name: request.name,
            schema: request.schema,
            share: request.share,
            storage_location: Some(location.raw),
            ..Default::default()
        })
    }

    async fn generate_temporary_skill_credentials(
        &self,
        request: GenerateTemporarySkillCredentialsRequest,
        context: Cx,
    ) -> Result<SharingTemporaryCredentials> {
        self.authorize(
            SharingAction::ReadShare {
                share: &request.share,
            },
            &context,
        )
        .await?;
        let asset = SharingVolumeReference::new(&request.share, &request.schema, &request.name);
        let location = self
            .resolve_asset_location(&asset, SharedAssetKind::AgentSkill, &context)
            .await?;
        self.vend_read_credential(&location, &context).await
    }
}

// ---------------------------------------------------------------------------
// The hand-written NDJSON query path (version / metadata / query), plus the
// not-yet-implemented protocol additions (CDF, async queries).
// ---------------------------------------------------------------------------

/// The hand-written table-query surface: `version` / `metadata` / `query` return
/// newline-delimited JSON (`application/x-ndjson`) with result-derived headers, a
/// streaming contract the generated JSON handlers do not model. The recently
/// added protocol endpoints — change data feed and asynchronous queries — have
/// their types defined but return [`Error::NotImplemented`].
#[async_trait::async_trait]
pub trait SharingQueryHandler<Cx = crate::DefaultContext>: Send + Sync + 'static {
    /// The current Delta version of a shared table.
    async fn get_table_version(
        &self,
        request: GetTableVersionRequest,
        context: Cx,
    ) -> Result<GetTableVersionResponse>;

    /// The NDJSON protocol + metadata actions for a shared table.
    async fn get_table_metadata(
        &self,
        request: GetTableMetadataRequest,
        context: Cx,
    ) -> Result<Bytes>;

    /// The NDJSON protocol + metadata + file actions for a shared table query.
    async fn query_table(&self, request: QueryTableRequest, context: Cx) -> Result<Bytes>;

    /// Change data feed for a shared table. **Not implemented** — the request type
    /// and route exist for spec coverage but the serving path returns 501.
    async fn get_table_changes(&self, request: QueryTableRequest, context: Cx) -> Result<Bytes>;

    /// Poll an asynchronous query by id. **Not implemented** — the route exists for
    /// spec coverage but the serving path returns 501.
    async fn poll_query(&self, query_id: String, context: Cx) -> Result<Bytes>;
}

#[async_trait::async_trait]
impl<B, Cx> SharingQueryHandler<Cx> for B
where
    B: SharingBackend<Cx>,
    Cx: Clone + Send + Sync + 'static,
{
    async fn get_table_version(
        &self,
        request: GetTableVersionRequest,
        context: Cx,
    ) -> Result<GetTableVersionResponse> {
        self.authorize(
            SharingAction::ReadShare {
                share: &request.share,
            },
            &context,
        )
        .await?;
        let table_ref = SharingTableReference::new(&request.share, &request.schema, &request.name);
        let location = self.resolve_table_location(&table_ref, &context).await?;
        let snapshot = self.kernel_session().read_snapshot(&location, None).await?;
        Ok(GetTableVersionResponse {
            version: snapshot.version() as i64,
            ..Default::default()
        })
    }

    async fn get_table_metadata(
        &self,
        request: GetTableMetadataRequest,
        context: Cx,
    ) -> Result<Bytes> {
        self.authorize(
            SharingAction::ReadShare {
                share: &request.share,
            },
            &context,
        )
        .await?;
        let table_ref = SharingTableReference::new(&request.share, &request.schema, &request.name);
        let location = self.resolve_table_location(&table_ref, &context).await?;
        let snapshot = self.kernel_session().read_snapshot(&location, None).await?;

        let table_config = snapshot.table_configuration();
        let mut response = serde_json::to_vec(&MetadataResponse::MetaData(
            MetadataResponseData::ParquetMetadata(table_config.metadata().try_into()?),
        ))?;
        response.push(b'\n');
        response.extend(serde_json::to_vec(&MetadataResponse::Protocol(
            ProtocolResponseData::ParquetProtocol(table_config.protocol().into()),
        ))?);

        Ok(Bytes::from(response))
    }

    async fn query_table(&self, request: QueryTableRequest, context: Cx) -> Result<Bytes> {
        self.authorize(
            SharingAction::ReadShare {
                share: &request.share,
            },
            &context,
        )
        .await?;
        let table_ref = SharingTableReference::new(&request.share, &request.schema, &request.name);
        let location = self.resolve_table_location(&table_ref, &context).await?;
        self.kernel_session()
            .extract_sharing_query_response(&table_ref, &location)
            .await
    }

    async fn get_table_changes(&self, _request: QueryTableRequest, _context: Cx) -> Result<Bytes> {
        Err(Error::not_implemented(
            "Delta Sharing change data feed (/changes) is not implemented",
        ))
    }

    async fn poll_query(&self, _query_id: String, _context: Cx) -> Result<Bytes> {
        Err(Error::not_implemented(
            "Delta Sharing asynchronous queries (POST /queries/{id}) are not implemented",
        ))
    }
}

/// A convenience alias for the full set of handler traits a sharing router needs.
///
/// Any [`SharingBackend`] satisfies this via the blanket impls above. Named so a
/// server (or the router) can bound on one trait instead of four.
pub trait SharingApiHandler<Cx = crate::DefaultContext>:
    SharingHandler<Cx> + SharingVolumeHandler<Cx> + SharingSkillHandler<Cx> + SharingQueryHandler<Cx>
{
}

impl<T, Cx> SharingApiHandler<Cx> for T where
    T: SharingHandler<Cx>
        + SharingVolumeHandler<Cx>
        + SharingSkillHandler<Cx>
        + SharingQueryHandler<Cx>
{
}
