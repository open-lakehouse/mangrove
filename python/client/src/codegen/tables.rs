// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::TableClient;
use unitycatalog_common::models::tables::v1::*;
use unitycatalog_common::models::*;

/// The requests on one `Table`.
#[pyclass(name = "TableClient", module = "unitycatalog_client._client.tables")]
pub struct PyTableClient {
    pub(crate) client: TableClient,
}

#[pymethods]
impl PyTableClient {
    /// Get a table
    ///
    /// Args:
    ///     include_delta_metadata: Whether delta metadata should be included in the response.
    ///     include_browse: Whether to include tables in the response for which the principal can only
    ///                     access selective metadata for
    ///     include_manifest_capabilities: Whether to include a manifest containing capabilities the table
    ///                                    has.
    ///
    /// Returns:
    ///     The requested resource
    #[pyo3(
        signature = (
            include_delta_metadata = None,
            include_browse = None,
            include_manifest_capabilities = None
        )
    )]
    pub fn get(
        &self,
        py: Python,
        include_delta_metadata: Option<bool>,
        include_browse: Option<bool>,
        include_manifest_capabilities: Option<bool>,
    ) -> PyUnityCatalogResult<PyTable> {
        let request = self
            .client
            .get()
            .with_include_delta_metadata(include_delta_metadata)
            .with_include_browse(include_browse)
            .with_include_manifest_capabilities(include_manifest_capabilities);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyTable::from(result))
        })
    }

    /// Get boolean reflecting if table exists
    ///
    /// Returns:
    ///     The requested resource
    pub fn get_table_exists(&self, py: Python) -> PyUnityCatalogResult<PyGetTableExistsResponse> {
        let request = self.client.get_table_exists();
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyGetTableExistsResponse::from(result))
        })
    }

    /// Delete a table
    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let request = self.client.delete();
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
}

impl PyTableClient {
    pub fn new(client: TableClient) -> Self {
        Self { client }
    }
}
