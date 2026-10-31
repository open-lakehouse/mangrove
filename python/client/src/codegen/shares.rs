// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::ShareClient;
use unitycatalog_common::models::shares::v1::*;
use unitycatalog_common::models::*;

/// The requests on one `Share`.
#[pyclass(name = "ShareClient", module = "unitycatalog_client._client.shares")]
pub struct PyShareClient {
    pub(crate) client: ShareClient,
}

#[pymethods]
impl PyShareClient {
    /// Get a share by name.
    ///
    /// Args:
    ///     include_shared_data: Query for data to include in the share.
    ///
    /// Returns:
    ///     The requested resource
    #[pyo3(signature = (include_shared_data = None))]
    pub fn get(
        &self,
        py: Python,
        include_shared_data: Option<bool>,
    ) -> PyUnityCatalogResult<PyShare> {
        let request = self
            .client
            .get()
            .with_include_shared_data(include_shared_data);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyShare::from(result))
        })
    }

    /// Update a share.
    ///
    /// Args:
    ///     updates: Array of shared data object updates. At most 100 data objects may be updated per
    ///              request.
    ///     new_name: A new name for the share.
    ///     owner: Owner of the share.
    ///     comment: User-provided free-form text description.
    ///
    /// Returns:
    ///     The requested resource
    #[pyo3(signature = (updates = None, new_name = None, owner = None, comment = None))]
    pub fn update(
        &self,
        py: Python,
        updates: ::core::option::Option<::std::vec::Vec<PyDataObjectUpdate>>,
        new_name: Option<String>,
        owner: Option<String>,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<PyShare> {
        let request = self
            .client
            .update()
            .with_updates(
                updates
                    .unwrap_or_default()
                    .into_iter()
                    .map(::core::convert::Into::into)
                    .collect::<::std::vec::Vec<_>>(),
            )
            .with_new_name(new_name)
            .with_owner(owner)
            .with_comment(comment);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyShare::from(result))
        })
    }

    /// Deletes a share.
    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let request = self.client.delete();
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }

    /// Gets the permissions for a data share from the metastore.
    ///
    /// Args:
    ///     max_results: The maximum number of results per page that should be returned. If unset or 0, the
    ///                  server's default page size is used. Negative values are rejected.
    ///     page_token: Opaque pagination token to go to next page based on previous query.
    ///
    /// Returns:
    ///     Response to list shares.
    #[pyo3(signature = (max_results = None, page_token = None))]
    pub fn get_permissions(
        &self,
        py: Python,
        max_results: Option<i32>,
        page_token: Option<String>,
    ) -> PyUnityCatalogResult<PyGetPermissionsResponse> {
        let request = self
            .client
            .get_permissions()
            .with_max_results(max_results)
            .with_page_token(page_token);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyGetPermissionsResponse::from(result))
        })
    }

    /// Updates the permissions for a data share in the metastore.
    ///
    /// Args:
    ///     changes: Array of permissions change objects.
    ///     omit_permissions_list: Whether to return the latest permissions list of the share in the
    ///                            response.
    ///
    /// Returns:
    ///     The requested resource
    #[pyo3(signature = (changes = None, omit_permissions_list = None))]
    pub fn update_permissions(
        &self,
        py: Python,
        changes: ::core::option::Option<::std::vec::Vec<PyPermissionsChange>>,
        omit_permissions_list: Option<bool>,
    ) -> PyUnityCatalogResult<PyUpdatePermissionsResponse> {
        let request = self
            .client
            .update_permissions()
            .with_changes(
                changes
                    .unwrap_or_default()
                    .into_iter()
                    .map(::core::convert::Into::into)
                    .collect::<::std::vec::Vec<_>>(),
            )
            .with_omit_permissions_list(omit_permissions_list);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyUpdatePermissionsResponse::from(result))
        })
    }
}

impl PyShareClient {
    pub fn new(client: ShareClient) -> Self {
        Self { client }
    }
}
