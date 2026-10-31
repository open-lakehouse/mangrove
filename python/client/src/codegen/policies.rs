// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::PolicyClient;
use unitycatalog_common::models::policies::v1::*;
use unitycatalog_common::models::*;

/// The requests on one `PolicyInfo`.
#[pyclass(name = "PolicyClient", module = "unitycatalog_client._client.policies")]
pub struct PyPolicyClient {
    pub(crate) client: PolicyClient,
}

#[pymethods]
impl PyPolicyClient {
    /// Get a policy
    ///
    /// Gets the policy that matches the supplied name, defined on the specified securable.
    ///
    /// Returns:
    ///     A row-filter or column-mask policy bound to principals and scoped to a securable. Policies
    ///     are
    ///     read as whole documents; enforcement is performed by the query engine, not by this service.
    ///     `when_condition` is stored and returned opaque — this service does not parse or evaluate it.
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<PyPolicyInfo> {
        let request = self.client.get();
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyPolicyInfo::from(result))
        })
    }

    /// Update a policy
    ///
    /// Updates the policy that matches the supplied name, defined on the specified securable.
    ///
    /// Args:
    ///     policy_type: The kind of enforcement this policy applies.
    ///     comment: User-provided free-form text description of the policy.
    ///     to_principals: The principals the policy applies to. Empty means "all principals".
    ///     except_principals: The principals explicitly excluded from the policy, even if matched by
    ///                        to_principals.
    ///     for_securable_type: The type of securables the policy takes effect on.
    ///     when_condition: An opaque condition expression evaluated by the query engine at enforcement
    ///                     time.
    ///     match_columns: The columns referenced by the policy function, and the aliases they are bound to.
    ///     row_filter: The row-filter function to invoke. Set only when policy_type is
    ///                 POLICY_TYPE_ROW_FILTER.
    ///     column_mask: The column-mask function to invoke. Set only when policy_type is
    ///                  POLICY_TYPE_COLUMN_MASK.
    ///     deny: The privileges to deny. Set only when policy_type is POLICY_TYPE_DENY.
    ///     grant: The privileges to grant. Set only when policy_type is POLICY_TYPE_GRANT.
    ///     update_mask: The fields to update, as a comma-separated list of field names. If empty or "*",
    ///                  every field set in the request is updated. Otherwise only the listed fields are
    ///                  updated, and a listed field that is unset in the request is cleared.
    ///
    /// Returns:
    ///     A row-filter or column-mask policy bound to principals and scoped to a securable. Policies
    ///     are
    ///     read as whole documents; enforcement is performed by the query engine, not by this service.
    ///     `when_condition` is stored and returned opaque — this service does not parse or evaluate it.
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
        py: Python,
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
    ) -> PyUnityCatalogResult<PyPolicyInfo> {
        let request = self
            .client
            .update()
            .with_policy_type(policy_type.map(::core::convert::Into::into))
            .with_comment(comment)
            .with_to_principals(to_principals.unwrap_or_default())
            .with_except_principals(except_principals.unwrap_or_default())
            .with_for_securable_type(for_securable_type)
            .with_when_condition(when_condition)
            .with_match_columns(
                match_columns
                    .unwrap_or_default()
                    .into_iter()
                    .map(::core::convert::Into::into)
                    .collect::<::std::vec::Vec<_>>(),
            )
            .with_row_filter(row_filter.map(::core::convert::Into::into))
            .with_column_mask(column_mask.map(::core::convert::Into::into))
            .with_deny(deny.map(::core::convert::Into::into))
            .with_grant(grant.map(::core::convert::Into::into))
            .with_update_mask(update_mask);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyPolicyInfo::from(result))
        })
    }

    /// Delete a policy
    ///
    /// Deletes the policy that matches the supplied name, defined on the specified securable.
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

impl PyPolicyClient {
    pub fn new(client: PolicyClient) -> Self {
        Self { client }
    }
}
