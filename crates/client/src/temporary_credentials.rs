use olai_http::CloudClient;
use reqwest::IntoUrl;
use unitycatalog_common::models::temporary_credentials::v1::TemporaryCredential;
use unitycatalog_common::{
    models::temporary_credentials::v1::{
        GenerateTemporaryPathCredentialsRequest, GenerateTemporaryTableCredentialsRequest,
        GenerateTemporaryVolumeCredentialsRequest,
        generate_temporary_path_credentials_request::Operation as PthOperation,
        generate_temporary_table_credentials_request::Operation as TblOperation,
        generate_temporary_volume_credentials_request::Operation as VolOperation,
    },
    tables::v1::GetTableRequest,
    volumes::v1::GetVolumeRequest,
};
use url::Url;
use uuid::Uuid;

use crate::Result;
use crate::codegen::tables::TableServiceClient;
pub(super) use crate::codegen::temporary_credentials::TemporaryCredentialClient as TemporaryCredentialClientBase;
use crate::codegen::volumes::client::VolumeServiceClient;

/// A reference to a table in unity catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableReference {
    /// The unique identifier of the table.
    Id(Uuid),
    /// The fully qualified name of the table.
    Name(String),
}

impl From<String> for TableReference {
    fn from(name: String) -> Self {
        TableReference::Name(name)
    }
}

impl From<&str> for TableReference {
    fn from(name: &str) -> Self {
        TableReference::Name(name.to_string())
    }
}

impl From<Uuid> for TableReference {
    fn from(id: Uuid) -> Self {
        TableReference::Id(id)
    }
}

/// A reference to a volume in unity catalog.
///
/// Use [`VolumeReference::Name`] for the three-level
/// `<catalog>.<schema>.<volume>` form and [`VolumeReference::Id`] when you
/// already hold the volume's UUID. The client resolves names to IDs
/// transparently on first use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeReference {
    /// The unique identifier of the volume.
    Id(Uuid),
    /// The fully qualified `<catalog>.<schema>.<volume>` name.
    Name(String),
}

impl From<String> for VolumeReference {
    fn from(name: String) -> Self {
        VolumeReference::Name(name)
    }
}

impl From<&str> for VolumeReference {
    fn from(name: &str) -> Self {
        VolumeReference::Name(name.to_string())
    }
}

impl From<Uuid> for VolumeReference {
    fn from(id: Uuid) -> Self {
        VolumeReference::Id(id)
    }
}

/// The kind of access requested for a table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableOperation {
    /// Read-only access.
    Read,
    /// Read and write access.
    ReadWrite,
}

impl From<TableOperation> for i32 {
    fn from(operation: TableOperation) -> Self {
        match operation {
            TableOperation::Read => TblOperation::Read as i32,
            TableOperation::ReadWrite => TblOperation::ReadWrite as i32,
        }
    }
}

impl From<TableOperation> for TblOperation {
    fn from(operation: TableOperation) -> Self {
        match operation {
            TableOperation::Read => TblOperation::Read,
            TableOperation::ReadWrite => TblOperation::ReadWrite,
        }
    }
}

/// The kind of access requested for a storage path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathOperation {
    /// Read-only access.
    Read,
    /// Read and write access.
    ReadWrite,
    /// Access for creating a new table at the path.
    CreateTable,
}

impl From<PathOperation> for i32 {
    fn from(operation: PathOperation) -> Self {
        match operation {
            PathOperation::Read => PthOperation::PathRead as i32,
            PathOperation::ReadWrite => PthOperation::PathReadWrite as i32,
            PathOperation::CreateTable => PthOperation::PathCreateTable as i32,
        }
    }
}

impl From<PathOperation> for PthOperation {
    fn from(operation: PathOperation) -> Self {
        match operation {
            PathOperation::Read => PthOperation::PathRead,
            PathOperation::ReadWrite => PthOperation::PathReadWrite,
            PathOperation::CreateTable => PthOperation::PathCreateTable,
        }
    }
}

/// The kind of access requested for a volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeOperation {
    /// Read-only access.
    Read,
    /// Read and write access.
    ReadWrite,
}

impl From<VolumeOperation> for i32 {
    fn from(operation: VolumeOperation) -> Self {
        match operation {
            VolumeOperation::Read => VolOperation::ReadVolume as i32,
            VolumeOperation::ReadWrite => VolOperation::WriteVolume as i32,
        }
    }
}

