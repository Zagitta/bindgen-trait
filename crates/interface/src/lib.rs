pub struct FatPtr(u64);

impl From<String> for FatPtr {
    fn from(s: String) -> Self {
        FatPtr(s.as_ptr() as u64)
    }
}

#[cfg(feature = "guest")]
pub use guest::{Host, MyGuestInterface};

#[cfg(feature = "guest")]
mod guest {
    pub trait MyGuestInterface {
        fn foobar(&mut self) -> i32;
    }
    extern "Rust" {
        fn __fp_get_myguestinterface_impl() -> &'static mut dyn MyGuestInterface;
    }

    #[no_mangle]
    fn __fp_gen_foobar() -> i32 {
        unsafe { __fp_get_myguestinterface_impl().foobar() }
    }

    pub struct Host;
    impl Host {
        pub fn barfoo(i: i32) -> i32 {
            #[link(wasm_import_module = "fp")]
            extern "C" {
                fn __fp_gen_barfoo(i: i32) -> i32;
            }
            unsafe { __fp_gen_barfoo(i) }
        }
    }
}

#[cfg(feature = "host")]
pub use host::{MyGuestInterface, MyHostInterface, Runtime};

#[cfg(feature = "host")]
pub mod host {
    use std::sync::{Arc, Mutex};
    use wasmer::{
        imports, Function, ImportObject, LazyInit, Memory, NativeFunc, RuntimeError, Store,
        WasmerEnv,
    };

    // Notice that this is declared again as the host side calls can fail with a runtime error
    pub trait MyGuestInterface {
        fn foobar(&mut self) -> Result<i32, RuntimeError>;
    }

    pub trait MyHostInterface {
        fn barfoo(&mut self, i: i32) -> i32;
    }

    #[derive(WasmerEnv, Clone, Default)]
    #[doc(hidden)]
    pub struct RuntimeImpl<T>
    where
        T: Clone + Send + Sync,
    {
        #[wasmer(export)]
        pub(crate) memory: LazyInit<Memory>,
        #[wasmer(export)]
        __fp_gen_foobar: LazyInit<NativeFunc<(), i32>>,

        data: T,
    }

    pub type Runtime<T> = RuntimeImpl<Arc<Mutex<T>>>;

    impl<T: MyHostInterface + Sync + Send + Clone + Default + 'static> Runtime<T> {
        pub fn new(data: T) -> Self {
            Self {
                data: Arc::new(Mutex::new(data)),
                ..Default::default()
            }
        }

        pub fn imports<'a>(&'a self, store: &'a Store) -> ImportObject {
            imports! {
                "fp" => {
                    "__fp_gen_barfoo" => Function::new_native_with_env(store, self.clone(), Self::barfoo)
                }
            }
        }

        fn barfoo(&self, i: i32) -> i32 {
            self.data.lock().unwrap().barfoo(i)
        }
    }

    impl<T: Sync + Send + Default + Clone> MyGuestInterface for Runtime<T> {
        fn foobar(&mut self) -> Result<i32, RuntimeError> {
            self.__fp_gen_foobar_ref().unwrap().call()
        }
    }
}
