//! Internal (`pub(crate)`) physical operators and the per-file `Load` leaf provider the SSA
//! compiler splices into a compiled plan. None of these are public entry points — the crate's one
//! public, table-level provider is [`crate::DeltaSsaTableProvider`].

mod field_id_adapter;
mod file_listing;
mod load_exec;
mod load_helpers;
mod load_provider;

pub(crate) use field_id_adapter::FieldIdPhysicalExprAdapterFactory;
pub(crate) use file_listing::FileListingExec;
pub(crate) use load_exec::LoadExec;
pub(crate) use load_provider::LoadTableProvider;
