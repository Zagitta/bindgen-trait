use crate::common::{FutureStatus, GuestFuture, GuestPtr};
use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};
use wasmer::{
    imports, Exports, FromToNativeWasmType, Function, Global, HostEnvInitError, ImportObject,
    Instance, InstantiationError, LazyInit, Memory, MemoryView, Module, NativeFunc, RuntimeError,
    Store, WasmPtr, WasmerEnv,
};

unsafe impl FromToNativeWasmType for GuestPtr {
    type Native = i64;

    #[inline]
    fn from_native(native: Self::Native) -> Self {
        Self {
            ptr: ((native >> 32) & 0xFFFF) as u32,
            len: (native & 0xFFFF) as u32,
        }
    }

    #[inline]
    fn to_native(self) -> Self::Native {
        ((self.ptr as u64) << 32 | self.len as u64) as i64
    }
}

pub trait Protocol<P, T> {
    const NAME: &'static str;
    const VERSION: (u32, u32, u32);
    fn imports(env: &Arc<Mutex<Inner<P, T>>>, store: &Store) -> Exports;
}

#[derive(Default, Clone)]
pub struct Inner<P, T> {
    memory: LazyInit<Memory>,
    free_fn: LazyInit<NativeFunc<GuestPtr>>,
    malloc_fn: LazyInit<NativeFunc<u32, GuestPtr>>,
    guest_resolve_async_value_fn: LazyInit<NativeFunc<(GuestPtr, GuestPtr)>>,
    guest_version_major: LazyInit<Global>,

    pub imports: P,
    pub data: T,
}

impl<P, T> Inner<P, T>
where
    P: WasmerEnv + Default,
    T: Clone + Send + Sync,
{
    pub(crate) fn guest_malloc(&self, size: u32) -> Result<GuestPtr, RuntimeError> {
        unsafe { self.malloc_fn.get_unchecked().call(size) }
    }

    pub(crate) fn guest_free(&self, ptr: GuestPtr) -> Result<(), RuntimeError> {
        unsafe { self.free_fn.get_unchecked().call(ptr) }
    }

    //pub fn guest_resolve_async_value(&self, )

    pub(crate) fn host_resolve_async_value(&self) {}
}

impl<P, T> Protocol<P, T> for Inner<P, T>
where
    P: Protocol<P, T> + WasmerEnv + Default + 'static,
    T: Clone + Send + Sync + 'static,
{
    const NAME: &'static str = "fp";
    const VERSION: (u32, u32, u32) = (0, 1, 0);

    fn imports(env: &Arc<Mutex<Inner<P, T>>>, store: &Store) -> Exports {
        let mut exports = wasmer::Exports::new();
        exports.insert(
            "host_resolve_async_value",
            Function::new_native_with_env(&store, env.clone(), |env: &Arc<Mutex<Inner<P, T>>>| {
                env.lock().unwrap().host_resolve_async_value()
            }),
        );

        exports
    }
}

impl<P, T> wasmer::WasmerEnv for Inner<P, T>
where
    P: WasmerEnv + Default,
    T: Clone + Send + Sync,
{
    fn init_with_instance(
        &mut self,
        instance: &wasmer::Instance,
    ) -> Result<(), wasmer::HostEnvInitError> {
        self.memory.initialize(
            instance
                .exports
                .get_with_generics_weak::<Memory, _, _>("memory")?,
        );
        self.free_fn
            .initialize(instance.exports.get_native_function("__fp_free")?);
        self.malloc_fn
            .initialize(instance.exports.get_native_function("__fp_malloc")?);
        self.guest_resolve_async_value_fn.initialize(
            instance
                .exports
                .get_native_function("__fp_guest_resolve_async_value")?,
        );
        self.guest_version_major.initialize(
            instance
                .exports
                .get_with_generics_weak("__fp_version_major")?,
        );

        self.imports.init_with_instance(instance)
    }
}

