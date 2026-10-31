// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::RecipientClient;
use unitycatalog_common::models::recipients::v1::*;
use unitycatalog_common::models::*;

/// The requests on one `Recipient`.
#[pyclass(
    name = "RecipientClient",
    module = "unitycatalog_client._client.recipients"
)]
pub struct PyRecipientClient {
    pub(crate) client: RecipientClient,
}

#[pymethods]
impl PyRecipientClient {
    /// Get a recipient by name.
    ///
    /// Returns:
    ///     The requested resource
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<PyRecipient> {
        let request = self.client.get();
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyRecipient::from(result))
        })
    }

    /// Update a recipient.
    ///
    /// Args:
    ///     new_name: New name for the recipient
    ///     owner: Username of the recipient owner.
    ///     comment: Description about the recipient.
    ///     properties_kvpairs: Recipient properties as map of string key-value pairs. When provided
    ///                         in update request, the specified properties will override the existing
    ///                         properties. To add and remove properties, one would need to perform a read-
    ///                         modify-write.
    ///     expiration_time: Expiration timestamp of the token, in epoch milliseconds.
    ///
    /// Returns:
    ///     The requested resource
    #[pyo3(
        signature = (
            new_name = None,
            owner = None,
            comment = None,
            properties_kvpairs = None,
            expiration_time = None
        )
    )]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        owner: Option<String>,
        comment: Option<String>,
        properties_kvpairs: ::core::option::Option<PySecurablePropertiesKvPairs>,
        expiration_time: Option<i64>,
    ) -> PyUnityCatalogResult<PyRecipient> {
        let request = self
            .client
            .update()
            .with_new_name(new_name)
            .with_owner(owner)
            .with_comment(comment)
            .with_properties_kvpairs(properties_kvpairs.map(::core::convert::Into::into))
            .with_expiration_time(expiration_time);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyRecipient::from(result))
        })
    }

    /// Delete a recipient.
    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let request = self.client.delete();
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }

    /// Rotate a recipient's token: issue a new token (and activation link) and retire the existing one.
    ///
    /// Args:
    ///     existing_token_expire_in_seconds: Seconds until the existing token expires. It can only shorten
    ///                                       the existing token's lifetime, never extend it; 0 expires
    ///                                       it immediately.
    ///
    /// Returns:
    ///     The requested resource
    #[pyo3(signature = (existing_token_expire_in_seconds = None))]
    pub fn rotate_recipient_token(
        &self,
        py: Python,
        existing_token_expire_in_seconds: Option<i64>,
    ) -> PyUnityCatalogResult<PyRecipient> {
        let request = self
            .client
            .rotate_recipient_token()
            .with_existing_token_expire_in_seconds(existing_token_expire_in_seconds);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyRecipient::from(result))
        })
    }
}

impl PyRecipientClient {
    pub fn new(client: RecipientClient) -> Self {
        Self { client }
    }
}
