use std::cell::UnsafeCell;

use interface::*;
struct MyService {
    num: i32,
}

impl MyService {
    const fn new() -> MyService {
        MyService { num: 0 }
    }
}

impl MyGuestInterface for MyService {
    fn foobar(&mut self) -> i32 {
        self.num += Host::barfoo(1);
        self.num
    }
}

//register_impl!(MyService, MyGuestInterface); generates the following code:
static mut MY_SERVICE: UnsafeCell<MyService> = UnsafeCell::new(MyService::new());
#[no_mangle]
#[inline]
fn __fp_get_myguestinterface_impl() -> &'static mut dyn MyGuestInterface {
    unsafe { MY_SERVICE.get_mut() }
}
