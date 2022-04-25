use ocaml_rust::closure::{Fn0, Fn1};

fn option_result(v: Option<isize>, e: String) -> Result<isize, String> {
    match v {
        Some(v) => Ok(v),
        None => Err(e),
    }
}

fn add_one(x: isize) -> isize {
    x + 1
}

fn add_i64(x: i64, y: i64) -> i64 {
    x + y
}

fn str_format(x: (isize, isize), y: String) -> String {
    format!("foo<{}|{}>: {}", x.0, x.1, y)
}

fn pair(xy: (String, f64, (isize, isize))) -> String {
    let (x, y, (a, b)) = xy;
    format!("{}:{}:{}:{}", x, y, a, b)
}
fn vec_add(x: Vec<isize>, y: isize) -> Vec<isize> {
    x.iter().map(|x| x + y).collect()
}

#[ocaml_rust::bridge]
mod ffi {
    extern "Rust" {
        fn add_one(x: isize) -> isize;
        fn add_i64(x: i64, y: i64) -> i64;
        fn str_format(x: (isize, isize), y: String) -> String;
        fn pair(xy: (String, f64, (isize, isize))) -> String;
        fn option_result(v: Option<isize>, e: String) -> Result<isize, String>;
        fn vec_add(x: Vec<isize>, y: isize) -> Vec<isize>;
    }
}

fn vec_new() -> MyVec {
    Custom::new(Vec::new())
}

fn vec_push(v: &MyVec, x: isize) {
    let mut v = v.inner().lock().unwrap();
    v.push(x as i64);
}

fn vec_content(v: &MyVec) -> Vec<i64> {
    let v = v.inner().lock().unwrap();
    v.clone()
}

#[ocaml_rust::bridge]
mod ffi2 {
    type MyVec = Custom<Vec<i64>>;

    extern "Rust" {
        fn vec_new() -> MyVec;
        fn vec_push(vec: &MyVec, v: isize);
        fn vec_content(vec: &MyVec) -> Vec<i64>;
    }
}

#[ocaml_rust::bridge]
mod ffi3 {
    // The following is included in the generated OCaml code.
    ocaml_include!("open! Sexplib.Conv");

    #[derive(Debug, Clone)]
    enum MyEnum {
        NoArg,
        OneArg(isize),
        TwoArgs(isize, String),
        StructArgs { x: isize, y: String },
        Rec(Box<MyEnum>),
    }

    #[ocaml_deriving(sexp)]
    #[derive(Debug, Clone)]
    struct MyStruct {
        x: isize,
        y: String,
        z: (isize, Option<String>, f64),
        zs: Vec<f64>,
    }

    extern "Rust" {
        fn mystruct_to_string(v: &MyStruct) -> String;
        fn mystruct_add_x(v: &MyStruct, x: isize) -> MyStruct;
        fn myenum_to_string(v: &MyEnum) -> String;
        fn myenum_add_x(m: &MyEnum, v: isize) -> MyEnum;
    }
}

fn mystruct_to_string(v: &MyStruct) -> String {
    format!("{:?}", v)
}

fn mystruct_add_x(v: &MyStruct, x: isize) -> MyStruct {
    let mut v = v.clone();
    v.x += x;
    v
}

fn myenum_to_string(v: &MyEnum) -> String {
    format!("{:?}", v)
}

fn myenum_add_x(m: &MyEnum, v: isize) -> MyEnum {
    match m {
        MyEnum::NoArg => MyEnum::NoArg,
        MyEnum::OneArg(x) => MyEnum::OneArg(x + v),
        MyEnum::TwoArgs(x, s) => MyEnum::TwoArgs(x + v, s.to_string()),
        MyEnum::StructArgs { x, y } => MyEnum::StructArgs { x: x + v, y: y.to_string() },
        MyEnum::Rec(r) => MyEnum::Rec(Box::new(myenum_add_x(r, v))),
    }
}

#[ocaml_rust::bridge]
mod ffi4 {
    extern "Rust" {
        fn map_callback(vs: &Vec<isize>, f: &mut Fn1<isize, String>) -> Vec<String>;

        fn sum_n(n: isize, f: &mut Fn0<isize>) -> isize;
    }
}

fn map_callback(vs: &[isize], f: &mut Fn1<isize, String>) -> Vec<String> {
    vs.iter().map(|x| f.call1(*x).unwrap()).collect()
}

fn sum_n(n: isize, f: &mut Fn0<isize>) -> isize {
    (0..n).map(|_x| f.call0().unwrap()).sum()
}

#[derive(Debug, Clone)]
struct Foo {
    v: isize,
}

impl Drop for Foo {
    fn drop(&mut self) {
        println!("dropping foo {}", self.v)
    }
}

use ocaml_rust::Custom;

#[ocaml_rust::bridge]
mod ffi6 {
    type C = Custom<Foo>;
    extern "Rust" {
        fn create_foo2(v: isize) -> C;
        fn foo2_to_string(v: &C) -> String;
    }
}

fn create_foo2(v: isize) -> Custom<Foo> {
    Custom::new(Foo { v })
}

fn foo2_to_string(v: &Custom<Foo>) -> String {
    let v = v.inner().lock().unwrap();
    format!("v: {:?}", v)
}

struct CompactToken();

impl ocaml_rust::to_value::ToValue for CompactToken {
    fn to_value(&self) -> ocaml_sys::Value {
        unsafe {
            ocaml_sys::caml_gc_compaction(ocaml_sys::UNIT);
            ocaml_sys::UNIT
        }
    }
}

#[ocaml_rust::bridge]
mod ffi7 {
    type Compact = CompactToken;
    extern "Rust" {
        fn generate(i: isize) -> ((((i64, i64), Compact), i64), i64);
    }
}

#[allow(clippy::type_complexity)]
fn generate(i0: isize) -> ((((i64, i64), Compact), i64), i64) {
    let mut i = i0 as i64;
    let d = i % 1000;
    i /= 1000;
    let c = i % 1000;
    i /= 1000;
    let b = i % 1000;
    i /= 1000;
    let a = i;
    println!("Rust: {} {} {} {} {}", i0, a, b, c, d);
    ((((a, b), CompactToken()), c), d)
}
