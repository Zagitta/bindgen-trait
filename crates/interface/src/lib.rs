pub struct FatPtr(u64);

impl From<String> for FatPtr {
    fn from(s: String) -> Self {
        FatPtr(s.as_ptr() as u64)
    }
}
pub trait MyGuestInterface {
    fn foobar(&mut self) -> i32;
}

pub trait MyHostInterface {
    fn barfoo(i: i32) -> i32;
}

#[cfg(feature = "guest")]
pub use guest::Host;

#[cfg(feature = "guest")]
mod guest {
    use super::{MyGuestInterface, MyHostInterface};
    extern "Rust" {
        fn __fp_get_myguestinterface_impl() -> &'static mut dyn MyGuestInterface;
    }

    #[no_mangle]
    fn __fp_gen_foobar() -> i32 {
        unsafe { __fp_get_myguestinterface_impl().foobar() }
    }

    pub struct Host;

    impl MyHostInterface for Host {
        fn barfoo(i: i32) -> i32 {
            #[link(wasm_import_module = "fp")]
            extern "C" {
                fn __fp_gen_barfoo(i: i32) -> i32;
            }
            unsafe { __fp_gen_barfoo(i) }
        }
    }
}
