// TODO: pretty much everything.
//   - Closures.
//   - Call from Rust, start the OCaml runtime in that case?
pub mod closure;
pub mod custom;
pub mod from_value;
pub mod gc;
mod rooted;
pub mod to_value;
pub mod value;
pub use ocaml_rust_macro::bridge;
pub use rooted::RootedValue;
pub use to_value::to_rooted_value;
pub use value::Value;

static PANIC_HOOK_SETUP: std::sync::Once = std::sync::Once::new();

pub fn initial_setup() {
    PANIC_HOOK_SETUP.call_once(|| unsafe {
        std::panic::set_hook(Box::new(|panic_info| {
            let payload = std::ffi::CString::new(panic_info.to_string());
            let ptr = if let Ok(payload) = payload {
                payload.as_ptr()
            } else {
                "unhandled rust panic\0".as_ptr() as *const i8
            };
            ocaml_sys::caml_failwith(libc::strdup(ptr));
        }))
    });
}
