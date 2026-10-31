// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::TagPolicyClient;
use unitycatalog_common::models::tags::v1::*;
use unitycatalog_common::models::*;

/// The requests on one `TagPolicy`.
#[pyclass(name = "TagPolicyClient", module = "unitycatalog_client._client.tags")]
pub struct PyTagPolicyClient {
    pub(crate) client: TagPolicyClient,
}

#[pymethods]
impl PyTagPolicyClient {
    /// Get a tag policy
    ///
    /// Gets the governed tag definition for the specified tag key.
    ///
    /// Returns:
    ///     A governed tag definition (tag policy). A tag policy defines a tag key together with the
    ///     rules
    ///     that govern how it can be used, including the optional set of allowed values. Assigning a
    ///     governed tag to an entity is done through the Entity Tag Assignments API.
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<PyTagPolicy> {
        let request = self.client.get();
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyTagPolicy::from(result))
        })
    }

    /// Update a tag policy
    ///
    /// Updates the governed tag definition that matches the supplied tag key.
    ///
    /// Args:
    ///     description: User-provided free-form text description of the tag policy.
    ///     values: The set of allowed values for the governed tag. The list is unordered.
    ///     update_mask: The fields to update, as a comma-separated list of field names. If empty or "*",
    ///                  every field set in the request is updated. Otherwise only the listed fields are
    ///                  updated, and a listed field that is unset in the request is cleared.
    ///
    /// Returns:
    ///     A governed tag definition (tag policy). A tag policy defines a tag key together with the
    ///     rules
    ///     that govern how it can be used, including the optional set of allowed values. Assigning a
    ///     governed tag to an entity is done through the Entity Tag Assignments API.
    #[pyo3(signature = (description = None, values = None, update_mask = None))]
    pub fn update(
        &self,
        py: Python,
        description: Option<String>,
        values: ::core::option::Option<::std::vec::Vec<PyValue>>,
        update_mask: Option<String>,
    ) -> PyUnityCatalogResult<PyTagPolicy> {
        let request = self
            .client
            .update()
            .with_description(description)
            .with_values(
                values
                    .unwrap_or_default()
                    .into_iter()
                    .map(::core::convert::Into::into)
                    .collect::<::std::vec::Vec<_>>(),
            )
            .with_update_mask(update_mask);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyTagPolicy::from(result))
        })
    }

    /// Delete a tag policy
    ///
    /// Deletes the governed tag definition that matches the supplied tag key.
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

impl PyTagPolicyClient {
    pub fn new(client: TagPolicyClient) -> Self {
        Self { client }
    }
}
