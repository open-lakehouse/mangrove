// @generated — do not edit by hand.
use crate::codegen::runtime::{ListIterator, block_on};
use crate::error::PyUnityCatalogError;
use futures::stream::StreamExt;
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::PolicyClient;
use unitycatalog_common::models::policies::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "PolicyClient")]
pub struct PyPolicyClient {
    pub(crate) client: PolicyClient,
}
#[pymethods]
impl PyPolicyClient {
    pub fn get(&self, py: Python<'_>) -> PyResult<PyPolicyInfo> {
        let request = self.client.get();
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyPolicyInfo::from(result))
    }
    #[pyo3(
        signature = (
            policy_type = None,
            comment = None,
            to_principals = None,
            except_principals = None,
            for_securable_type = None,
            when_condition = None,
            match_columns = None,
            row_filter = None,
            column_mask = None,
            deny = None,
            grant = None,
            update_mask = None
        )
    )]
    pub fn update(
        &self,
        py: Python<'_>,
        policy_type: ::core::option::Option<PyPolicyType>,
        comment: Option<String>,
        to_principals: Option<Vec<String>>,
        except_principals: Option<Vec<String>>,
        for_securable_type: Option<String>,
        when_condition: Option<String>,
        match_columns: ::core::option::Option<::std::vec::Vec<PyMatchColumn>>,
        row_filter: ::core::option::Option<PyFunctionRef>,
        column_mask: ::core::option::Option<PyFunctionRef>,
        deny: ::core::option::Option<PyPrivilegeOptions>,
        grant: ::core::option::Option<PyPrivilegeOptions>,
        update_mask: Option<String>,
    ) -> PyResult<PyPolicyInfo> {
        let mut request = self.client.update();
        request = request.with_policy_type(policy_type.map(::core::convert::Into::into));
        request = request.with_comment(comment);
        if let Some(to_principals) = to_principals {
            request = request.with_to_principals(to_principals);
        }
        if let Some(except_principals) = except_principals {
            request = request.with_except_principals(except_principals);
        }
        request = request.with_for_securable_type(for_securable_type);
        request = request.with_when_condition(when_condition);
        if let Some(match_columns) = match_columns {
            request = request.with_match_columns(
                match_columns
                    .into_iter()
                    .map(::core::convert::Into::into)
                    .collect::<::std::vec::Vec<_>>(),
            );
        }
        request = request.with_row_filter(row_filter.map(::core::convert::Into::into));
        request = request.with_column_mask(column_mask.map(::core::convert::Into::into));
        request = request.with_deny(deny.map(::core::convert::Into::into));
        request = request.with_grant(grant.map(::core::convert::Into::into));
        request = request.with_update_mask(update_mask);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyPolicyInfo::from(result))
    }
    pub fn delete(&self, py: Python<'_>) -> PyResult<()> {
        let request = self.client.delete();
        block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(())
    }
}
impl PyPolicyClient {
    pub fn new(client: PolicyClient) -> Self {
        Self { client }
    }
}
