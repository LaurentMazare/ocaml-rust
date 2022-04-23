pub type RustError = Box<dyn std::error::Error>;
pub type RustResult<T> = std::result::Result<T, RustError>;

impl crate::to_value::ToValue for RustError {
    fn to_value<F, U>(&self, pin: F) -> U
    where
        U: Sized,
        F: FnOnce(ocaml_sys::Value) -> U,
    {
        // This is handled as a string both here and in the ocaml code generation.
        self.to_string().to_value(pin)
    }
}
