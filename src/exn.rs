#[derive(Clone, Debug)]
pub struct OCamlExn {
    pub message: String,
}

pub type OCamlError<'a> = crate::value::Value<'a, OCamlExn>;
pub type Result<'a, T> = std::result::Result<T, OCamlError<'a>>;

impl<'a> std::fmt::Debug for OCamlError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let v = crate::from_value::FromValue::from_value(self);
        write!(f, "ocaml exn: {}", &v.message)
    }
}

impl std::fmt::Display for OCamlExn {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ocaml exn: {}", &self.message)
    }
}

impl crate::from_value::FromSysValue for OCamlExn {
    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
        let c_ptr = ocaml_sys::caml_format_exception(v);
        let c_str = std::ffi::CStr::from_ptr(c_ptr);
        let message: &str = c_str.to_str().unwrap_or("non UTF8 exception");
        OCamlExn { message: message.to_string() }
    }
}
