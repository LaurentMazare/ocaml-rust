fn create() -> VecI64 {
    Vec::<i64>::new()
}

fn push(v: &mut VecI64, i: i64) {
    v.push(i)
}

fn pop(v: &mut VecI64) -> Option<i64> {
    v.pop()
}

fn len(v: &VecI64) -> isize {
    v.len() as isize
}

#[ocaml_rust::bridge]
mod ffi {
    type VecI64 = Vec<i64>;

    extern "Rust" {
        fn create() -> VecI64;
        fn push(v: &mut VecI64, i: i64);
        fn pop(v: &mut VecI64) -> Option<i64>;
        fn len(v: &VecI64) -> isize;
    }
}

#[test]
fn test() {
    assert_eq!(21 + 21, 42)
}
