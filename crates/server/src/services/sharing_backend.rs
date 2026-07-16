//! The server's [`SharingBackend`] adapter.
//!
//! Implements the `olai-uc-sharing-api` port over the server's own share store,
//! table/volume resolution, credential vending, and authorization. Implementing
//! this one trait gives the server the full Delta Sharing / Open Sharing surface
//! (discovery, asset, and NDJSON query handlers) via the crate's blanket impls.
//!
//! All the sharing *semantics* live in the crate; this adapter is pure data
//! access plus the server-specific topology routing (self-contained vs
//! side-by-side) for resolving the backing Table / Volume primitives.

use async_trait::async_trait;
use itertools::Itertools;

use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::credentials::v1::GetCredentialRequest;
use unitycatalog_common::models::shares::v1::{
    DataObjectType, GetShareRequest as SharesGetShareRequest,
};
use unitycatalog_common::models::tables::v1::{GetTableRequest, Table};
use unitycatalog_common::models::temporary_credentials::v1::{
    TemporaryCredential, temporary_credential::Credentials as UcCredentials,
};
use unitycatalog_common::models::volumes::v1::{GetVolumeRequest as UcGetVolumeRequest, Volume};
use unitycatalog_common::{ResourceIdent, ResourceName, Share};
use unitycatalog_sharing_api::backend::{
    BackendResult, ResolvedLocation, ResolvedShare, ShareObject, ShareObjectKind, SharedAssetKind,
    SharingAction, SharingBackend, SharingTableReference, SharingVolumeReference,
};
use unitycatalog_sharing_api::error::Error as SharingError;
use unitycatalog_sharing_api::session::KernelSession;
use unitycatalog_sharing_client::models::open_sharing::v1::{
    GetShareRequest, ListSharesRequest, SharingAwsCredentials, SharingAzureUserDelegationSas,
    SharingGcpOauthToken, SharingR2Credentials, SharingTemporaryCredentials,
    sharing_temporary_credentials::Credentials as SharingCredentials,
};

use super::credential_vending::{VendOperation, vend_credential};
use super::object_store::find_external_location_for_url;
use super::{Policy, ServerHandler, StorageLocationUrl};
use crate::api::credentials::CredentialHandlerExt;
use crate::api::{RequestContext, SecuredAction};
use crate::policy::{Permission, process_resources};
use crate::store::ResourceStoreReader;

/// Map any server-internal error (or a store/common error that promotes into one)
/// into the sharing crate's error contract, preserving the caller-facing status.
fn to_sharing_err(e: impl Into<crate::Error>) -> SharingError {
    match e.into() {
        crate::Error::NotFound => SharingError::NotFound,
        crate::Error::NotAllowed => SharingError::NotAllowed,
        crate::Error::InvalidArgument(m) => SharingError::InvalidArgument(m),
        crate::Error::NotImplemented(m) => SharingError::not_implemented(m),
        crate::Error::Common { source } => SharingError::Common { source },
        crate::Error::SerDe { source } => SharingError::MalformedResponse { source },
        crate::Error::DeltaKernel { source } => SharingError::DeltaKernel { source },
        other => SharingError::Generic(other.to_string()),
    }
}

/// Map the [`ShareObjectKind`] the crate asks for onto the UC [`DataObjectType`].
fn data_object_type(kind: ShareObjectKind) -> DataObjectType {
    match kind {
        ShareObjectKind::Table => DataObjectType::Table,
        ShareObjectKind::Volume => DataObjectType::Volume,
        ShareObjectKind::AgentSkill => DataObjectType::AgentSkill,
    }
}

