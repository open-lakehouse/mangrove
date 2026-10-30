use self::client::{
    PyCatalogClient, PyCredentialClient, PyExternalLocationClient, PySchemaClient, PyTableClient,
    PyTagPolicyClient, PyTemporaryCredentialClient, PyUnityCatalogClient, PyVolumeClient,
};
use pyo3::prelude::*;
// The Python-visible model classes are the trestle-generated `Py*` wrapper
// `#[pyclass]` types (each carries `#[pyclass(name = "…")]` so the Python name is
// unchanged). buffa models are plain structs with no `#[pyclass]`, so unlike the
// old prost stack we register the wrappers here, not the bare model types.
use unitycatalog_common::models::{
    PyAzureManagedIdentity, PyAzureServicePrincipal, PyAzureStorageKey, PyCatalog, PyCatalogType,
    PyColumn, PyColumnTypeName, PyCredential, PyDataSourceFormat, PyExternalLocation,
    PyModelVersion, PyModelVersionStatus, PyPurpose, PyRegisteredModel, PySchema, PyTable,
    PyTableType, PyTagPolicy, PyTemporaryCredential, PyValue, PyVolume, PyVolumeType,
};

mod client;
mod codegen;
mod error;
mod reference;
mod runtime;

/// A Python module implemented in Rust.
///
/// This is exposed at `unitycatalog_client._client`; the public surface is
/// re-exported by the pure-Python `unitycatalog_client.__init__` so users
/// just `import unitycatalog_client`.
#[pymodule]
fn _client(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // exception types
    error::register_exceptions(m)?;

    // objects and enums (generated Py* wrappers; Python names preserved via
    // `#[pyclass(name = "…")]`)
    m.add_class::<PyCatalog>()?;
    m.add_class::<PyCatalogType>()?;
    m.add_class::<PyCredential>()?;
    m.add_class::<PyPurpose>()?;
    m.add_class::<PyAzureManagedIdentity>()?;
    m.add_class::<PyAzureServicePrincipal>()?;
    m.add_class::<PyAzureStorageKey>()?;
    m.add_class::<PyExternalLocation>()?;
    m.add_class::<PySchema>()?;
    m.add_class::<PyTable>()?;
    m.add_class::<PyTableType>()?;
    m.add_class::<PyColumn>()?;
    m.add_class::<PyColumnTypeName>()?;
    m.add_class::<PyDataSourceFormat>()?;
    m.add_class::<PyTemporaryCredential>()?;
    m.add_class::<PyVolume>()?;
    m.add_class::<PyVolumeType>()?;
    m.add_class::<PyTagPolicy>()?;
    m.add_class::<PyValue>()?;
    m.add_class::<PyRegisteredModel>()?;
    m.add_class::<PyModelVersion>()?;
    m.add_class::<PyModelVersionStatus>()?;

    // service clients
    m.add_class::<PyCatalogClient>()?;
    m.add_class::<PyCredentialClient>()?;
    m.add_class::<PyExternalLocationClient>()?;
    m.add_class::<PySchemaClient>()?;
    m.add_class::<PyTableClient>()?;
    m.add_class::<PyTagPolicyClient>()?;
    m.add_class::<PyTemporaryCredentialClient>()?;
    m.add_class::<PyUnityCatalogClient>()?;
    m.add_class::<PyVolumeClient>()?;

    // URL-parser helpers (shared with `unitycatalog-object-store` so the
    // Rust and Python URL surfaces stay in lock-step).
    m.add_function(wrap_pyfunction!(reference::parse_uc_url, m)?)?;

    Ok(())
}