impl From<VolumeOperation> for VolOperation {
    fn from(operation: VolumeOperation) -> Self {
        match operation {
            VolumeOperation::Read => VolOperation::ReadVolume,
            VolumeOperation::ReadWrite => VolOperation::WriteVolume,
        }
    }
}

/// Client for vending temporary storage credentials for tables, volumes, and paths.
///
/// Wraps the generated low-level client with name → UUID resolution: callers may
/// pass a fully qualified name or a UUID, and the client issues the lookup needed
/// to obtain the ID the credential endpoint requires. Prefer
/// [`UnityCatalogClient::temporary_credentials`](crate::UnityCatalogClient::temporary_credentials).
#[derive(Clone)]
pub struct TemporaryCredentialClient {
    client: TemporaryCredentialClientBase,
}

impl TemporaryCredentialClient {
    /// Creates a client from a [`CloudClient`] (carrying auth) and a base URL.
    ///
    /// The base URL is normalized to end in `/` so it joins cleanly with the
    /// relative endpoint paths.
    pub fn new_with_url(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self {
            client: TemporaryCredentialClientBase::new(client, base_url),
        }
    }

    /// Creates a client wrapping an existing generated low-level client.
    pub fn new(client: TemporaryCredentialClientBase) -> Self {
        Self { client }
    }

    /// POST a credential request and deserialize the [`TemporaryCredential`]
    /// response, tolerating servers that emit the inactive `credentials` oneof
    /// siblings as explicit `null`s.
    ///
    /// The wire `credentials` field is a protobuf oneof (`aws_temp_credentials`,
    /// `azure_user_delegation_sas`, …). Our pbjson-generated deserializer maps
    /// every oneof key into a single slot and rejects the *second* oneof key it
    /// sees with a `duplicate field` error — even when that key's value is
    /// `null`. Some servers (e.g. the OSS reference) serialize all oneof
    /// variants, with the inactive ones set to `null`, which trips that check.
    /// We strip those null siblings before handing the body to the generated
    /// deserializer, so only the active variant remains. We replicate the
    /// generated client's request/error handling so behaviour is otherwise
    /// identical.
    async fn post_credential<R: serde::Serialize>(
        &self,
        path: &str,
        request: &R,
    ) -> Result<TemporaryCredential> {
        let url = self.client.base_url.join(path)?;
        let response = self
            .client
            .client
            .post(url)
            .json(request)
            .send_raw()
            .await
            .map_err(crate::Error::from_api_send)?;
        let bytes = response.bytes().await?;

        // Drop any `credentials` oneof member whose value is null, so the
        // generated oneof deserializer sees at most one of them. The generated
        // field matcher accepts both snake_case and camelCase, so match either.
        let mut value: serde_json::Value = serde_json::from_slice(&bytes)?;
        if let Some(obj) = value.as_object_mut() {
            const ONEOF_KEYS: [&str; 10] = [
                "aws_temp_credentials",
                "awsTempCredentials",
                "azure_user_delegation_sas",
                "azureUserDelegationSas",
                "azure_aad",
                "azureAad",
                "gcp_oauth_token",
                "gcpOauthToken",
                "r2_temp_credentials",
                "r2TempCredentials",
            ];
            obj.retain(|key, v| !(v.is_null() && ONEOF_KEYS.contains(&key.as_str())));
        }
        Ok(serde_json::from_value(value)?)
    }

    /// Gets a temporary credential for reading or writing to a table.
    ///
    /// # Arguments
    ///
    /// * `table`: The table to get a temporary credential for.
    /// * `operation`: The operation to perform on the table.
    ///
    /// Returns a tuple of the temporary credential and the resolved table ID.
    pub async fn temporary_table_credential(
        &self,
        table: impl Into<TableReference>,
        operation: TableOperation,
    ) -> Result<(TemporaryCredential, Uuid)> {
        // Resolve a name to an id, capturing the table's storage location so we
        // can backfill it onto the credential if the server omits the `url`.
        let (table_id, storage_location) = match table.into() {
            TableReference::Id(id) => (id.as_hyphenated().to_string(), None),
            TableReference::Name(name) => {
                let table_client = TableServiceClient::new(
                    self.client.client.clone(),
                    self.client.base_url.clone(),
                );
                let table_info = table_client
                    .get_table(&GetTableRequest {
                        full_name: name,
                        include_browse: Some(false),
                        include_delta_metadata: Some(false),
                        include_manifest_capabilities: Some(false),
                        ..Default::default()
                    })
                    .await?;
                (
                    table_info.table_id.clone().unwrap_or_default(),
                    table_info.storage_location.clone(),
                )
            }
        };
        let uuid =
            Uuid::parse_str(&table_id).map_err(unitycatalog_common::Error::InvalidIdentifier)?;
        let mut credential = self
            .post_credential(
                "temporary-table-credentials",
                &GenerateTemporaryTableCredentialsRequest {
                    table_id,
                    operation: TblOperation::from(operation).into(),
                    ..Default::default()
                },
            )
            .await?;
        backfill_credential_url(&mut credential, storage_location);
        Ok((credential, uuid))
    }

