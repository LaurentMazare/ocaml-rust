// OCaml C interface: https://ocaml.org/manual/intfc.html

/// An OCaml value, the lifetime mentioned in this value can either
/// come from:
/// - A [Gc] object that has allocated a new value.
/// - Decomposing another OCaml value.
#[derive(Clone, Copy)]
pub struct Value<'a, T>
where
    T: 'static,
{
    pub value: ocaml_sys::Value,
    phantom_data: std::marker::PhantomData<&'a T>,
}

pub unsafe fn new<'a, T>(value: ocaml_sys::Value) -> Value<'a, T> {
    Value { value, phantom_data: std::marker::PhantomData }
}

impl<'a, T> Value<'a, T> {
    pub unsafe fn new(value: ocaml_sys::Value) -> Value<'a, T> {
        new(value)
    }

    pub fn none() -> Value<'static, Option<T>> {
        unsafe { new(ocaml_sys::NONE) }
    }
}

pub fn unit() -> Value<'static, ()> {
    unsafe { new(ocaml_sys::UNIT) }
}

pub fn int(i: isize) -> Value<'static, isize> {
    unsafe { new(ocaml_sys::val_int(i)) }
}

pub fn bool(b: bool) -> Value<'static, bool> {
    let b = if b { 1 } else { 0 };
    unsafe { new(ocaml_sys::val_int(b)) }
}
