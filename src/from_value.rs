use crate::value::Value;
pub trait FromSysValue: Sized {
    #[doc(hidden)]
    unsafe fn from_value(v: ocaml_sys::Value) -> Self;
}

pub trait FromValue: Sized {
    fn from_value(v: &Value<Self>) -> Self;
}

impl FromSysValue for isize {
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        ocaml_sys::int_val(v)
    }
}

impl FromSysValue for bool {
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        ocaml_sys::int_val(v) != 0
    }
}

impl FromSysValue for f64 {
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        check_tag("double", v, ocaml_sys::DOUBLE);
        *(v as *const f64)
    }
}

impl FromSysValue for i64 {
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        check_tag("i64", v, ocaml_sys::CUSTOM);
        let v = ocaml_sys::field(v, 1);
        *(v as *const i64)
    }
}

impl FromSysValue for Vec<u8> {
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        let len = ocaml_sys::caml_string_length(v);
        let start_ptr = ocaml_sys::string_val(v);
        std::slice::from_raw_parts(start_ptr, len).to_vec()
    }
}

impl FromSysValue for String {
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        let len = ocaml_sys::caml_string_length(v);
        let start_ptr = ocaml_sys::string_val(v);
        let slice = std::slice::from_raw_parts(start_ptr, len);
        String::from_utf8_lossy(slice).into_owned()
    }
}

#[doc(hidden)]
pub unsafe fn check_tag(kind: &str, v: ocaml_sys::Value, expected: u8) {
    if ocaml_sys::is_long(v) {
        let v = ocaml_sys::int_val(v);
        panic!("expected a block, got a long {}", v)
    }
    let tag = ocaml_sys::tag_val(v);
    if tag != expected {
        panic!("unexpected tag for {}, {} <> {}", kind, tag, expected)
    }
}

unsafe fn check_tuple(v: ocaml_sys::Value, expected_len: usize) {
    check_tag("tuple", v, 0);
    let len = ocaml_sys::wosize_val(v);
    if len != expected_len {
        panic!("unexpected length for tuple, {} <> {}", len, expected_len)
    }
}

impl<T1, T2> FromSysValue for (T1, T2)
where
    T1: FromSysValue,
    T2: FromSysValue,
{
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        check_tuple(v, 2);
        let t1 = ocaml_sys::field(v, 0);
        let t2 = ocaml_sys::field(v, 1);
        let t1: T1 = FromSysValue::from_value(*t1);
        let t2: T2 = FromSysValue::from_value(*t2);
        (t1, t2)
    }
}

impl<T1, T2, T3> FromSysValue for (T1, T2, T3)
where
    T1: FromSysValue,
    T2: FromSysValue,
    T3: FromSysValue,
{
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        check_tuple(v, 3);
        let t1 = ocaml_sys::field(v, 0);
        let t2 = ocaml_sys::field(v, 1);
        let t3 = ocaml_sys::field(v, 2);
        let t1: T1 = FromSysValue::from_value(*t1);
        let t2: T2 = FromSysValue::from_value(*t2);
        let t3: T3 = FromSysValue::from_value(*t3);
        (t1, t2, t3)
    }
}

impl<T1, T2, T3, T4> FromSysValue for (T1, T2, T3, T4)
where
    T1: FromSysValue,
    T2: FromSysValue,
    T3: FromSysValue,
    T4: FromSysValue,
{
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        check_tuple(v, 4);
        let t1 = ocaml_sys::field(v, 0);
        let t2 = ocaml_sys::field(v, 1);
        let t3 = ocaml_sys::field(v, 2);
        let t4 = ocaml_sys::field(v, 2);
        let t1: T1 = FromSysValue::from_value(*t1);
        let t2: T2 = FromSysValue::from_value(*t2);
        let t3: T3 = FromSysValue::from_value(*t3);
        let t4: T4 = FromSysValue::from_value(*t4);
        (t1, t2, t3, t4)
    }
}

impl FromSysValue for Vec<f64> {
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        let tag = ocaml_sys::tag_val(v);
        if tag == 0 {
            let len = ocaml_sys::wosize_val(v);
            let mut vs = Vec::new();
            for idx in 0..len {
                let t = ocaml_sys::field(v, idx);
                vs.push(FromSysValue::from_value(*t));
            }
            vs
        } else if tag == ocaml_sys::DOUBLE_ARRAY {
            let len = ocaml_sys::wosize_val(v);
            let mut vs = Vec::new();
            for idx in 0..len {
                let t = ocaml_sys::field(v, idx);
                vs.push(*(t as *const f64))
            }
            vs
        } else {
            panic!("unexpected tag for double array, {}", tag)
        }
    }
}

// The need for this hack will be removed once trait specialization
// is stable.
// https://rust-lang.github.io/rfcs/1210-impl-specialization.html
pub trait NotF64 {}

impl NotF64 for i64 {}
impl NotF64 for String {}
impl NotF64 for isize {}
impl NotF64 for () {}
impl<T> NotF64 for Vec<T> {}
impl<T> NotF64 for Option<T> {}
impl<T1> NotF64 for (T1,) {}
impl<T1, T2> NotF64 for (T1, T2) {}
impl<T1, T2, T3> NotF64 for (T1, T2, T3) {}
impl<T1, T2, T3, T4> NotF64 for (T1, T2, T3, T4) {}

impl<T> FromSysValue for Vec<T>
where
    T: FromSysValue + NotF64,
{
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        check_tag("array", v, 0);
        let len = ocaml_sys::wosize_val(v);
        let mut vs = Vec::new();
        for idx in 0..len {
            let t = ocaml_sys::field(v, idx);
            vs.push(FromSysValue::from_value(*t));
        }
        vs
    }
}

impl<T> FromSysValue for Option<T>
where
    T: FromSysValue,
{
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        if v == ocaml_sys::NONE {
            None
        } else {
            check_tag("option-some", v, ocaml_sys::TAG_SOME);
            let t = ocaml_sys::field(v, 0);
            Some(T::from_value(*t))
        }
    }
}

impl<T, E> FromSysValue for Result<T, E>
where
    T: FromSysValue,
    E: FromSysValue,
{
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        match ocaml_sys::tag_val(v) {
            0 => {
                let t = ocaml_sys::field(v, 0);
                Ok(T::from_value(*t))
            }
            1 => {
                let t = ocaml_sys::field(v, 0);
                Err(E::from_value(*t))
            }
            tag => panic!("unexpected tag for Result {}", tag),
        }
    }
}

impl<T> FromValue for T
where
    T: FromSysValue,
{
    fn from_value(v: &Value<T>) -> T {
        unsafe { FromSysValue::from_value(v.value) }
    }
}

// impl<T> FromSysValue for &'a std::cell::UnsafeCell<T> {
//     unsafe fn from_value(v: ocaml_sys::Value) -> Self {
//         let v: Value<Box<T>> = Value::new(v);
//         crate::custom::get(v)
//     }
// }
