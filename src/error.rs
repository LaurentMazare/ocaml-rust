pub type RustError = Box<dyn std::error::Error>;
pub type RustResult<T> = std::result::Result<T, RustError>;

impl crate::to_value::ToValue for RustError {
    fn to_value(&self) -> ocaml_sys::Value {
        // This is handled as a string both here and in the ocaml code generation.
        self.to_string().to_value()
    }
}