/// Map a Unity Catalog [`TemporaryCredential`] to the Open Sharing
/// [`SharingTemporaryCredentials`] envelope. The two carry the same
/// provider-specific payloads; only the message names differ.
fn to_sharing_credentials(cred: TemporaryCredential) -> BackendResult<SharingTemporaryCredentials> {
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
            return Err(SharingError::generic(
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

#[async_trait]
impl SharingBackend<RequestContext> for ServerHandler<RequestContext> {
    fn kernel_session(&self) -> &KernelSession {
        &self.session
    }

    async fn authorize(&self, action: SharingAction<'_>, cx: &RequestContext) -> BackendResult<()> {
        match action {
            SharingAction::ListShares => {
                // Share-undefined resource: read permission on the share surface.
                let request = ListSharesRequest::default();
                self.check_required(&request, cx)
                    .await
                    .map_err(to_sharing_err)
            }
            SharingAction::ReadShare { share } => {
                let request = GetShareRequest {
                    name: share.to_string(),
                    ..Default::default()
                };
                self.check_required(&request, cx)
                    .await
                    .map_err(to_sharing_err)
            }
        }
    }

    async fn list_shares(
        &self,
        max_results: Option<usize>,
        page_token: Option<String>,
        cx: &RequestContext,
    ) -> BackendResult<(Vec<ResolvedShare>, Option<String>)> {
        let (mut resources, next_page_token) = self
            .list(&ObjectLabel::Share, None, max_results, page_token)
            .await
            .map_err(to_sharing_err)?;
        process_resources(self, cx, &Permission::Read, &mut resources)
            .await
            .map_err(to_sharing_err)?;

        // If every resource on this page was filtered out but more pages remain,
        // fetch the next page rather than returning an empty result early.
        if resources.is_empty() && next_page_token.is_some() {
            return SharingBackend::list_shares(self, max_results, next_page_token, cx).await;
        }

        let shares: Vec<Share> = resources
            .into_iter()
            .map(|r| r.try_into())
            .try_collect()
            .map_err(to_sharing_err)?;
        let items = shares
            .into_iter()
            .map(|s| ResolvedShare {
                name: s.name,
                id: s.id,
                comment: s.comment,
            })
            .collect();
        Ok((items, next_page_token))
    }

    async fn get_share(&self, share: &str, _cx: &RequestContext) -> BackendResult<ResolvedShare> {
        let request = SharesGetShareRequest {
            name: share.to_string(),
            include_shared_data: Some(false),
            ..Default::default()
        };
        let share_info: Share = self
            .get(&request.resource())
            .await
            .map_err(to_sharing_err)?
            .0
            .try_into()
            .map_err(to_sharing_err)?;
        Ok(ResolvedShare {
            name: share_info.name,
            id: share_info.id,
            comment: share_info.comment,
        })
    }

    async fn list_share_objects(
        &self,
        share: &str,
        kind: ShareObjectKind,
        _cx: &RequestContext,
    ) -> BackendResult<Vec<ShareObject>> {
        let request = SharesGetShareRequest {
            name: share.to_string(),
            include_shared_data: Some(true),
            ..Default::default()
        };
        let share_info: Share = self
            .get(&request.resource())
            .await
            .map_err(to_sharing_err)?
            .0
            .try_into()
            .map_err(to_sharing_err)?;
        let want = data_object_type(kind);
        let items = share_info
            .objects
            .iter()
            .filter(|o| o.data_object_type == want)
            .filter_map(|o| {
                let (schema, name) = o.shared_as.as_deref().unwrap_or_default().split_once('.')?;
                Some(ShareObject {
                    schema: schema.to_string(),
                    name: name.to_string(),
                })
            })
            .collect();
        Ok(items)
    }

    async fn resolve_table_location(
        &self,
        table: &SharingTableReference,
        cx: &RequestContext,
    ) -> BackendResult<ResolvedLocation> {
        let share_ident = ResourceIdent::share(ResourceName::new([table.share.as_str()]));
        let share_info: Share = self
            .get(&share_ident)
            .await
            .map_err(to_sharing_err)?
            .0
            .try_into()
            .map_err(to_sharing_err)?;
        let Some(table_object) = share_info.objects.iter().find(|o| {
            o.shared_as.as_deref().unwrap_or_default()
                == format!("{}.{}", table.schema, table.table)
        }) else {
            return Err(SharingError::NotFound);
        };

        let table_info: Table = if let Some(table_source) = self.table_source() {
            // Side-by-side topology: resolve the Table primitive through the
            // routed handler (e.g. upstream Unity Catalog), keyed by full name.
            let request = GetTableRequest {
                full_name: table_object.name.clone(),
                ..Default::default()
            };
            table_source
                .get_table(request, cx.clone())
                .await
                .map_err(to_sharing_err)?
        } else {
            // Self-contained topology: the Table primitive lives in the local
            // store alongside the Share.
            let table_ident = ResourceIdent::table(ResourceName::new(table_object.name.split(".")));
            self.get(&table_ident)
                .await
                .map_err(to_sharing_err)?
                .0
                .try_into()
                .map_err(to_sharing_err)?
        };

        let location = table_info.storage_location.ok_or(SharingError::NotFound)?;
        resolve_from_raw(&location)
    }

    async fn resolve_asset_location(
        &self,
        asset: &SharingVolumeReference,
        kind: SharedAssetKind,
        cx: &RequestContext,
    ) -> BackendResult<ResolvedLocation> {
        let share_ident = ResourceIdent::share(ResourceName::new([asset.share.as_str()]));
        let share_info: Share = self
            .get(&share_ident)
            .await
            .map_err(to_sharing_err)?
            .0
            .try_into()
            .map_err(to_sharing_err)?;
        let shared_as = format!("{}.{}", asset.schema, asset.name);
        let want = match kind {
            SharedAssetKind::Volume => DataObjectType::Volume,
            SharedAssetKind::AgentSkill => DataObjectType::AgentSkill,
        };
        let Some(object) = share_info.objects.iter().find(|o| {
            o.shared_as.as_deref().unwrap_or_default() == shared_as && o.data_object_type == want
        }) else {
            return Err(SharingError::NotFound);
        };

        let volume_info: Volume = if let Some(volume_source) = self.volume_source() {
            // Side-by-side topology: resolve the Volume primitive through the
            // routed handler (e.g. upstream Unity Catalog), keyed by full name.
            let request = UcGetVolumeRequest {
                name: object.name.clone(),
                ..Default::default()
            };
            volume_source
                .get_volume(request, cx.clone())
                .await
                .map_err(to_sharing_err)?
        } else {
            // Self-contained topology: the Volume primitive lives in the local
            // store alongside the Share.
            let volume_ident = ResourceIdent::volume(ResourceName::new(object.name.split(".")));
            self.get(&volume_ident)
                .await
                .map_err(to_sharing_err)?
                .0
                .try_into()
                .map_err(to_sharing_err)?
        };

        resolve_from_raw(&volume_info.storage_location)
    }

    async fn vend_read_credential(
        &self,
        location: &ResolvedLocation,
        _cx: &RequestContext,
    ) -> BackendResult<SharingTemporaryCredentials> {
        let storage_location = StorageLocationUrl::parse(&location.raw).map_err(to_sharing_err)?;
        let ext_loc = find_external_location_for_url(&storage_location, self)
            .await
            .map_err(to_sharing_err)?;
        let credential = self
            .get_credential_internal(GetCredentialRequest {
                name: ext_loc.credential_name.clone(),
                ..Default::default()
            })
            .await
            .map_err(to_sharing_err)?;
        // Open Sharing only grants read access to shared assets.
        let cred = vend_credential(
            &credential,
            storage_location.raw().as_str(),
            VendOperation::Read,
        )
        .await
        .map_err(to_sharing_err)?;
        to_sharing_credentials(cred)
    }
}

/// Parse a raw storage-location string into a [`ResolvedLocation`], routing the
/// URL parse error through the server's `StorageLocationUrl` (so `file://`
/// normalization / scheme handling stays identical) and returning its
/// canonicalized inner location URL.
fn resolve_from_raw(raw: &str) -> BackendResult<ResolvedLocation> {
    let storage_location = StorageLocationUrl::parse(raw).map_err(to_sharing_err)?;
    Ok(ResolvedLocation {
        url: storage_location.location().clone(),
        raw: storage_location.raw().to_string(),
    })
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
