// @generated — do not edit by hand.
use crate::codegen::runtime::{ListIterator, block_on};
use crate::error::PyUnityCatalogError;
use futures::stream::StreamExt;
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::ShareClient;
use unitycatalog_common::models::shares::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "ShareClient")]
pub struct PyShareClient {
    pub(crate) client: ShareClient,
}
#[pymethods]
impl PyShareClient {
    #[pyo3(signature = (include_shared_data = None))]
    pub fn get(&self, py: Python<'_>, include_shared_data: Option<bool>) -> PyResult<PyShare> {
        let mut request = self.client.get();
        request = request.with_include_shared_data(include_shared_data);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyShare::from(result))
    }
    #[pyo3(signature = (updates = None, new_name = None, owner = None, comment = None))]
    pub fn update(
        &self,
        py: Python<'_>,
        updates: ::core::option::Option<::std::vec::Vec<PyDataObjectUpdate>>,
        new_name: Option<String>,
        owner: Option<String>,
        comment: Option<String>,
    ) -> PyResult<PyShare> {
        let mut request = self.client.update();
        if let Some(updates) = updates {
            request = request.with_updates(
                updates
                    .into_iter()
                    .map(::core::convert::Into::into)
                    .collect::<::std::vec::Vec<_>>(),
            );
        }
        request = request.with_new_name(new_name);
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyShare::from(result))
    }
    pub fn delete(&self, py: Python<'_>) -> PyResult<()> {
        let request = self.client.delete();
        block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(())
    }
    #[pyo3(signature = (max_results = None, page_token = None))]
    pub fn get_permissions(
        &self,
        py: Python<'_>,
        max_results: Option<i32>,
        page_token: Option<String>,
    ) -> PyResult<PyGetPermissionsResponse> {
        let mut request = self.client.get_permissions();
        request = request.with_max_results(max_results);
        request = request.with_page_token(page_token);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyGetPermissionsResponse::from(result))
    }
    #[pyo3(signature = (changes = None, omit_permissions_list = None))]
    pub fn update_permissions(
        &self,
        py: Python<'_>,
        changes: ::core::option::Option<::std::vec::Vec<PyPermissionsChange>>,
        omit_permissions_list: Option<bool>,
    ) -> PyResult<PyUpdatePermissionsResponse> {
        let mut request = self.client.update_permissions();
        if let Some(changes) = changes {
            request = request.with_changes(
                changes
                    .into_iter()
                    .map(::core::convert::Into::into)
                    .collect::<::std::vec::Vec<_>>(),
            );
        }
        request = request.with_omit_permissions_list(omit_permissions_list);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyUpdatePermissionsResponse::from(result))
    }
}
impl PyShareClient {
    pub fn new(client: ShareClient) -> Self {
        Self { client }
    }
}
