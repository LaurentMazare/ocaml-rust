use crate::RootedValue;

pub trait ToValue: Sized {
    /// It's the responsability of callers to `to_value` to immediately make the
    /// returned value reachable from an OCaml root, for instance by using
    /// `crate::RootedValue::create`, or by setting it in a field of a reachable
    /// object.
    fn to_value(&self) -> ocaml_sys::Value;
}

pub fn to_rooted_value<T>(t: &T) -> crate::RootedValue<T>
where
    T: ToValue,
{
    crate::RootedValue::create(t.to_value())
}

impl ToValue for () {
    fn to_value(&self) -> ocaml_sys::Value {
        ocaml_sys::UNIT
    }
}

impl ToValue for i32 {
    fn to_value(&self) -> ocaml_sys::Value {
        unsafe { ocaml_sys::caml_copy_int32(*self) }
    }
}

impl ToValue for i64 {
    fn to_value(&self) -> ocaml_sys::Value {
        unsafe { ocaml_sys::caml_copy_int64(*self) }
    }
}

impl ToValue for f32 {
    fn to_value(&self) -> ocaml_sys::Value {
        unsafe { ocaml_sys::caml_copy_double(*self as f64) }
    }
}

impl ToValue for f64 {
    fn to_value(&self) -> ocaml_sys::Value {
        unsafe { ocaml_sys::caml_copy_double(*self) }
    }
}

impl ToValue for isize {
    fn to_value(&self) -> ocaml_sys::Value {
        unsafe { ocaml_sys::val_int(*self) }
    }
}

impl ToValue for usize {
    fn to_value(&self) -> ocaml_sys::Value {
        unsafe { ocaml_sys::val_int(*self as isize) }
    }
}

impl ToValue for bool {
    fn to_value(&self) -> ocaml_sys::Value {
        let v = if *self { 1 } else { 0 };
        unsafe { ocaml_sys::val_int(v) }
    }
}

impl<T1, T2> ToValue for (T1, T2)
where
    T1: ToValue,
    T2: ToValue,
{
    fn to_value(&self) -> ocaml_sys::Value {
        let (v1, v2) = self;
        let t = unsafe { ocaml_sys::caml_alloc_tuple(2) };
        let rv: RootedValue<()> = RootedValue::create(t);
        let v1 = T1::to_value(v1);
        unsafe { ocaml_sys::store_field(rv.value().value, 0, v1) };
        let v2 = T2::to_value(v2);
        unsafe { ocaml_sys::store_field(rv.value().value, 1, v2) };
        rv.value().value
    }
}

impl<T1, T2, T3> ToValue for (T1, T2, T3)
where
    T1: ToValue,
    T2: ToValue,
    T3: ToValue,
{
    fn to_value(&self) -> ocaml_sys::Value {
        let (v1, v2, v3) = self;
        let t = unsafe { ocaml_sys::caml_alloc_tuple(3) };
        let rv: RootedValue<()> = RootedValue::create(t);
        let v1 = T1::to_value(v1);
        unsafe { ocaml_sys::store_field(rv.value().value, 0, v1) };
        let v2 = T2::to_value(v2);
        unsafe { ocaml_sys::store_field(rv.value().value, 1, v2) };
        let v3 = T3::to_value(v3);
        unsafe { ocaml_sys::store_field(rv.value().value, 2, v3) };
        rv.value().value
    }
}

// We use a macro rather than a trait to define ToValue for
// Vec<f32> and Vec<f64> as a trait would conflict with NotF64.
macro_rules! vec_float {
    ($ty:ty) => {
        impl ToValue for Vec<$ty> {
            fn to_value(&self) -> ocaml_sys::Value {
                let len = self.len();
                let rv: RootedValue<()> =
                    RootedValue::create(unsafe { ocaml_sys::caml_alloc_float_array(len) });
                for (i, &v) in self.iter().enumerate() {
                    unsafe { ocaml_sys::caml_sys_store_double_field(rv.value().value, i, v as f64) }
                }
                rv.value().value
            }
        }
    };
}
vec_float!(f32);
vec_float!(f64);

impl<T> ToValue for Vec<T>
where
    T: ToValue + crate::from_value::NotF64,
{
    fn to_value(&self) -> ocaml_sys::Value {
        let len = self.len();
        let rv: RootedValue<()> = RootedValue::create(unsafe { ocaml_sys::caml_alloc_tuple(len) });
        for (i, v) in self.iter().enumerate() {
            let v = T::to_value(v);
            unsafe { ocaml_sys::store_field(rv.value().value, i, v) }
        }
        rv.value().value
    }
}

impl<T> ToValue for Option<T>
where
    T: ToValue,
{
    fn to_value(&self) -> ocaml_sys::Value {
        match self {
            None => ocaml_sys::NONE,
            Some(some) => {
                let rv: RootedValue<()> =
                    RootedValue::create(unsafe { ocaml_sys::caml_alloc(1, ocaml_sys::TAG_SOME) });
                let some = T::to_value(some);
                unsafe { ocaml_sys::store_field(rv.value().value, 0, some) };

                rv.value().value
            }
        }
    }
}

impl<T> ToValue for Box<T>
where
    T: ToValue,
{
    fn to_value(&self) -> ocaml_sys::Value {
        T::to_value(self)
    }
}

impl<T, E> ToValue for Result<T, E>
where
    T: ToValue,
    E: ToValue,
{
    fn to_value(&self) -> ocaml_sys::Value {
        match self {
            Err(err) => {
                let rv: RootedValue<()> =
                    RootedValue::create(unsafe { ocaml_sys::caml_alloc(1, 1) });
                let err = E::to_value(err);
                unsafe { ocaml_sys::store_field(rv.value().value, 0, err) };
                rv.value().value
            }
            Ok(ok) => {
                let rv: RootedValue<()> =
                    RootedValue::create(unsafe { ocaml_sys::caml_alloc(1, 0) });
                let ok = T::to_value(ok);
                unsafe { ocaml_sys::store_field(rv.value().value, 0, ok) };
                rv.value().value
            }
        }
    }
}

impl ToValue for Vec<u8> {
    fn to_value(&self) -> ocaml_sys::Value {
        let value = unsafe { ocaml_sys::caml_alloc_string(self.len()) };
        let content_ptr = unsafe { ocaml_sys::string_val(value) };
        unsafe { std::ptr::copy_nonoverlapping(self.as_ptr(), content_ptr, self.len()) };
        value
    }
}

impl ToValue for String {
    fn to_value(&self) -> ocaml_sys::Value {
        let value = unsafe { ocaml_sys::caml_alloc_string(self.len()) };
        let content_ptr = unsafe { ocaml_sys::string_val(value) };
        unsafe { std::ptr::copy_nonoverlapping(self.as_ptr(), content_ptr, self.len()) };
        value
    }
}

impl<T> ToValue for RootedValue<T> {
    fn to_value(&self) -> ocaml_sys::Value {
        self.value().value
    }
}
