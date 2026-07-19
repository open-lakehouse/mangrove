mod _gen;
mod association;
mod error;
mod functions_ext;
// The `Object` / `ResourceName` layer is backed by the resource-store crate
// (`olai-store`), which is native-only; gate it behind the `store` feature so the
// wire-DTO models compile without it (e.g. on wasm).
#[cfg(feature = "store")]
mod object;
#[cfg(feature = "store")]
mod resources;

pub use _gen::*;
pub use association::AssociationLabel;
pub use error::ErrorResponse;
#[cfg(feature = "store")]
pub use object::Object;
#[cfg(feature = "store")]
pub use resources::*;
