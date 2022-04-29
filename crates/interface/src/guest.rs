use async_trait::async_trait;
use fp_bindgen_support::common::mem::FatPtr;
#[async_trait]
pub trait MyGuestInterface {
    fn foobar(&mut self) -> i32;
    async fn my_async_guest_fn(&mut self) -> String;
}
extern "Rust" {
    fn __fp_get_myguestinterface_impl() -> &'static mut dyn MyGuestInterface;
}

#[no_mangle]
pub static __fp_version_major: i32 = 1;

#[no_mangle]
fn __fp_gen_foobar() -> i32 {
    unsafe { __fp_get_myguestinterface_impl().foobar() }
}

#[link(wasm_import_module = "MyProtocol")]
extern "C" {
    fn __fp_gen_barfoo(i: i32) -> i32;
    fn __fp_gen_my_async_host_fn() -> FatPtr;
}
pub struct Host;
impl Host {
    pub fn barfoo(i: i32) -> i32 {
        unsafe { __fp_gen_barfoo(i) }
    }

    pub async fn my_async_host_fn() -> String {
        let ptr = unsafe { __fp_gen_my_async_host_fn() };

        let ret = unsafe {
            fp_bindgen_support::guest::io::import_value_from_host(
                fp_bindgen_support::guest::r#async::HostFuture::new(ptr).await,
            )
        };
        ret
    }
}
