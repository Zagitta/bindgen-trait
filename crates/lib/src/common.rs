use wasmer::ValueType;

pub type FatPtr = u64;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GuestPtr {
    pub(crate) ptr: u32,
    pub(crate) len: u32,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum FutureStatus {
    Pending = 0,
    Ready = 1,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GuestFuture {
    pub(crate) status: FutureStatus,
    pub(crate) result_ptr: GuestPtr,
}

unsafe impl ValueType for GuestFuture {}

impl GuestFuture {
    pub fn new() -> Self {
        todo!()
    }
}
