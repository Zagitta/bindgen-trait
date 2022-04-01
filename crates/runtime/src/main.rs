use anyhow::Result;
use wasmer::{imports, Instance, Module, Store};

fn main() -> Result<()> {
    let wasm_bytes = std::fs::read("target/wasm32-unknown-unknown/release/module.wasm")?;
    let store = Store::default();
    let module = Module::new(&store, wasm_bytes)?;

    let import_object = imports! {};
    let instance = Instance::new(&module, &import_object)?;

    println!("{:#?}", instance.exports);

    let func = instance
        .exports
        .get_native_function::<(), i32>("__fp_gen_foobar")?;

    for _ in 0..3 {
        let res = func.call()?;

        println!("guest func returned: {:#x}", res);
    }

    Ok(())
}
