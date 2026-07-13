use bytes::Bytes;

use unitycatalog_common::models::shares::v1::{
    DataObjectType, GetShareRequest as SharesGetShareRequest,
};
use unitycatalog_common::models::tables::v1::{DataSourceFormat, GetTableRequest, Table};
use unitycatalog_common::models::temporary_credentials::v1::{
    TemporaryCredential, temporary_credential::Credentials as UcCredentials,
};
use unitycatalog_common::models::volumes::v1::{GetVolumeRequest as UcGetVolumeRequest, Volume};
use unitycatalog_common::{ResourceIdent, ResourceName, Share};
use unitycatalog_sharing_client::models::open_sharing::v1::{
    sharing_temporary_credentials::Credentials as SharingCredentials, *,
};

use super::credential_vending::{VendOperation, vend_credential};
use super::object_store::find_external_location_for_url;
use super::{Policy, ServerHandler, StorageLocationUrl, TableManager};
use crate::api::credentials::CredentialHandlerExt;
use crate::api::sharing::{
    MetadataResponse, MetadataResponseData, ProtocolResponseData, SharingQueryHandler,
};
use crate::api::{RequestContext, SecuredAction};
use crate::error::{Error, Result};
use crate::sharing::{SharingSkillHandler, SharingVolumeHandler};
use crate::store::ResourceStoreReader;
use unitycatalog_common::models::credentials::v1::GetCredentialRequest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharingTableReference {
    share: String,
    schema: String,
    table: String,
}

impl SharingTableReference {
    pub(super) fn system_table_name(&self) -> String {
        format!("{}__{}__{}", self.share, self.schema, self.table)
    }
}

/// A reference to a shared storage-backed asset (volume or agent skill) within
/// a share/schema. Both asset kinds resolve through the backing Volume
/// primitive, so they share one reference type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharingVolumeReference {
    pub share: String,
    pub schema: String,
    pub name: String,
}

impl ServerHandler<RequestContext> {
    /// Resolve the storage location of a shared table.
    ///
    /// The Share itself is always read from the local store (shares are a
    /// sharing-server-owned primitive). The backing Table primitive is resolved
    /// through the configured [`table_source`](ServerHandler::table_source) when
    /// present — so in the side-by-side topology it is fetched from the upstream
    /// Unity Catalog rather than the local store — and falls back to a local
    /// store lookup otherwise.
    pub(super) async fn resolve_table_location(
        &self,
        table_ref: &SharingTableReference,
        context: &RequestContext,
    ) -> Result<StorageLocationUrl> {
        let share_ident = ResourceIdent::share(ResourceName::new([table_ref.share.as_str()]));
        let share_info: Share = self.get(&share_ident).await?.0.try_into()?;
        let Some(table_object) = share_info.objects.iter().find(|o| {
            o.shared_as.as_deref().unwrap_or_default()
                == format!("{}.{}", table_ref.schema, table_ref.table)
        }) else {
            return Err(Error::NotFound);
        };

        let table_info: Table = if let Some(table_source) = self.table_source() {
            // Side-by-side topology: resolve the Table primitive through the
            // routed handler (e.g. upstream Unity Catalog), keyed by full name.
            let request = GetTableRequest {
                full_name: table_object.name.clone(),
                ..Default::default()
            };
            table_source.get_table(request, context.clone()).await?
        } else {
            // Self-contained topology: the Table primitive lives in the local
            // store alongside the Share.
            let table_ident = ResourceIdent::table(ResourceName::new(table_object.name.split(".")));
            self.get(&table_ident).await?.0.try_into()?
        };

        let location = table_info.storage_location.ok_or(Error::NotFound)?;
        Ok(StorageLocationUrl::parse(&location)?)
    }

