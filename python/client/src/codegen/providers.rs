// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::ProviderClient;
use unitycatalog_common::models::providers::v1::*;
use unitycatalog_common::models::*;

/// The requests on one `Provider`.
#[pyclass(
    name = "ProviderClient",
    module = "unitycatalog_client._client.providers"
)]
pub struct PyProviderClient {
    pub(crate) client: ProviderClient,
}

#[pymethods]
impl PyProviderClient {
    /// Get a provider by name.
    ///
    /// Returns:
    ///     A provider represents an organization sharing data with this metastore. A provider is the
    ///     inbound counterpart of a recipient: it is registered from a share activation/ credential
    ///     file
    ///     and is used to access shares offered by an upstream Delta Sharing server.
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<PyProvider> {
        let request = self.client.get();
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyProvider::from(result))
        })
    }

    /// Update a provider.
    ///
    /// Args:
    ///     new_name: New name for the provider.
    ///     owner: Username of the provider owner.
    ///     comment: Description about the provider.
    ///     recipient_profile_str: The recipient profile (credential file contents) used to connect to
    ///                            the sharing server. Required when authentication_type is TOKEN or
    ///                            OAUTH_CLIENT_CREDENTIALS.
    ///     properties: Provider properties as map of string key-value pairs. When provided in update
    ///                 request, the specified properties will override the existing properties. To add and
    ///                 remove properties, one would need to perform a read-modify-write.
    ///
    /// Returns:
    ///     A provider represents an organization sharing data with this metastore. A provider is the
    ///     inbound counterpart of a recipient: it is registered from a share activation/ credential
    ///     file
    ///     and is used to access shares offered by an upstream Delta Sharing server.
    #[pyo3(
        signature = (
            new_name = None,
            owner = None,
            comment = None,
            recipient_profile_str = None,
            properties = None
        )
    )]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        owner: Option<String>,
        comment: Option<String>,
        recipient_profile_str: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<PyProvider> {
        let request = self
            .client
            .update()
            .with_new_name(new_name)
            .with_owner(owner)
            .with_comment(comment)
            .with_recipient_profile_str(recipient_profile_str)
            .with_properties(properties.unwrap_or_default());
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyProvider::from(result))
        })
    }

    /// Delete a provider.
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

impl PyProviderClient {
    pub fn new(client: ProviderClient) -> Self {
        Self { client }
    }
}
