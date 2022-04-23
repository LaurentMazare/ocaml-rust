pub trait ToValue {
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U;
}

impl ToValue for () {
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        pin(ocaml_sys::UNIT)
    }
}

impl ToValue for i32 {
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        pin(unsafe { ocaml_sys::caml_copy_int32(*self) })
    }
}

impl ToValue for i64 {
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        pin(unsafe { ocaml_sys::caml_copy_int64(*self) })
    }
}

impl ToValue for f64 {
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        pin(unsafe { ocaml_sys::caml_copy_double(*self) })
    }
}

impl ToValue for isize {
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        pin(unsafe { ocaml_sys::val_int(*self) })
    }
}

impl ToValue for usize {
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        pin(unsafe { ocaml_sys::val_int(*self as isize) })
    }
}

impl ToValue for bool {
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        let v = if *self { 1 } else { 0 };
        pin(unsafe { ocaml_sys::val_int(v) })
    }
}

impl<T1, T2> ToValue for (T1, T2)
where
    T1: ToValue,
    T2: ToValue,
{
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        let (v1, v2) = self;
        let v = unsafe { ocaml_sys::caml_alloc_tuple(2) };
        let res = pin(v);
        T1::to_value(v1, |x| unsafe { ocaml_sys::store_field(v, 0, x) });
        T2::to_value(v2, |x| unsafe { ocaml_sys::store_field(v, 1, x) });
        res
    }
}

impl<T1, T2, T3> ToValue for (T1, T2, T3)
where
    T1: ToValue,
    T2: ToValue,
    T3: ToValue,
{
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        let (v1, v2, v3) = self;
        let v = unsafe { ocaml_sys::caml_alloc_tuple(3) };
        let res = pin(v);
        T1::to_value(v1, |x| unsafe { ocaml_sys::store_field(v, 0, x) });
        T2::to_value(v2, |x| unsafe { ocaml_sys::store_field(v, 1, x) });
        T3::to_value(v3, |x| unsafe { ocaml_sys::store_field(v, 2, x) });
        res
    }
}

impl<T> ToValue for Vec<T>
where
    T: ToValue,
{
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        let len = self.len();
        let array = unsafe { ocaml_sys::caml_alloc_tuple(len) };
        let res = pin(array);
        for (i, v) in self.iter().enumerate() {
            T::to_value(v, |x| unsafe { ocaml_sys::store_field(array, i, x) })
        }
        res
    }
}

impl<T> ToValue for Option<T>
where
    T: ToValue,
{
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        match self {
            None => pin(ocaml_sys::NONE),
            Some(some) => {
                let v = unsafe { ocaml_sys::caml_alloc(1, ocaml_sys::TAG_SOME) };
                let res = pin(v);
                T::to_value(some, |x| unsafe { ocaml_sys::store_field(v, 0, x) });
                res
            }
        }
    }
}

impl<T, E> ToValue for Result<T, E>
where
    T: ToValue,
    E: ToValue,
{
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        match self {
            Err(err) => {
                let v = unsafe { ocaml_sys::caml_alloc(1, 1) };
                let res = pin(v);
                E::to_value(err, |x| unsafe { ocaml_sys::store_field(v, 0, x) });
                res
            }
            Ok(ok) => {
                let v = unsafe { ocaml_sys::caml_alloc(1, 0) };
                let res = pin(v);
                T::to_value(ok, |x| unsafe { ocaml_sys::store_field(v, 0, x) });
                res
            }
        }
    }
}

impl ToValue for Vec<u8> {
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        let v = unsafe { ocaml_sys::caml_alloc_string(self.len()) };
        let res = pin(v);
        let content_ptr = unsafe { ocaml_sys::string_val(v) };
        unsafe { std::ptr::copy_nonoverlapping(self.as_ptr(), content_ptr, self.len()) };
        res
    }
}

impl ToValue for String {
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        let v = unsafe { ocaml_sys::caml_alloc_string(self.len()) };
        let res = pin(v);
        let content_ptr = unsafe { ocaml_sys::string_val(v) };
        unsafe { std::ptr::copy_nonoverlapping(self.as_ptr(), content_ptr, self.len()) };
        res
    }
}

pub fn to_rooted_value<T>(t: &T) -> crate::RootedValue<T>
where
    T: ToValue,
{
    t.to_value(crate::RootedValue::create)
}
