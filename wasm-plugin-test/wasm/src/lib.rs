use common::wasm_plugin_framework::{plugin, abi};

plugin!(common, "PLUGIN 1");

use common::{A, B};

#[no_mangle]
pub extern "C" fn main(ptr: u32) -> u32 {
    println!("Hello world!");
    let a: A = abi::from_abi(ptr);
    println!("{:?}", a);
    let b = B {
        test: "AAAAAAAAAAAAAAAAAAAAA".into(),
        test2: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".into()
    };
    return abi::into_abi(&b);
}
