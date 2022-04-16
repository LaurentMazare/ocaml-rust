use std::cell::UnsafeCell;

const CUSTOM_OPERATIONS_IDENTIFIER: &str = "_custom_rust_box\0";

extern "C" fn finalize_box(v: ocaml_sys::Value) {
    unsafe {
        let b = ocaml_sys::field(v, 1);
        // TODO: properly drop the box here.
        // drop(Box::from_raw(b))
    }
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
    let boxed_t = Box::into_raw(Box::new(UnsafeCell::new(t)));
    let sys_value = unsafe {
        ocaml_sys::caml_alloc_custom(
            &CUSTOM_OPERATIONS_FOR_BOX,
            std::mem::size_of::<Box<T>>(),
            0,
            1,
        )
    };
    unsafe { ocaml_sys::store_field(sys_value, 1, boxed_t as isize) };
    unsafe { crate::Value::new(sys_value) }
}

pub fn get<'a, T: 'static>(v: crate::Value<'a, Box<T>>) -> &'a UnsafeCell<T> {
    let v = unsafe { ocaml_sys::field(v.value, 1) };
    unsafe { &*(v as *const Box<UnsafeCell<T>>) }
}
