open! Base
open! Sexplib.Conv
open! Test_gen

type 'a res = ('a, string) Result.t [@@deriving sexp]

let%expect_test _ =
  Stdio.printf "%d\n%!" (Ffi.add_one 41);
  Stdio.printf
    "%d\n%!"
    (Ffi.add_i64 (Int64.of_int 1234) (Int64.of_int 5678) |> Int64.to_int_exn);
  let res = Ffi.pair ("foobar", 3.14159265358979, (1337, 299792458)) in
  Stdio.printf "<%s>\n%!" res;
  [%expect {|
    42
    6912
    <foobar:3.14159265358979:1337:299792458> |}]

let%expect_test _ =
  let v = Ffi2.vec_new () in
  for i = 1 to 10 do
    Ffi2.vec_push v i
  done;
  Caml.Gc.compact ();
  let array = Ffi2.vec_content v in
  Caml.Gc.compact ();
  let array = Array.map array ~f:Int64.to_int_exn in
  Stdio.print_s ([%sexp_of: int array] array);
  [%expect {| (1 2 3 4 5 6 7 8 9 10) |}];
  Stdio.print_s (Ffi.option_result (Some 1) "foo" |> [%sexp_of: int res]);
  Stdio.print_s (Ffi.option_result None "foo" |> [%sexp_of: int res]);
  Stdio.printf "= %s =\n%!" (Ffi.str_format (42, -1337) "bar baz");
  [%expect {|
    (Ok 1)
    (Error foo)
    = foo<42|-1337>: bar baz = |}];
  let v = Ffi.vec_add [| 3; 1; 4; 1; 5; 9; 2; 6; 5 |] (-1) in
  Stdio.print_s ([%sexp_of: int array] v);
  [%expect {| (2 0 3 0 4 8 1 5 4) |}]

let%expect_test _ =
  Stdio.printf "\n==== Test Struct ====\n";
  let t =
    { Ffi3.x = 42; y = "foo"; z = 1337, None, 3.14; zs = [| 3.14; 2.71828182846 |] }
  in
  Stdio.printf "<%s>\n%!" (Ffi3.mystruct_to_string t);
  let t = Ffi3.mystruct_add_x t 1337 in
  Stdio.printf "<%s>\n%!" (Ffi3.mystruct_to_string t);
  Stdio.printf "%s\n" (Ffi3.sexp_of_my_struct t |> Sexp.to_string);
  [%expect
    {|
    ==== Test Struct ====
    <MyStruct { x: 42, y: "foo", z: (1337, None, 3.14), zs: [3.14, 2.71828182846] }>
    <MyStruct { x: 1379, y: "foo", z: (1337, None, 3.14), zs: [3.14, 2.71828182846] }>
    ((x 1379)(y foo)(z(1337()3.14))(zs(3.14 2.71828182846))) |}]

let%expect_test _ =
  Stdio.printf "\n==== Test Enum ====\n";
  let myenum m =
    let s1 = Ffi3.myenum_to_string m in
    let s2 = Ffi3.myenum_add_x m 42 |> Ffi3.myenum_to_string in
    Stdio.printf "<%s> <%s>\n%!" s1 s2
  in
  myenum NoArg;
  myenum (OneArg 42);
  myenum (StructArgs { x = 1337; y = "FooBar" });
  myenum (TwoArgs (1337, "FooBar"));
  myenum (Rec NoArg);
  myenum (Rec (Rec NoArg));
  myenum (Rec (Rec (Rec (StructArgs { x = 1337; y = "FooBar" }))));
  [%expect
    {|
    ==== Test Enum ====
    <NoArg> <NoArg>
    <OneArg(42)> <OneArg(84)>
    <StructArgs { x: 1337, y: "FooBar" }> <StructArgs { x: 1379, y: "FooBar" }>
    <TwoArgs(1337, "FooBar")> <TwoArgs(1379, "FooBar")>
    <Rec(NoArg)> <Rec(NoArg)>
    <Rec(Rec(NoArg))> <Rec(Rec(NoArg))>
    <Rec(Rec(Rec(StructArgs { x: 1337, y: "FooBar" })))> <Rec(Rec(Rec(StructArgs { x: 1379, y: "FooBar" })))> |}]

let%expect_test _ =
  Stdio.printf "\n==== Test Closures ====\n";
  Ffi4.map_callback [| 3; 1; 4; 1; 5; 9; 2 |] (Printf.sprintf "<%d>")
  |> [%sexp_of: string array]
  |> Stdio.print_s;
  [%expect {|
    ==== Test Closures ====
    (<3> <1> <4> <1> <5> <9> <2>) |}];
  let r = ref 0 in
  Ffi4.sum_n 20 (fun () ->
      Int.incr r;
      !r)
  |> Stdio.printf "%d\n%!";
  [%expect {| 210 |}];
  try
    let (_ : int) = Ffi4.sum_n 1 (fun () -> failwith "ocaml-failwith") in
    ()
  with
  | exn ->
    Stdio.printf "failed as expected, %s\n%!" (Exn.to_string exn);
    [%expect
      {|
      failed as expected, (Failure
        "panicked at 'called `Result::unwrap()` on an `Err` value: ocaml exn: Failure(\"ocaml-failwith\")', example/src/lib.rs:137:31") |}]

let%expect_test _ =
  Stdio.printf "\n==== Test Custom Drop ====\n";
  let foo1 = Ffi6.create_foo2 42 in
  let foo2 = Ffi6.create_foo2 1337 in
  Stdio.printf "%s\n%!" (Ffi6.foo2_to_string foo1);
  Stdio.printf "%s\n%!" (Ffi6.foo2_to_string foo2);
  [%expect
    {|
    ==== Test Custom Drop ====
    v: Foo { v: 42 }
    v: Foo { v: 1337 } |}];
  Caml.Gc.compact ();
  [%expect {| dropping foo 1337 |}];
  Stdio.printf "%s\n%!" (Ffi6.foo2_to_string foo1);
  Caml.Gc.compact ();
  [%expect {|
    v: Foo { v: 42 }
    dropping foo 42 |}]

let%expect_test _ =
  Stdio.printf "\n==== Test GC Safety ====\n";
  let (((a, b), _c), d), e = Ffi7.generate 1664 in
  Stdio.printf "%Ld %Ld %Ld %Ld\n" a b d e;
  [%expect {|
    Rust: 1664 0 0 1 664

    ==== Test GC Safety ====
    0 0 1 664 |}]

let%expect_test _ =
  Stdio.printf "\n==== Test Double Array ====\n";
  let vs = Ffi_double_array.add_ones [| 3.14; 15.92; 65.35 |] in
  Stdio.print_s ([%sexp_of: float array] vs);
  [%expect {|
    ==== Test Double Array ====
    (4.1400000000000006 16.92 66.35) |}];
  let q = { Ffi_double_array.a = 3.14; b = 15.92; c = 65.35; d = 89.79 } in
  let q_2 = Ffi_double_array.add_quat q q in
  Stdio.print_s ([%sexp_of: Ffi_double_array.quaternion] q_2);
  [%expect {|
    Quaternion { a: 3.14, b: 15.92, c: 65.35, d: 89.79 } Quaternion { a: 3.14, b: 15.92, c: 65.35, d: 89.79 }
    ((a 6.94557779813029E-310) (b 6.9455777981295E-310) (c 6.94557779812871E-310)
     (d 6.94557779812792E-310)) |}]
