// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::SchemaClient;
use unitycatalog_common::models::schemas::v1::*;
use unitycatalog_common::models::*;

/// The requests on one `Schema`.
#[pyclass(name = "SchemaClient", module = "unitycatalog_client._client.schemas")]
pub struct PySchemaClient {
    pub(crate) client: SchemaClient,
}

#[pymethods]
impl PySchemaClient {
    /// Gets the specified schema within the metastore. The caller must be a metastore admin, the owner of
    /// the schema, or a user that has the USE_SCHEMA privilege on the schema.
    ///
    /// Args:
    ///     include_browse: Whether to include schemas in the response for which the principal can only
    ///                     access selective metadata for
    ///
    /// Returns:
    ///     A schema is a namespace within a catalog that contains tables.
    #[pyo3(signature = (include_browse = None))]
    pub fn get(&self, py: Python, include_browse: Option<bool>) -> PyUnityCatalogResult<PySchema> {
        let request = self.client.get().with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PySchema::from(result))
        })
    }

    /// Updates a schema for a catalog. The caller must be the owner of the schema or a metastore admin.
    /// If the caller is a metastore admin, only the owner field can be changed in the update. If the name
    /// field must be updated, the caller must be a metastore admin or have the CREATE_SCHEMA privilege on
    /// the parent catalog.
    ///
    /// Args:
    ///     comment: User-provided free-form text description.
    ///     properties: A map of key-value properties attached to the securable. When provided in update
    ///                 request, the specified properties will override the existing properties. To add and
    ///                 remove properties, one would need to perform a read-modify-write.
    ///     new_name: New name for the schema.
    ///
    /// Returns:
    ///     A schema is a namespace within a catalog that contains tables.
    #[pyo3(signature = (comment = None, properties = None, new_name = None))]
    pub fn update(
        &self,
        py: Python,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        new_name: Option<String>,
    ) -> PyUnityCatalogResult<PySchema> {
        let request = self
            .client
            .update()
            .with_comment(comment)
            .with_properties(properties.unwrap_or_default())
            .with_new_name(new_name);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PySchema::from(result))
        })
    }

    /// Deletes the specified schema from the parent catalog. The caller must be the owner of the schema or
    /// an owner of the parent catalog.
    ///
    /// Args:
    ///     force: Force deletion even if the schema is not empty.
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

impl PySchemaClient {
    pub fn new(client: SchemaClient) -> Self {
        Self { client }
    }
}
