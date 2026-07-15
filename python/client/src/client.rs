use olai_http::CloudClient;
use pyo3::prelude::*;
use unitycatalog_client::{
    ModelVersionOperation, PathOperation, TableOperation, TableReference,
    TemporaryCredentialClient, VolumeOperation, VolumeReference,
};
// Return the generated `#[pyclass]` wrapper — the bare buffa `TemporaryCredential`
// model is not a Python object.
use unitycatalog_common::models::PyTemporaryCredential;

pub use crate::codegen::PyUnityCatalogClient;
pub use crate::codegen::catalogs::PyCatalogClient;
pub use crate::codegen::credentials::PyCredentialClient;
pub use crate::codegen::external_locations::PyExternalLocationClient;
pub use crate::codegen::recipients::PyRecipientClient;
pub use crate::codegen::schemas::PySchemaClient;
pub use crate::codegen::shares::PyShareClient;
pub use crate::codegen::tables::PyTableClient;
pub use crate::codegen::tag_policies::PyTagPolicyClient;
pub use crate::codegen::volumes::PyVolumeClient;
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;

/// Client for vending short-lived credentials for tables, volumes, and
/// raw external paths governed by Unity Catalog.
///
/// Construct directly with `TemporaryCredentialClient(base_url=...,
/// token=...)`, or use the convenience helpers in
/// [`unitycatalog_client.obstore`](crate::obstore) to plug the credentials
/// into the `obstore` Python library.
#[pyclass(name = "TemporaryCredentialClient", module = "unitycatalog_client")]
pub struct PyTemporaryCredentialClient {
    client: TemporaryCredentialClient,
}

#[pymethods]
impl PyTemporaryCredentialClient {
    #[new]
    #[pyo3(signature = (base_url, token = None))]
    pub fn new(base_url: String, token: Option<String>) -> PyResult<Self> {
        let cloud = if let Some(token) = token {
            CloudClient::new_with_token(token)
        } else {
            CloudClient::new_unauthenticated()
        };
        let base_url = base_url.parse().map_err(PyUnityCatalogError::from)?;
        Ok(Self {
            client: TemporaryCredentialClient::new_with_url(cloud, base_url),
        })
    }

    /// Vend a temporary credential for a Unity Catalog table.
    #[pyo3(signature = (table, operation))]
    pub fn temporary_table_credential(
        &self,
        py: Python,
        table: String,
        operation: String,
    ) -> PyUnityCatalogResult<(PyTemporaryCredential, String)> {
        let table_ref = TableReference::Name(table);
        let op = match operation.to_ascii_lowercase().as_str() {
            "read" => TableOperation::Read,
            "read_write" => TableOperation::ReadWrite,
            _ => {
                return Err(PyUnityCatalogError::from(
                    unitycatalog_common::error::Error::invalid_argument(format!(
                        "Invalid operation: {}. Must be 'read' or 'read_write'",
                        operation
                    )),
                ));
            }
        };
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let (credential, uuid) =
                runtime.block_on(self.client.temporary_table_credential(table_ref, op))?;
            Ok::<_, PyUnityCatalogError>((
                PyTemporaryCredential::from(credential),
                uuid.to_string(),
            ))
        })
    }

    /// Vend a temporary credential for a Unity Catalog volume.
    ///
    /// `volume` is the three-level `catalog.schema.volume` name. Server
    /// support requires the metastore's `external_access_enabled` flag and
    /// the caller's `EXTERNAL_USE_SCHEMA` privilege.
    #[pyo3(signature = (volume, operation))]
    pub fn temporary_volume_credential(
        &self,
        py: Python,
        volume: String,
        operation: String,
    ) -> PyUnityCatalogResult<(PyTemporaryCredential, String)> {
        let volume_ref = VolumeReference::Name(volume);
        let op = match operation.to_ascii_lowercase().as_str() {
            "read" => VolumeOperation::Read,
            "read_write" | "write" => VolumeOperation::ReadWrite,
            _ => {
                return Err(PyUnityCatalogError::from(
                    unitycatalog_common::error::Error::invalid_argument(format!(
                        "Invalid operation: {}. Must be 'read' or 'read_write'",
                        operation
                    )),
                ));
            }
        };
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let (credential, uuid) =
                runtime.block_on(self.client.temporary_volume_credential(volume_ref, op))?;
            Ok::<_, PyUnityCatalogError>((
                PyTemporaryCredential::from(credential),
                uuid.to_string(),
            ))
        })
    }

    #[pyo3(signature = (path, operation, dry_run = None))]
    pub fn temporary_path_credential(
        &self,
        py: Python,
        path: String,
        operation: String,
        dry_run: Option<bool>,
    ) -> PyUnityCatalogResult<(PyTemporaryCredential, String)> {
        let runtime = get_runtime(py)?;
        let op = match operation.as_str().to_ascii_lowercase().as_str() {
            "read" => PathOperation::Read,
            "read_write" => PathOperation::ReadWrite,
            "create_table" => PathOperation::CreateTable,
            _ => {
                return Err(PyUnityCatalogError::from(
                    unitycatalog_common::error::Error::invalid_argument(format!(
                        "Invalid operation: {}. Must be 'read', 'read_write', or 'create_table'",
                        operation
                    )),
                ));
            }
        };
        py.allow_threads(|| {
            let (credential, url) =
                runtime.block_on(self.client.temporary_path_credential(path, op, dry_run))?;
            Ok::<_, PyUnityCatalogError>((PyTemporaryCredential::from(credential), url.to_string()))
        })
    }

    /// Vend a temporary credential for a Unity Catalog model version.
    ///
    /// `full_name` is the three-level `catalog.schema.model` name of the parent
    /// registered model; `version` is its integer version number. Server support
    /// requires the metastore's `external_access_enabled` flag and the caller's
    /// `EXTERNAL_USE_SCHEMA` privilege.
    #[pyo3(signature = (full_name, version, operation))]
    pub fn temporary_model_version_credential(
        &self,
        py: Python,
        full_name: String,
        version: i64,
        operation: String,
    ) -> PyUnityCatalogResult<PyTemporaryCredential> {
        let op = match operation.to_ascii_lowercase().as_str() {
            "read" => ModelVersionOperation::Read,
            "read_write" | "write" => ModelVersionOperation::ReadWrite,
            _ => {
                return Err(PyUnityCatalogError::from(
                    unitycatalog_common::error::Error::invalid_argument(format!(
                        "Invalid operation: {}. Must be 'read' or 'read_write'",
                        operation
                    )),
                ));
            }
        };
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let credential = runtime.block_on(
                self.client
                    .temporary_model_version_credential(full_name, version, op),
            )?;
            Ok::<_, PyUnityCatalogError>(PyTemporaryCredential::from(credential))
        })
    }
}
