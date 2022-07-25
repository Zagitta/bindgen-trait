use anyhow::Result;
use interface::*;
use wasmer::{Instance, Module, Store, WasmerEnv};

#[derive(Default, Clone)]
struct MyHostImpl {
    counter: i32,
}

#[async_trait::async_trait]
impl MyHostInterface for MyHostImpl {
    fn barfoo(&mut self, i: i32) -> i32 {
        self.counter += i;
        self.counter
    }
    async fn my_async_host_fn(&mut self) -> String {
        "hello".to_string()
    }
}

fn main() -> Result<()> {
    let wasm_bytes = std::fs::read("target/wasm32-unknown-unknown/release/module.wasm")?;
    let store = Store::default();
    let module = Module::new(&store, wasm_bytes)?;
    let (mut env, instance) = Runtime::new(MyHostImpl::default(), &store, &module)?;

    for _ in 0..3 {
        let res = env.foobar()?;

        println!("guest func returned: {:#}", res);
    }

    Ok(())
}
