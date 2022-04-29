use crate::to_value::ToValue;
use ocaml_sys::bigarray::Kind;

trait Elem {
    const KIND: Kind;
}

pub struct BigArray1<'a, E>(&'a [E]);

impl<'a, E: Elem> ToValue for BigArray1<'a, E> {
    fn to_value(&self) -> ocaml_sys::Value {
        // https://github.com/ocaml/ocaml/blob/66b63e2f2459c0a2754658e847894eacb4cacc34/runtime/caml/bigarray.h#L61
        // CAML_BA_C_LAYOUT is 0
        let flags = E::KIND as i32;
        let len = self.0.len();
        unsafe {
            let content = ocaml_sys::bigarray::malloc(len * std::mem::size_of::<E>());
            std::ptr::copy_nonoverlapping(self.0.as_ptr(), content as *mut E, len);
            let value = ocaml_sys::bigarray::caml_ba_alloc_dims(flags, 1, content, len);
            value
        }
    }
}

impl Elem for f32 {
    const KIND: Kind = Kind::FLOAT32;
}

impl Elem for f64 {
    const KIND: Kind = Kind::FLOAT64;
}

impl Elem for i32 {
    const KIND: Kind = Kind::INT32;
}

impl Elem for i64 {
    const KIND: Kind = Kind::INT64;
}
