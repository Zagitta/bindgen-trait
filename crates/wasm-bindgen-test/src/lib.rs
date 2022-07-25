use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Foo {
    internal: i32,
    whatever: String,
}

#[wasm_bindgen]
impl Foo {
    #[wasm_bindgen(constructor)]
    pub fn new(val: i32) -> Foo {
        Foo {
            internal: val,
            whatever: format!("hello"),
        }
    }

    pub fn get(&self) -> i32 {
        self.internal
    }

    pub fn set(&mut self, val: i32) {
        self.internal = val;
    }
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) -> Foo {
    let foo = format!("Hello, {}!", name);
    alert(&foo);

    let asd: <Foo as wasm_bindgen::convert::ReturnWasmAbi>::Abi;

    Foo::new(foo.len() as i32)
}
