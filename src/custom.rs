// Use two Box indirections here, the first Box is leaked to result in a ptr.
// The second box is used to get a fat pointer so that the proper destructor
// can be called in the finalizer.
// This is very close to the ocaml-interop implementation.
// https://github.com/tezedge/ocaml-interop/blob/265773e1d73585aad73ee579ae80c0e6b5fb4c57/src/memory.rs#L197

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

use std::sync::{Arc, Mutex};

/// A wrapped Rust value protected by a mutex and seen as an abstract type
/// from OCaml.
pub struct Custom<T> {
    _inner: Arc<Mutex<T>>,
}

/// A wrapped Rust value that can only be accessed via a non-mutable reference.
pub struct CustomConst<T> {
    // It should be possible to get rid of this const if [to_value] was consuming
    // its argument.
    _inner: Arc<T>,
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
    fn to_value(&self) -> ocaml_sys::Value {
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
        sys_value
    }
}

impl<T: 'static> crate::from_value::FromSysValue for Custom<T> {
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        let v = *ocaml_sys::field(v, 1) as *mut Box<dyn std::any::Any>;
        let inner = { &**v }.downcast_ref::<Arc<Mutex<T>>>().expect("unexpected box content");
        Custom { _inner: inner.clone() }
    }
}

impl<T> CustomConst<T> {
    pub fn new(t: T) -> Self {
        CustomConst { _inner: Arc::new(t) }
    }

    pub fn inner(&self) -> &T {
        self._inner.as_ref()
    }
}

impl<T: 'static> crate::to_value::ToValue for CustomConst<T> {
    fn to_value(&self) -> ocaml_sys::Value {
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
        sys_value
    }
}

impl<T: 'static> crate::from_value::FromSysValue for CustomConst<T> {
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        let v = *ocaml_sys::field(v, 1) as *mut Box<dyn std::any::Any>;
        let inner = { &**v }.downcast_ref::<Arc<T>>().expect("unexpected box content");
        CustomConst { _inner: inner.clone() }
    }
}
