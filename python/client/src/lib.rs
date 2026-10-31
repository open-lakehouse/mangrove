//! Python bindings of the Unity Catalog client, the extension module
//! `unitycatalog_client._client`.
//!
//! The module itself is generated (`codegen::extension_module`): the root client, one
//! submodule of model classes and resource clients per API, and the exception classes.
//! The pure-Python `unitycatalog_client` package re-exports it. The hand-written parts are
//! exported through `trestle.yaml` (`clients.python.options.exports`):
//!
//! - [`client::PyTemporaryCredentialClient`] — credential vending by securable name,
//! - [`reference::parse_uc_url`] — the UC URL parser behind `unitycatalog_client.obstore`,
//!
//! and [`error`] maps the client's errors onto the exception classes.

mod client;
mod codegen;
mod error;
mod reference;
