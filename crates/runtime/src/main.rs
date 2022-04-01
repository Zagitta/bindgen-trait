use anyhow::Result;
use interface::*;
use wasmer::{Instance, Module, Store, WasmerEnv};

#[derive(Default, Clone)]
struct MyHostImpl {
    counter: i32,
}

impl MyHostInterface for MyHostImpl {
    fn barfoo(&mut self, i: i32) -> i32 {
        self.counter += i;
        self.counter
    }
}

fn main() -> Result<()> {
    let wasm_bytes = std::fs::read("target/wasm32-unknown-unknown/release/module.wasm")?;
    let store = Store::default();
    let module = Module::new(&store, wasm_bytes)?;

    let mut env = Runtime::new(MyHostImpl::default());
    let instance = Instance::new(&module, &env.imports(&store))?;

    env.init_with_instance(&instance)?;

    for _ in 0..3 {
        let res = env.foobar()?;

        println!("guest func returned: {:#}", res);
    }

    Ok(())
}
