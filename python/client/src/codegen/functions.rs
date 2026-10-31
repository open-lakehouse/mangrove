// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::FunctionClient;
use unitycatalog_common::models::functions::v1::*;
use unitycatalog_common::models::*;

/// The requests on one `Function`.
#[pyclass(
    name = "FunctionClient",
    module = "unitycatalog_client._client.functions"
)]
pub struct PyFunctionClient {
    pub(crate) client: FunctionClient,
}

#[pymethods]
impl PyFunctionClient {
    /// Get a function
    ///
    /// Gets a function from within a parent catalog and schema. For the fetch to succeed, the caller must
    /// be a metastore admin, the owner of the function, or have SELECT on the function.
    ///
    /// Args:
    ///     include_browse: Whether to include functions in the response for which the principal can only
    ///                     access selective metadata for
    ///
    /// Returns:
    ///     A User-Defined Function (UDF) registered under a catalog + schema hierarchy.
    #[pyo3(signature = (include_browse = None))]
    pub fn get(
        &self,
        py: Python,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<PyFunction> {
        let request = self.client.get().with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyFunction::from(result))
        })
    }

    /// Update a function
    ///
    /// Updates the function that matches the supplied name. Only the owner of the function can be updated.
    ///
    /// Args:
    ///     owner: Username of new owner of the function.
    ///
    /// Returns:
    ///     A User-Defined Function (UDF) registered under a catalog + schema hierarchy.
    #[pyo3(signature = (owner = None))]
    pub fn update(&self, py: Python, owner: Option<String>) -> PyUnityCatalogResult<PyFunction> {
        let request = self.client.update().with_owner(owner);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyFunction::from(result))
        })
    }

    /// Delete a function
    ///
    /// Deletes the function that matches the supplied name. For the deletion to succeed, the caller must be
    /// the owner of the function.
    ///
    /// Args:
    ///     force: Force deletion even if the function is not empty.
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

impl PyFunctionClient {
    pub fn new(client: FunctionClient) -> Self {
        Self { client }
    }
}