#[derive(Clone, Default)]
#[doc(hidden)]
pub struct RuntimeImpl<P, T>
where
    P: Protocol<P, T> + WasmerEnv + Default + 'static,
    T: Clone + Send + Sync + 'static,
{
    pub inner: Arc<Mutex<Inner<P, T>>>,
}

impl<P, T> RuntimeImpl<P, T>
where
    P: Protocol<P, T> + WasmerEnv + Default + 'static,
    T: Clone + Send + Sync + 'static,
{
    pub fn new(
        data: T,
        store: &Store,
        module: &Module,
    ) -> Result<(Self, Instance), InstantiationError> {
        let mut this = Self {
            inner: Arc::new(Mutex::new(Inner {
                memory: Default::default(),
                free_fn: Default::default(),
                malloc_fn: Default::default(),
                guest_resolve_async_value_fn: Default::default(),
                guest_version_major: Default::default(),
                imports: P::default(),
                data,
            })),
        };
        let instance = Instance::new(module, &this.imports(store))?;
        this.inner.init_with_instance(&instance)?;

        Ok((this, instance))
    }

    pub fn imports(&self, store: &Store) -> ImportObject {
        let inner_imports = Inner::imports(&self.inner, store);
        let protocol_imports = P::imports(&self.inner, store);
        imports!(
            Inner::<P,T>::NAME => inner_imports,
            P::NAME => protocol_imports,
        )
    }

    //fn resolve_async_value(&self,)
}

pub trait GuestInterface: WasmerEnv {
    const NAME: &'static str;
    const VERSION: (u32, u32, u32);
}
#[derive(WasmerEnv, Clone, Default)]
struct BaseGuestInterface {
    memory: LazyInit<Memory>,
    free_fn: LazyInit<NativeFunc<GuestPtr>>,
    malloc_fn: LazyInit<NativeFunc<u32, GuestPtr>>,
    guest_resolve_async_value_fn: LazyInit<NativeFunc<(GuestPtr, GuestPtr)>>,
    guest_version_major: LazyInit<Global>,
    guest_version_minor: LazyInit<Global>,
    guest_version_patch: LazyInit<Global>,
}

impl BaseGuestInterface {
    pub fn get_protocol_version(&self) -> (u32, u32, u32) {
        let major = unsafe { self.guest_version_major.get_unchecked() }
            .get()
            .i32()
            .unwrap_or_default();
        let minor = unsafe { self.guest_version_minor.get_unchecked() }
            .get()
            .i32()
            .unwrap_or_default();
        let patch = unsafe { self.guest_version_patch.get_unchecked() }
            .get()
            .i32()
            .unwrap_or_default();

        (major as u32, minor as u32, patch as u32)
    }

    fn host_resolve_async_value(&self, guest_future_ptr: GuestPtr, result_ptr: GuestPtr) {
        let wasm_ptr = WasmPtr::<GuestFuture>::new(guest_future_ptr.ptr);
        let cell = wasm_ptr
            .deref(unsafe { self.memory.get_unchecked() })
            .unwrap();
        cell.set(GuestFuture {
            status: FutureStatus::Ready,
            result_ptr,
        });
    }
}
pub struct WasmerRuntime<I: GuestInterface> {
    instance: Instance,
    base_interface: BaseGuestInterface,
    pub guest_interface: I,
}

impl<I: GuestInterface> WasmerRuntime<I> {
    pub fn new(interface: I, store: &Store, module: &Module) -> Result<Self, InstantiationError> {
        let mut base_interface = BaseGuestInterface::default();

        let imports = imports! {
            "__fp" => {
                "host_resolve_async_value" => Function::new_native_with_env(store, base_interface.clone(), BaseGuestInterface::host_resolve_async_value)
            }
        };

        let instance = Instance::new(module, &imports)?;
        base_interface.init_with_instance(&instance)?;

        todo!()
    }
}
