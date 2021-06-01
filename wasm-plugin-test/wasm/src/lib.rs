use common::wasm_plugin_framework::{plugin, abi};

plugin!(common, "PLUGIN 1");

use common::A;

#[no_mangle]
pub extern "C" fn main(ptr: u32) {
    println!("Hello world!");
    let a: A = abi::from_abi(ptr);
    println!("{:?}", a);
}
