// @generated — do not edit by hand.
use crate::codegen::runtime::{ListIterator, block_on};
use crate::error::PyUnityCatalogError;
use futures::stream::StreamExt;
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::StagingTableClient;
use unitycatalog_common::models::staging_tables::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "StagingTableClient")]
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