    /// Gets a temporary credential for reading or writing to a storage path.
    ///
    /// Pass `dry_run` to validate access without vending a usable credential.
    /// Returns a tuple of the temporary credential and the parsed path URL.
    pub async fn temporary_path_credential(
        &self,
        path: impl IntoUrl,
        operation: PathOperation,
        dry_run: impl Into<Option<bool>>,
    ) -> Result<(TemporaryCredential, Url)> {
        let url = path.into_url()?;
        Ok((
            self.post_credential(
                "temporary-path-credentials",
                &GenerateTemporaryPathCredentialsRequest {
                    url: url.to_string(),
                    operation: PthOperation::from(operation).into(),
                    dry_run: dry_run.into(),
                    ..Default::default()
                },
            )
            .await?,
            url,
        ))
    }

    /// Gets a temporary credential for reading or writing to a volume.
    ///
    /// # Arguments
    ///
    /// * `volume`: The volume to get a temporary credential for. May be either
    ///   a [`VolumeReference::Id`] (UUID, preferred when known) or a
    ///   [`VolumeReference::Name`] in three-level dotted form
    ///   (`catalog.schema.volume`). Names are resolved to IDs by issuing a
    ///   `GetVolume` request.
    /// * `operation`: Whether the credentials should grant read-only or
    ///   read-write access.
    ///
    /// Returns a tuple of the temporary credential and the resolved volume ID.
    ///
    /// The Unity Catalog metastore must have `external_access_enabled = true`
    /// and the caller must hold `EXTERNAL_USE_SCHEMA` on the parent schema.
    pub async fn temporary_volume_credential(
        &self,
        volume: impl Into<VolumeReference>,
        operation: VolumeOperation,
    ) -> Result<(TemporaryCredential, Uuid)> {
        let (volume_id, storage_location) = match volume.into() {
            VolumeReference::Id(id) => (id.as_hyphenated().to_string(), None),
            VolumeReference::Name(name) => {
                let volume_client = VolumeServiceClient::new(
                    self.client.client.clone(),
                    self.client.base_url.clone(),
                );
                let info = volume_client
                    .get_volume(&GetVolumeRequest {
                        name,
                        include_browse: Some(false),
                        ..Default::default()
                    })
                    .await?;
                (info.volume_id, Some(info.storage_location))
            }
        };
        let uuid =
            Uuid::parse_str(&volume_id).map_err(unitycatalog_common::Error::InvalidIdentifier)?;
        let mut credential = self
            .post_credential(
                "temporary-volume-credentials",
                &GenerateTemporaryVolumeCredentialsRequest {
                    volume_id,
                    operation: VolOperation::from(operation).into(),
                    ..Default::default()
                },
            )
            .await?;
        backfill_credential_url(&mut credential, storage_location);
        Ok((credential, uuid))
    }
}

/// Backfill a credential's `url` from a known storage location when the server
/// left it empty.
///
/// The `url` field of [`TemporaryCredential`] ("the storage path accessible by
/// the temporary credential") is required by downstream object-store wiring, but
/// some servers (e.g. the OSS reference) omit it for table/volume credentials and
/// only echo it for path credentials. When we already know the securable's
/// storage location we fill it in so the store can be built.
fn backfill_credential_url(credential: &mut TemporaryCredential, storage_location: Option<String>) {
    if credential.url.is_empty()
        && let Some(location) = storage_location
        && !location.is_empty()
    {
        credential.url = location;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A malformed server-supplied table id must surface as an error, not panic
    /// (the credential path previously called `Uuid::parse_str(..).unwrap()`).
    #[test]
    fn malformed_table_id_is_error_not_panic() {
        let result =
            Uuid::parse_str("not-a-uuid").map_err(unitycatalog_common::Error::InvalidIdentifier);
        let err: crate::Error = result.unwrap_err().into();
        assert!(matches!(err, crate::Error::Common { .. }));
    }
}
