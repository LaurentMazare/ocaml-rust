use crate::rooted::RootedValue;
use crate::to_value::ToValue;
use ocaml_sys::bigarray::Kind;

pub trait Elem {
    const KIND: Kind;
}

pub struct BigArray1<E: 'static>(RootedValue<Vec<E>>);

impl<E: Elem> crate::from_value::FromSysValue for BigArray1<E> {
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        BigArray1(RootedValue::create(v))
    }
}

impl<E: Elem> ToValue for BigArray1<E> {
    fn to_value(&self) -> ocaml_sys::Value {
        self.0.value().value
    }
}

impl<E: Elem> BigArray1<E> {
    pub fn new(vs: &[E]) -> Self {
        // https://github.com/ocaml/ocaml/blob/66b63e2f2459c0a2754658e847894eacb4cacc34/runtime/caml/bigarray.h#L61
        // CAML_BA_C_LAYOUT is 0
        let flags = E::KIND as i32;
        let len = vs.len();
        unsafe {
            let content = ocaml_sys::bigarray::malloc(len * std::mem::size_of::<E>());
            std::ptr::copy_nonoverlapping(vs.as_ptr(), content as *mut E, len);
            let value = ocaml_sys::bigarray::caml_ba_alloc_dims(flags, 1, content, len);
            BigArray1(RootedValue::create(value))
        }
    }

    pub fn data(&self) -> &[E] {
        let v = self.0.to_value();
        let ba = unsafe { ocaml_sys::field(v, 1) } as *mut ocaml_sys::bigarray::Bigarray;
        let ba = unsafe { &*ba };
        if ba.num_dims != 1 {
            panic!("unexpected number of dimensions for bigarray {}", ba.num_dims)
        }
        let dim = unsafe { std::slice::from_raw_parts(ba.dim.as_ptr() as *const isize, 1) };
        unsafe { std::slice::from_raw_parts(ba.data as *const E, dim[0] as usize) }
    }
}

impl Elem for u8 {
    const KIND: Kind = Kind::UINT8;
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
