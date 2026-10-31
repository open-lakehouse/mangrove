//! Python binding for [`unitycatalog_common::UCReference`].
//!
//! Re-using the Rust parser keeps the URL surface (schemes, kind segments,
//! case sensitivity, percent-decoding, error messages) consistent between
//! the `unitycatalog-object-store` Rust crate and the
//! `unitycatalog_client.obstore` Python helpers.

use std::collections::HashMap;

use pyo3::prelude::*;
use unitycatalog_common::UCReference;

use crate::codegen::error::PyUnityCatalogResult;

/// Parse a Unity Catalog URL into a `(kind, payload)` tuple.
///
/// `kind` is one of:
///
/// * ``"volume"`` — payload has `catalog`, `schema`, `volume`, `path` keys
///   (`path` is the empty string when the URL addresses the volume root).
/// * ``"table"``  — payload has `catalog`, `schema`, `table` keys.
/// * ``"path"``   — payload has a single `url` key carrying the raw cloud
///   URL string.
///
/// Raises a `UnityCatalogError` on invalid input.
#[pyfunction]
#[pyo3(name = "parse_uc_url", signature = (url, /))]
pub fn parse_uc_url(url: &str) -> PyUnityCatalogResult<(String, HashMap<String, String>)> {
    let reference = UCReference::parse(url).map_err(unitycatalog_client::Error::from)?;
    let (kind, payload) = match reference {
        UCReference::Volume {
            catalog,
            schema,
            volume,
            path,
        } => {
            let mut map = HashMap::with_capacity(4);
            map.insert("catalog".into(), catalog);
            map.insert("schema".into(), schema);
            map.insert("volume".into(), volume);
            map.insert("path".into(), path);
            ("volume", map)
        }
        UCReference::Table {
            catalog,
            schema,
            table,
        } => {
            let mut map = HashMap::with_capacity(3);
            map.insert("catalog".into(), catalog);
            map.insert("schema".into(), schema);
            map.insert("table".into(), table);
            ("table", map)
        }
        UCReference::Path(url) => {
            let mut map = HashMap::with_capacity(1);
            map.insert("url".into(), url.into());
            ("path", map)
        }
    };
    Ok((kind.to_string(), payload))
}
