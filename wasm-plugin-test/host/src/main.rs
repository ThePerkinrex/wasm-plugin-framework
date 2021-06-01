use wasmer::Exports;


fn main() -> anyhow::Result<()> {
    let bytes = include_bytes!("wasm.wasm");

    let p = common::Plugin::new(bytes);
    println!("API NAME: {}", common::metadata::API_NAME);
    println!("API VERSION: {}", common::metadata::API_VERSION);
    println!("PLUGIN NAME: {}", p.name);

    let e: &Exports = &(&p).instance.exports;

    for (name, extrn) in e.iter() {
        println!("{}: {:?}", name, extrn);
    }

    println!("{:?}", e.get_function("main").unwrap());

    let a = common::A { test: "Hey Ho".into(), test2: 1000000000000000000u64 };
    let ptr = common::wasm_plugin_framework::abi::into_abi(&p, &a);
    let r = e.get_function("main").unwrap().call(&[(ptr as i32).into()]).unwrap()[0].unwrap_i32() as u32;
    let b: common::B = common::wasm_plugin_framework::abi::from_abi(&p, r);
    println!("{:?}", b);
    Ok(())
}