    /// Resolve the storage location of a shared volume or agent skill.
    ///
    /// Mirrors [`resolve_table_location`](Self::resolve_table_location): the
    /// Share is read from the local store, and the backing Volume primitive is
    /// resolved through the configured
    /// [`volume_source`](ServerHandler::volume_source) when present (side-by-side
    /// topology, e.g. an upstream Unity Catalog), falling back to a local store
    /// lookup otherwise. The matching share object must be a `VOLUME` or
    /// `AGENT_SKILL` (agent skills are backed by a volume's storage).
    pub(super) async fn resolve_volume_location(
        &self,
        volume_ref: &SharingVolumeReference,
        context: &RequestContext,
    ) -> Result<StorageLocationUrl> {
        let share_ident = ResourceIdent::share(ResourceName::new([volume_ref.share.as_str()]));
        let share_info: Share = self.get(&share_ident).await?.0.try_into()?;
        let shared_as = format!("{}.{}", volume_ref.schema, volume_ref.name);
        let Some(object) = share_info.objects.iter().find(|o| {
            o.shared_as.as_deref().unwrap_or_default() == shared_as
                && (o.data_object_type == DataObjectType::Volume
                    || o.data_object_type == DataObjectType::AgentSkill)
        }) else {
            return Err(Error::NotFound);
        };

        let volume_info: Volume = if let Some(volume_source) = self.volume_source() {
            // Side-by-side topology: resolve the Volume primitive through the
            // routed handler (e.g. upstream Unity Catalog), keyed by full name.
            let request = UcGetVolumeRequest {
                name: object.name.clone(),
                ..Default::default()
            };
            volume_source.get_volume(request, context.clone()).await?
        } else {
            // Self-contained topology: the Volume primitive lives in the local
            // store alongside the Share.
            let volume_ident = ResourceIdent::volume(ResourceName::new(object.name.split(".")));
            self.get(&volume_ident).await?.0.try_into()?
        };

        Ok(StorageLocationUrl::parse(&volume_info.storage_location)?)
    }
}

#[async_trait::async_trait]
impl SharingQueryHandler for ServerHandler<RequestContext> {
    async fn get_table_version(
        &self,
        request: GetTableVersionRequest,
        context: RequestContext,
    ) -> Result<GetTableVersionResponse> {
        self.check_required(&request, &context).await?;
        let table_ref = SharingTableReference {
            share: request.share,
            schema: request.schema,
            table: request.name,
        };
        let location = self.resolve_table_location(&table_ref, &context).await?;
        let snapshot = self
            .read_snapshot(&location, &DataSourceFormat::Delta, None)
            .await?;
        Ok(GetTableVersionResponse {
            version: snapshot.version() as i64,
            ..Default::default()
        })
    }

    async fn get_table_metadata(
        &self,
        request: GetTableMetadataRequest,
        context: RequestContext,
    ) -> Result<Bytes> {
        self.check_required(&request, &context).await?;
        let table_ref = SharingTableReference {
            share: request.share,
            schema: request.schema,
            table: request.name,
        };
        let location = self.resolve_table_location(&table_ref, &context).await?;
        let snapshot = self
            .read_snapshot(&location, &DataSourceFormat::Delta, None)
            .await?;

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

    async fn query_table(
        &self,
        request: QueryTableRequest,
        context: RequestContext,
    ) -> Result<Bytes> {
        self.check_required(&request, &context).await?;
        let table_ref = SharingTableReference {
            share: request.share,
            schema: request.schema,
            table: request.name,
        };
        let location = self.resolve_table_location(&table_ref, &context).await?;
        let data = self
            .session
            .extract_sharing_query_response(&table_ref, &location)
            .await?;
        Ok(data)
    }
}

/// Map a Unity Catalog [`TemporaryCredential`] to the Open Sharing
/// [`SharingTemporaryCredentials`] envelope. The two carry the same
/// provider-specific payloads; only the message names differ.
fn to_sharing_credentials(cred: TemporaryCredential) -> Result<SharingTemporaryCredentials> {
    let credentials = match cred.credentials {
        Some(UcCredentials::AwsTempCredentials(c)) => Some(SharingCredentials::AwsTempCredentials(
            Box::new(SharingAwsCredentials {
                access_key_id: c.access_key_id,
                secret_access_key: c.secret_access_key,
                session_token: c.session_token,
                ..Default::default()
            }),
        )),
        Some(UcCredentials::AzureUserDelegationSas(c)) => Some(
            SharingCredentials::AzureUserDelegationSas(Box::new(SharingAzureUserDelegationSas {
                sas_token: c.sas_token,
                ..Default::default()
            })),
        ),
        Some(UcCredentials::GcpOauthToken(c)) => Some(SharingCredentials::GcpOauthToken(Box::new(
            SharingGcpOauthToken {
                oauth_token: c.oauth_token,
                ..Default::default()
            },
        ))),
        Some(UcCredentials::R2TempCredentials(c)) => Some(SharingCredentials::R2Credentials(
            Box::new(SharingR2Credentials {
                access_key_id: c.access_key_id,
                secret_access_key: c.secret_access_key,
                session_token: c.session_token,
                ..Default::default()
            }),
        )),
        // Azure AD tokens have no Open Sharing equivalent; treat as unvendable.
        Some(UcCredentials::AzureAad(_)) | None => {
            return Err(Error::generic(
                "vended credential type is not supported by Open Sharing",
            ));
        }
    };
    Ok(SharingTemporaryCredentials {
        expiration_time: cred.expiration_time,
        url: Some(cred.url),
        credentials,
        ..Default::default()
    })
}

impl ServerHandler<RequestContext> {
    /// Load a share's objects of a given [`DataObjectType`], returning each
    /// object's `(schema, name)` derived from its `shared_as` name.
    async fn shared_assets(
        &self,
        share: &str,
        kind: DataObjectType,
    ) -> Result<(String, Option<String>, Vec<(String, String)>)> {
        let request = SharesGetShareRequest {
            name: share.to_string(),
            include_shared_data: Some(true),
            ..Default::default()
        };
        let share_info: Share = self.get(&request.resource()).await?.0.try_into()?;
        let items = share_info
            .objects
            .iter()
            .filter(|o| o.data_object_type == kind)
            .filter_map(|o| {
                let (schema, name) = o.shared_as.as_deref().unwrap_or_default().split_once('.')?;
                Some((schema.to_string(), name.to_string()))
            })
            .collect();
        Ok((share_info.name, share_info.id, items))
    }

