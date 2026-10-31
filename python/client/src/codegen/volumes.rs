// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::VolumeClient;
use unitycatalog_common::models::volumes::v1::*;
use unitycatalog_common::models::*;

/// The requests on one `Volume`.
#[pyclass(name = "VolumeClient", module = "unitycatalog_client._client.volumes")]
pub struct PyVolumeClient {
    pub(crate) client: VolumeClient,
}

#[pymethods]
impl PyVolumeClient {
    /// Args:
    ///     include_browse: Whether to include volumes in the response for which the principal can only
    ///                     access selective metadata for
    ///
    /// Returns:
    ///     The requested resource
    #[pyo3(signature = (include_browse = None))]
    pub fn get(&self, py: Python, include_browse: Option<bool>) -> PyUnityCatalogResult<PyVolume> {
        let request = self.client.get().with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyVolume::from(result))
        })
    }

    /// Args:
    ///     new_name: New name for the volume.
    ///     comment: The comment attached to the volume
    ///     owner: The identifier of the user who owns the volume
    ///
    /// Returns:
    ///     The requested resource
    #[pyo3(signature = (new_name = None, comment = None, owner = None))]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        comment: Option<String>,
        owner: Option<String>,
    ) -> PyUnityCatalogResult<PyVolume> {
        let request = self
            .client
            .update()
            .with_new_name(new_name)
            .with_comment(comment)
            .with_owner(owner);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyVolume::from(result))
        })
    }

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

impl PyVolumeClient {
    pub fn new(client: VolumeClient) -> Self {
        Self { client }
    }
}
