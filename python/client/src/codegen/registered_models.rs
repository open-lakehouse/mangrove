// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::RegisteredModelClient;
use unitycatalog_common::models::registered_models::v1::*;
use unitycatalog_common::models::*;

/// The requests on one `RegisteredModel`.
#[pyclass(
    name = "RegisteredModelClient",
    module = "unitycatalog_client._client.registered_models"
)]
pub struct PyRegisteredModelClient {
    pub(crate) client: RegisteredModelClient,
}

#[pymethods]
impl PyRegisteredModelClient {
    /// Get a registered model
    ///
    /// Gets a registered model from within a parent catalog and schema. For the fetch to succeed, the
    /// caller must be a metastore admin, the owner of the registered model, or have a privilege on the
    /// registered model.
    ///
    /// Args:
    ///     include_browse: Whether to include registered models in the response for which the principal can
    ///                     only access selective metadata for.
    ///
    /// Returns:
    ///     A registered model within the three-level namespace (catalog.schema.model). Registered
    ///     models
    ///     group a collection of model versions. Individual versions carry their own artifact storage
    ///     location and are created, finalized, and governed independently.
    #[pyo3(signature = (include_browse = None))]
    pub fn get(
        &self,
        py: Python,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<PyRegisteredModel> {
        let request = self.client.get().with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyRegisteredModel::from(result))
        })
    }

    /// Update a registered model
    ///
    /// Updates the registered model that matches the supplied name.
    ///
    /// Args:
    ///     new_name: New name for the registered model.
    ///     comment: User-provided free-form text description.
    ///     owner: Username of new owner of the registered model.
    ///
    /// Returns:
    ///     A registered model within the three-level namespace (catalog.schema.model). Registered
    ///     models
    ///     group a collection of model versions. Individual versions carry their own artifact storage
    ///     location and are created, finalized, and governed independently.
    #[pyo3(signature = (new_name = None, comment = None, owner = None))]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        comment: Option<String>,
        owner: Option<String>,
    ) -> PyUnityCatalogResult<PyRegisteredModel> {
        let request = self
            .client
            .update()
            .with_new_name(new_name)
            .with_comment(comment)
            .with_owner(owner);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyRegisteredModel::from(result))
        })
    }

    /// Delete a registered model
    ///
    /// Deletes the registered model that matches the supplied name. For the deletion to succeed, the caller
    /// must be the owner of the registered model.
    ///
    /// Args:
    ///     force: Force deletion even if the registered model still has model versions.
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

impl PyRegisteredModelClient {
    pub fn new(client: RegisteredModelClient) -> Self {
        Self { client }
    }
}
