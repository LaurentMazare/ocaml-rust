# ocaml-rust

This repo contains code for a proof of concept for a safe OCaml-Rust interop
inspired by [cxx](https://cxx.rs/). This is mostly optimized for calling
Rust code from OCaml at the moment. The interface to be exposed is defined
in Rust and used both to generate some Rust wrapping code as a macro but
also the necessary OCaml type and function definitions.

## Running the Examples

To try out the main example,
```bash
cd tests/ocaml
make test
```

## Calling Rust Functions from OCaml
In this example, the Rust code to be exposed is specified via the following
code. The `#[ocaml_rust::bridge]` macros wraps the Rust function in a way
that can be called from OCaml.

```rust
#[ocaml_rust::bridge]
mod ffi {
    extern "Rust" {
        fn add_one(x: isize) -> isize;
    }
}

fn add_one(x: isize) -> isize {
    x + 1
}
```

The OCaml code generation code in `gen/cmd` will automatically generate the
OCaml external definition:

```ocaml
type isize = int;;
module Ffi = struct
  external add_one : isize -> isize = "__ocaml_ffi_add_one"
end
```

And then OCaml code can simply refer to this module to call the Rust function.

```ocaml
let () = Stdio.printf "%d\n%!" (Test_gen.Ffi.add_one 41)
```

## Sharing Type Definitions between OCaml and Rust
It is also possible to define struct or enum types in the ffi module.
The equivalent OCaml record or variant definitions will be generated
and automatically converted too.

For example, a ffi module can be defined as the following:
```rust
#[ocaml_rust::bridge]
mod ffi3 {
    #[derive(Debug, Clone)]
    enum MyEnum {
        NoArg,
        OneArg(isize),
        TwoArgs(isize, String),
        StructArgs { x: isize, y: String },
    }

    #[derive(Debug, Clone)]
    struct MyStruct {
        x: isize,
        y: String,
        z: (isize, Option<String>, f64),
        zs: Vec<f64>,
    }

    extern "Rust" {
        fn mystruct_to_string(v: &MyStruct) -> String;
    }
}
```

This results in the following OCaml code being generated.
```ocaml
module Ffi3 = struct
  type my_enum =
  | NoArg
  | OneArg of isize
  | TwoArgs of isize * string
  | StructArgs of { x: isize; y: string }
  [@@boxed];;

  type my_struct = {
    x: isize;
    y: string;
    z: (isize * string option * f64);
    zs: f64 array;
  } [@@boxed];;

  external mystruct_to_string : my_struct -> string = "__ocaml_ffi3_mystruct_to_string"
end
```

Finally, defining type aliases in the ffi module results in the
Rust data to be wrapped in an OCaml abstract type. E.g.:

```rust
#[ocaml_rust::bridge]
mod ffi2 {
    type MyVec = Vec<i64>;

    extern "Rust" {
        fn vec_new() -> MyVec;
        fn vec_push(vec: &mut MyVec, v: isize);
        fn vec_content(vec: &MyVec) -> Vec<i64>;
    }
}
```

And the resulting OCaml module is:
```ocaml
module Ffi2 = struct
  type my_vec;;
  external vec_new : unit -> my_vec = "__ocaml_ffi2_vec_new"
  external vec_push : my_vec -> isize -> unit = "__ocaml_ffi2_vec_push"
  external vec_content : my_vec -> i64 array = "__ocaml_ffi2_vec_content"
```
 
## Other OCaml-Rust FFI

- [ocaml-interop](https://github.com/tezedge/ocaml-interop).
- [ocaml-rs](https://github.com/zshipko/ocaml-rs).
