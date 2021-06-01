use common::wasm_plugin_framework::{plugin, abi};

plugin!(common, "PLUGIN 1",
    fn a(a: A) -> B {
        println!("{:?}", a);
        let b = B {
            test: "AAAAAAAAAAAAAAAAAAAAA".into(),
            test2: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".into()
        };
        b
    }
);

use common::{A, B};