    /// Vend Open Sharing credentials for an already-resolved storage location.
    async fn vend_sharing_credentials(
        &self,
        location: &StorageLocationUrl,
    ) -> Result<SharingTemporaryCredentials> {
        let ext_loc = find_external_location_for_url(location, self).await?;
        let credential = self
            .get_credential_internal(GetCredentialRequest {
                name: ext_loc.credential_name.clone(),
                ..Default::default()
            })
            .await?;
        // Open Sharing only grants read access to shared assets.
        let cred =
            vend_credential(&credential, location.raw().as_str(), VendOperation::Read).await?;
        to_sharing_credentials(cred)
    }
}

#[async_trait::async_trait]
impl SharingVolumeHandler for ServerHandler<RequestContext> {
    async fn list_volumes(
        &self,
        request: ListVolumesRequest,
        context: RequestContext,
    ) -> Result<ListVolumesResponse> {
        self.check_required(&request, &context).await?;
        let (share, share_id, assets) = self
            .shared_assets(&request.share, DataObjectType::Volume)
            .await?;
        let items = assets
            .into_iter()
            .filter(|(schema, _)| schema == &request.schema)
            .map(|(schema, name)| SharingVolume {
                name,
                schema,
                share: share.clone(),
                share_id: share_id.clone(),
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
        context: RequestContext,
    ) -> Result<ListAllVolumesResponse> {
        self.check_required(&request, &context).await?;
        let (share, share_id, assets) = self
            .shared_assets(&request.share, DataObjectType::Volume)
            .await?;
        let items = assets
            .into_iter()
            .map(|(schema, name)| SharingVolume {
                name,
                schema,
                share: share.clone(),
                share_id: share_id.clone(),
                ..Default::default()
            })
            .collect();
        Ok(ListAllVolumesResponse {
            items,
            next_page_token: None,
            ..Default::default()
        })
    }

    async fn get_volume(
        &self,
        request: GetVolumeRequest,
        context: RequestContext,
    ) -> Result<SharingVolume> {
        self.check_required(&request, &context).await?;
        let volume_ref = SharingVolumeReference {
            share: request.share.clone(),
            schema: request.schema.clone(),
            name: request.name.clone(),
        };
        let location = self.resolve_volume_location(&volume_ref, &context).await?;
        Ok(SharingVolume {
            name: request.name,
            schema: request.schema,
            share: request.share,
            storage_location: Some(location.raw().as_str().to_string()),
            ..Default::default()
        })
    }

    async fn generate_temporary_volume_credentials(
        &self,
        request: GenerateTemporaryVolumeCredentialsRequest,
        context: RequestContext,
    ) -> Result<SharingTemporaryCredentials> {
        self.check_required(&request, &context).await?;
        let volume_ref = SharingVolumeReference {
            share: request.share,
            schema: request.schema,
            name: request.name,
        };
        let location = self.resolve_volume_location(&volume_ref, &context).await?;
        self.vend_sharing_credentials(&location).await
    }
}

#[async_trait::async_trait]
impl SharingSkillHandler for ServerHandler<RequestContext> {
    async fn list_skills(
        &self,
        request: ListSkillsRequest,
        context: RequestContext,
    ) -> Result<ListSkillsResponse> {
        self.check_required(&request, &context).await?;
        let (share, share_id, assets) = self
            .shared_assets(&request.share, DataObjectType::AgentSkill)
            .await?;
        let items = assets
            .into_iter()
            .filter(|(schema, _)| schema == &request.schema)
            .map(|(schema, name)| SharingSkill {
                name,
                schema,
                share: share.clone(),
                share_id: share_id.clone(),
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
        context: RequestContext,
    ) -> Result<ListAllSkillsResponse> {
        self.check_required(&request, &context).await?;
        let (share, share_id, assets) = self
            .shared_assets(&request.share, DataObjectType::AgentSkill)
            .await?;
        let items = assets
            .into_iter()
            .map(|(schema, name)| SharingSkill {
                name,
                schema,
                share: share.clone(),
                share_id: share_id.clone(),
                ..Default::default()
            })
            .collect();
        Ok(ListAllSkillsResponse {
            items,
            next_page_token: None,
            ..Default::default()
        })
    }

    async fn get_skill(
        &self,
        request: GetSkillRequest,
        context: RequestContext,
    ) -> Result<SharingSkill> {
        self.check_required(&request, &context).await?;
        let skill_ref = SharingVolumeReference {
            share: request.share.clone(),
            schema: request.schema.clone(),
            name: request.name.clone(),
        };
        let location = self.resolve_volume_location(&skill_ref, &context).await?;
        Ok(SharingSkill {
            name: request.name,
            schema: request.schema,
            share: request.share,
            storage_location: Some(location.raw().as_str().to_string()),
            ..Default::default()
        })
    }

    async fn generate_temporary_skill_credentials(
        &self,
        request: GenerateTemporarySkillCredentialsRequest,
        context: RequestContext,
    ) -> Result<SharingTemporaryCredentials> {
        self.check_required(&request, &context).await?;
        let skill_ref = SharingVolumeReference {
            share: request.share,
            schema: request.schema,
            name: request.name,
        };
        let location = self.resolve_volume_location(&skill_ref, &context).await?;
        self.vend_sharing_credentials(&location).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unitycatalog_common::models::temporary_credentials::v1::{
        AwsTemporaryCredentials, AzureAad, GcpOauthToken,
    };

    fn uc_cred(credentials: Option<UcCredentials>) -> TemporaryCredential {
        TemporaryCredential {
            expiration_time: 1_700_000_000_000,
            url: "s3://bucket/prefix".to_string(),
            credentials,
            ..Default::default()
        }
    }

    #[test]
    fn maps_aws_credentials_preserving_fields() {
        let cred = uc_cred(Some(UcCredentials::AwsTempCredentials(Box::new(
            AwsTemporaryCredentials {
                access_key_id: "AKIA".to_string(),
                secret_access_key: "secret".to_string(),
                session_token: "token".to_string(),
                access_point: String::new(),
                ..Default::default()
            },
        ))));
        let out = to_sharing_credentials(cred).unwrap();
        assert_eq!(out.expiration_time, 1_700_000_000_000);
        assert_eq!(out.url.as_deref(), Some("s3://bucket/prefix"));
        match out.credentials {
            Some(SharingCredentials::AwsTempCredentials(c)) => {
                assert_eq!(c.access_key_id, "AKIA");
                assert_eq!(c.secret_access_key, "secret");
                assert_eq!(c.session_token, "token");
            }
            other => panic!("expected AWS credentials, got {other:?}"),
        }
    }

    #[test]
    fn maps_gcp_credentials() {
        let cred = uc_cred(Some(UcCredentials::GcpOauthToken(Box::new(
            GcpOauthToken {
                oauth_token: "ya29".to_string(),
                ..Default::default()
            },
        ))));
        let out = to_sharing_credentials(cred).unwrap();
        assert!(matches!(
            out.credentials,
            Some(SharingCredentials::GcpOauthToken(_))
        ));
    }

    #[test]
    fn azure_aad_and_missing_credentials_are_unsupported() {
        assert!(to_sharing_credentials(uc_cred(None)).is_err());
        assert!(
            to_sharing_credentials(uc_cred(Some(UcCredentials::AzureAad(Box::new(AzureAad {
                aad_token: "aad".to_string(),
                ..Default::default()
            })))))
            .is_err()
        );
    }
}
