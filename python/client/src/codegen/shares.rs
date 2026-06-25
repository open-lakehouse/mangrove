// @generated — do not edit by hand.
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
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
    pub fn get(
        &self,
        py: Python,
        include_shared_data: Option<bool>,
    ) -> PyUnityCatalogResult<PyShare> {
        let mut request = self.client.get();
        request = request.with_include_shared_data(include_shared_data);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyShare::from(result))
        })
    }
    #[pyo3(signature = (updates = None, new_name = None, owner = None, comment = None))]
    pub fn update(
        &self,
        py: Python,
        updates: ::core::option::Option<::std::vec::Vec<PyDataObjectUpdate>>,
        new_name: Option<String>,
        owner: Option<String>,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<PyShare> {
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
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyShare::from(result))
        })
    }
    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let request = self.client.delete();
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
    #[pyo3(signature = (max_results = None, page_token = None))]
    pub fn get_permissions(
        &self,
        py: Python,
        max_results: Option<i32>,
        page_token: Option<String>,
    ) -> PyUnityCatalogResult<PyGetPermissionsResponse> {
        let mut request = self.client.get_permissions();
        request = request.with_max_results(max_results);
        request = request.with_page_token(page_token);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyGetPermissionsResponse::from(result))
        })
    }
    #[pyo3(signature = (changes = None, omit_permissions_list = None))]
    pub fn update_permissions(
        &self,
        py: Python,
        changes: ::core::option::Option<::std::vec::Vec<PyPermissionsChange>>,
        omit_permissions_list: Option<bool>,
    ) -> PyUnityCatalogResult<PyUpdatePermissionsResponse> {
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
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
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
