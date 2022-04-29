use async_trait::async_trait;
use lib::host::{Inner, RuntimeImpl};
use std::sync::{Arc, Mutex};
use wasmer::{Function, LazyInit, NativeFunc, RuntimeError, Store, WasmerEnv};

// Notice that this is declared again as the host side calls can fail with a runtime error
#[async_trait]
pub trait MyGuestInterface {
    fn foobar(&mut self) -> Result<i32, RuntimeError>;
    async fn my_async_guest_fn(&mut self) -> String;
}

#[async_trait]
pub trait MyHostInterface {
    fn barfoo(&mut self, i: i32) -> i32;
    async fn my_async_host_fn(&mut self) -> String;
}

#[derive(WasmerEnv, Clone, Default)]
#[doc(hidden)]
pub struct Protocol {
    #[wasmer(export)]
    pub(crate) __fp_gen_foobar: LazyInit<NativeFunc<(), i32>>,
}

impl<T: MyHostInterface + Clone + Send + Sync + 'static> lib::host::Protocol<Protocol, T>
    for Protocol
{
    const NAME: &'static str = "MyProtocol";
    const VERSION: (u32, u32, u32) = (0, 1, 0);

    fn imports(env: &Arc<Mutex<Inner<Protocol, T>>>, store: &Store) -> wasmer::Exports {
        let mut exports = wasmer::Exports::new();
        exports.insert(
            "__fp_gen_barfoo",
            Function::new_native_with_env(
                &store,
                env.clone(),
                |env: &Arc<Mutex<Inner<Protocol, T>>>, i: i32| env.lock().unwrap().data.barfoo(i),
            ),
        );
        exports.insert(
            "__fp_gen_my_async_host_fn",
            Function::new_native_with_env(
                &store,
                env.clone(),
                |env: &Arc<Mutex<Inner<Protocol, T>>>| todo!(),
            ),
        );

        exports
    }
}

pub type Runtime<T> = RuntimeImpl<Protocol, T>;

#[async_trait]
impl<T: MyHostInterface + Sync + Send + Default + Clone> MyGuestInterface for Runtime<T> {
    fn foobar(&mut self) -> Result<i32, RuntimeError> {
        self.inner
            .lock()
            .unwrap()
            .imports
            .__fp_gen_foobar_ref()
            .ok_or_else(|| RuntimeError::new("__fp_gen_foobar export not found"))?
            .call()
    }
    async fn my_async_guest_fn(&mut self) -> String {
        todo!()
    }
}
