// Use two Box indirections here, the first Box is leaked to result in a ptr.
// The second box is used to get a fat pointer so that the proper destructor
// can be called in the finalizer.
// This is very close to the ocaml-interop implementation.
// https://github.com/tezedge/ocaml-interop/blob/265773e1d73585aad73ee579ae80c0e6b5fb4c57/src/memory.rs#L197
use std::cell::UnsafeCell;

const CUSTOM_OPERATIONS_IDENTIFIER: &str = "_custom_rust_box\0";

extern "C" fn finalize_box(v: ocaml_sys::Value) {
    let v = unsafe { *ocaml_sys::field(v, 1) as *mut Box<dyn std::any::Any> };
    drop(unsafe { Box::from_raw(v) })
}

const CUSTOM_OPERATIONS_FOR_BOX: ocaml_sys::custom_operations = ocaml_sys::custom_operations {
    identifier: CUSTOM_OPERATIONS_IDENTIFIER.as_ptr() as *const ocaml_sys::Char,
    finalize: Some(finalize_box),
    compare: None,
    hash: None,
    serialize: None,
    deserialize: None,
    compare_ext: None,
    fixed_length: std::ptr::null(),
};

pub fn new<'a, T: 'static>(_gc: &'a mut crate::gc::Gc, t: T) -> crate::Value<'a, Box<T>> {
    let box_: Box<Box<dyn std::any::Any>> = Box::new(Box::new(UnsafeCell::new(t)));
    let boxed_t = Box::into_raw(box_);
    let sys_value = unsafe {
        ocaml_sys::caml_alloc_custom(
            &CUSTOM_OPERATIONS_FOR_BOX,
            std::mem::size_of::<Box<Box<dyn std::any::Any>>>(),
            0,
            1,
        )
    };
    let ptr = unsafe { ocaml_sys::field(sys_value, 1) } as *mut _;
    unsafe { std::ptr::write(ptr, boxed_t) };
    unsafe { crate::Value::new(sys_value) }
}

pub fn get<'a, T: 'static>(v: crate::Value<'a, Box<T>>) -> &'a UnsafeCell<T> {
    let v = unsafe { *ocaml_sys::field(v.value, 1) as *mut Box<dyn std::any::Any> };
    unsafe { &**v }.downcast_ref::<UnsafeCell<T>>().expect("unexpected box content")
}

use std::sync::{Arc, Mutex};

/// A wrapped Rust value protected by a mutex and seen as an abstract type
/// from OCaml.
pub struct Custom<T> {
    _inner: Arc<Mutex<T>>,
}

impl<T> Custom<T> {
    pub fn new(t: T) -> Self {
        Custom { _inner: Arc::new(Mutex::new(t)) }
    }

    pub fn inner(&self) -> &Mutex<T> {
        &self._inner
    }
}

impl<T: 'static> crate::to_value::ToValue for Custom<T> {
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        let box_: Box<Box<dyn std::any::Any>> = Box::new(Box::new(self._inner.clone()));
        let boxed_t = Box::into_raw(box_);
        let sys_value = unsafe {
            ocaml_sys::caml_alloc_custom(
                &CUSTOM_OPERATIONS_FOR_BOX,
                std::mem::size_of::<Box<Box<dyn std::any::Any>>>(),
                0,
                1,
            )
        };
        let ptr = unsafe { ocaml_sys::field(sys_value, 1) } as *mut _;
        unsafe { std::ptr::write(ptr, boxed_t) };
        pin(sys_value)
    }
}

impl<T: 'static> crate::from_value::FromSysValue for Custom<T> {
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        let v = *ocaml_sys::field(v, 1) as *mut Box<dyn std::any::Any>;
        let inner = { &**v }.downcast_ref::<Arc<Mutex<T>>>().expect("unexpected box content");
        Custom { _inner: inner.clone() }
    }
}
