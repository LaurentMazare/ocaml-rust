// TODO:
//   - Call from Rust, start the OCaml runtime in that case?
//   - Dedicated extern "OCaml" section.
//   - Provide a way to specify/override the generated ocaml types.
pub mod bigarray;
pub mod closure;
pub mod custom;
pub mod error;
pub mod exn;
pub mod from_value;
pub mod gc;
pub mod rooted;
pub mod to_value;
pub mod value;
pub use bigarray::BigArray1;
pub use custom::{Custom, CustomConst};
pub use error::{RustError, RustResult};
pub use exn::OCamlExn;
pub use ocaml_rust_macro::bridge;
pub use rooted::RootedValue;
pub use value::Value;

static PANIC_HOOK_SETUP: std::sync::Once = std::sync::Once::new();

pub fn initial_setup() {
    PANIC_HOOK_SETUP.call_once(|| unsafe {
        std::panic::set_hook(Box::new(|panic_info| {
            let panic_info = panic_info.to_string();
            let v = ocaml_sys::caml_alloc_string(panic_info.len());
            let ptr = ocaml_sys::string_val(v);
            core::ptr::copy_nonoverlapping(panic_info.as_ptr(), ptr, panic_info.len());
            ocaml_sys::caml_failwith_value(v);
        }))
    });
}

/// A struct to represent having released the OCaml runtime lock. The
/// drop implementation guarantees acquiring the lock back at the end
/// of the scope.
pub struct RuntimeLock {}

impl RuntimeLock {
    pub fn release() -> Self {
        unsafe { ocaml_sys::caml_enter_blocking_section() };
        RuntimeLock {}
    }
}

impl Drop for RuntimeLock {
    fn drop(&mut self) {
        unsafe { ocaml_sys::caml_leave_blocking_section() };
    }
}
