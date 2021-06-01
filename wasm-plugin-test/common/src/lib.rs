pub use wasm_plugin_framework;
use wasm_plugin_framework::common_plugin_implementation;

common_plugin_implementation!("API NAME", "0.1.0", Plugin);

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct A {
    pub test: String,
    pub test2: u64,
}

