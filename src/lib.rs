// TODO: pretty much everything.
//   - Call from Rust, start the OCaml runtime in that case?
pub mod closure;
pub mod custom;
pub mod error;
pub mod exn;
pub mod from_value;
pub mod gc;
pub mod rooted;
pub mod to_value;
pub mod value;
pub use custom::Custom;
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
