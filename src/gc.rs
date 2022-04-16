use crate::value::Value;

/// An empty object used to control access to the OCaml runtime
/// functions that could allocate.
/// These allocating functions require a mutable reference to this
/// object, whereas non-allocating functions require a non-mutable
/// reference. The lifetime associated to this reference is also
/// passed to the generated OCaml object. This ensures that an allocating
/// function cannot be called in the presence of an OCaml value that
/// has been created with this Gc object, this would be unsafe as the
/// allocating call could result in a GC cycle that moves the OCaml
/// value around.
pub struct Gc();

/// Run the function f given a reference to a gc object.
/// Only a single gc reference should be acquired this way, though
/// this is not encoded via the type system.
pub fn with_gc<F, T>(f: F) -> T
where
    F: FnOnce(&mut Gc) -> T,
{
    f(&mut Gc())
}

pub fn tuple2<'a, T1, T2>(
    _gc: &'a mut Gc,
    v1: Value<'a, T1>,
    v2: Value<'a, T2>,
) -> Value<'a, (T1, T2)> {
    let v = unsafe { ocaml_sys::caml_alloc_tuple(2) };
    unsafe { ocaml_sys::store_field(v, 0, v1.value) };
    unsafe { ocaml_sys::store_field(v, 1, v2.value) };
    unsafe { Value::new(v) }
}

pub fn tuple3<'a, T1, T2, T3>(
    _gc: &'a mut Gc,
    v1: Value<'a, T1>,
    v2: Value<'a, T2>,
    v3: Value<'a, T3>,
) -> Value<'a, (T1, T2, T3)> {
    let v = unsafe { ocaml_sys::caml_alloc_tuple(3) };
    unsafe { ocaml_sys::store_field(v, 0, v1.value) };
    unsafe { ocaml_sys::store_field(v, 1, v2.value) };
    unsafe { ocaml_sys::store_field(v, 2, v3.value) };
    unsafe { Value::new(v) }
}

pub fn tuple4<'a, T1, T2, T3, T4>(
    _gc: &'a mut Gc,
    v1: Value<'a, T1>,
    v2: Value<'a, T2>,
    v3: Value<'a, T3>,
    v4: Value<'a, T4>,
) -> Value<'a, (T1, T2, T3, T4)> {
    let v = unsafe { ocaml_sys::caml_alloc_tuple(4) };
    unsafe { ocaml_sys::store_field(v, 0, v1.value) };
    unsafe { ocaml_sys::store_field(v, 1, v2.value) };
    unsafe { ocaml_sys::store_field(v, 2, v3.value) };
    unsafe { ocaml_sys::store_field(v, 3, v4.value) };
    unsafe { Value::new(v) }
}

pub fn array<'a, T>(_gc: &'a mut Gc, vs: Vec<Value<'a, T>>) -> Value<'a, Vec<T>> {
    let array_v = unsafe { ocaml_sys::caml_alloc_tuple(vs.len()) };
    vs.iter()
        .enumerate()
        .for_each(|(idx, v)| unsafe { ocaml_sys::store_field(array_v, idx, v.value) });
    unsafe { Value::new(array_v) }
}

pub fn string<'a, T>(_gc: &'a mut Gc, str: &T) -> Value<'a, String>
where
    T: AsRef<str>,
{
    let str = str.as_ref();
    let v = unsafe { ocaml_sys::caml_alloc_string(str.len()) };
    let content_ptr = unsafe { ocaml_sys::string_val(v) };
    unsafe { std::ptr::copy_nonoverlapping(str.as_ptr(), content_ptr, str.len()) };
    unsafe { Value::new(v) }
}

pub fn double(_gc: &mut Gc, f: f64) -> Value<f64> {
    let v = unsafe { ocaml_sys::caml_copy_double(f) };
    unsafe { Value::new(v) }
}

pub fn int64(_gc: &mut Gc, i: i64) -> Value<i64> {
    let v = unsafe { ocaml_sys::caml_copy_int64(i) };
    unsafe { Value::new(v) }
}
