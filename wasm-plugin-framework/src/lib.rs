pub use wasm_plugin_framework_macros::*;
#[cfg(not(target_arch = "wasm32"))]
pub use wasmer;
#[cfg(not(target_arch = "wasm32"))]
pub use wasmer_wasi;
#[doc(hidden)]
pub mod abi;
