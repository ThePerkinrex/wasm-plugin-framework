pub use wasm_plugin_framework;
use wasm_plugin_framework::common_plugin_implementation;

common_plugin_implementation!("API NAME", "0.1.0", Plugin,
    fn a(arg: A) -> B;
);

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct A {
    pub test: String,
    pub test2: u64,
}


#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct B {
    pub test: String,
    pub test2: String,
}

