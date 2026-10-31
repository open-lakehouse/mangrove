// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::StagingTableClient;
use unitycatalog_common::models::staging_tables::v1::*;
use unitycatalog_common::models::*;

/// The requests on one `StagingTable`.
#[pyclass(
    name = "StagingTableClient",
    module = "unitycatalog_client._client.staging_tables"
)]
pub struct PyStagingTableClient {
    pub(crate) client: StagingTableClient,
}

#[pymethods]
impl PyStagingTableClient {}
impl PyStagingTableClient {
    pub fn new(client: StagingTableClient) -> Self {
        Self { client }
    }
}
