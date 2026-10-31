// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::ExternalLocationClient;
use unitycatalog_common::models::external_locations::v1::*;
use unitycatalog_common::models::*;

/// The requests on one `ExternalLocation`.
#[pyclass(
    name = "ExternalLocationClient",
    module = "unitycatalog_client._client.external_locations"
)]
pub struct PyExternalLocationClient {
    pub(crate) client: ExternalLocationClient,
}

#[pymethods]
impl PyExternalLocationClient {
    /// Get an external location
    ///
    /// Args:
    ///     include_browse: Whether to include external locations in the response for which the principal
    ///                     can only access selective metadata for
    ///
    /// Returns:
    ///     The requested resource
    #[pyo3(signature = (include_browse = None))]
    pub fn get(
        &self,
        py: Python,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<PyExternalLocation> {
        let request = self.client.get().with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyExternalLocation::from(result))
        })
    }

    /// Update an external location
    ///
    /// Args:
    ///     url: Path URL of the external location.
    ///     credential_name: Name of the storage credential used with this location.
    ///     read_only: Indicates whether the external location is read-only.
    ///     owner: owner of the external location.
    ///     comment: User-provided free-form text description.
    ///     new_name: new name of the external location.
    ///     force: force update of the external location.
    ///     skip_validation: Skips validation of the storage credential associated with the external
    ///                      location.
    ///
    /// Returns:
    ///     The requested resource
    #[pyo3(
        signature = (
            url = None,
            credential_name = None,
            read_only = None,
            owner = None,
            comment = None,
            new_name = None,
            force = None,
            skip_validation = None
        )
    )]
    pub fn update(
        &self,
        py: Python,
        url: Option<String>,
        credential_name: Option<String>,
        read_only: Option<bool>,
        owner: Option<String>,
        comment: Option<String>,
        new_name: Option<String>,
        force: Option<bool>,
        skip_validation: Option<bool>,
    ) -> PyUnityCatalogResult<PyExternalLocation> {
        let request = self
            .client
            .update()
            .with_url(url)
            .with_credential_name(credential_name)
            .with_read_only(read_only)
            .with_owner(owner)
            .with_comment(comment)
            .with_new_name(new_name)
            .with_force(force)
            .with_skip_validation(skip_validation);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyExternalLocation::from(result))
        })
    }

    /// Delete an external location
    ///
    /// Args:
    ///     force: Force deletion even if the external location is not empty.
    #[pyo3(signature = (force = None))]
    pub fn delete(&self, py: Python, force: Option<bool>) -> PyUnityCatalogResult<()> {
        let request = self.client.delete().with_force(force);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
}

impl PyExternalLocationClient {
    pub fn new(client: ExternalLocationClient) -> Self {
        Self { client }
    }
}
