// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::CatalogClient;
use unitycatalog_common::models::catalogs::v1::*;
use unitycatalog_common::models::*;

/// The requests on one `Catalog`.
#[pyclass(
    name = "CatalogClient",
    module = "unitycatalog_client._client.catalogs"
)]
pub struct PyCatalogClient {
    pub(crate) client: CatalogClient,
}

#[pymethods]
impl PyCatalogClient {
    /// Get a catalog
    ///
    /// Gets the specified catalog in a metastore. The caller must be a metastore admin, the owner of the
    /// catalog, or a user that has the USE_CATALOG privilege set for their account.
    ///
    /// Args:
    ///     include_browse: Whether to include catalogs in the response for which the principal can only
    ///                     access selective metadata for
    ///
    /// Returns:
    ///     A catalog is a root-level namespace that contains schemas.
    #[pyo3(signature = (include_browse = None))]
    pub fn get(&self, py: Python, include_browse: Option<bool>) -> PyUnityCatalogResult<PyCatalog> {
        let request = self.client.get().with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyCatalog::from(result))
        })
    }

    /// Update a catalog
    ///
    /// Updates the catalog that matches the supplied name. The caller must be either the owner of the
    /// catalog, or a metastore admin (when changing the owner field of the catalog).
    ///
    /// Args:
    ///     owner: Username of new owner of catalog.
    ///     comment: User-provided free-form text description.
    ///     properties: A map of key-value properties attached to the securable. When provided in update
    ///                 request, the specified properties will override the existing properties. To add and
    ///                 remove properties, one would need to perform a read-modify-write.
    ///     new_name: New name for the catalog.
    ///
    /// Returns:
    ///     A catalog is a root-level namespace that contains schemas.
    #[pyo3(
        signature = (owner = None, comment = None, properties = None, new_name = None)
    )]
    pub fn update(
        &self,
        py: Python,
        owner: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        new_name: Option<String>,
    ) -> PyUnityCatalogResult<PyCatalog> {
        let request = self
            .client
            .update()
            .with_owner(owner)
            .with_comment(comment)
            .with_properties(properties.unwrap_or_default())
            .with_new_name(new_name);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyCatalog::from(result))
        })
    }

    /// Delete a catalog
    ///
    /// Deletes the catalog that matches the supplied name. The caller must be a metastore admin or the
    /// owner of the catalog.
    ///
    /// Args:
    ///     force: Force deletion even if the catalog is not empty.
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

impl PyCatalogClient {
    pub fn new(client: CatalogClient) -> Self {
        Self { client }
    }
}
