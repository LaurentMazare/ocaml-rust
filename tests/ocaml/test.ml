open! Base
open! Sexplib.Conv
module Ffi = Test_gen.Ffi
module Ffi2 = Test_gen.Ffi2
module Ffi3 = Test_gen.Ffi3
module Ffi4 = Test_gen.Ffi4
module Ffi5 = Test_gen.Ffi5

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
  let array = Ffi2.vec_content v |> Array.map ~f:Int64.to_int_exn in
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
  [%expect
    {|
    ==== Test Struct ====
    <MyStruct { x: 42, y: "foo", z: (1337, None, 3.14), zs: [3.14, 2.71828182846] }>
    <MyStruct { x: 1379, y: "foo", z: (1337, None, 3.14), zs: [3.14, 2.71828182846] }> |}]

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
  [%expect
    {|
    ==== Test Enum ====
    <NoArg> <NoArg>
    <OneArg(42)> <OneArg(84)>
    <StructArgs { x: 1337, y: "FooBar" }> <StructArgs { x: 1379, y: "FooBar" }>
    <TwoArgs(1337, "FooBar")> <TwoArgs(1379, "FooBar")> |}]

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
    [%expect {| failed as expected, (Failure "@\134C\132hU") |}]

let%expect_test _ =
  Stdio.printf "\n==== Test Custom Drop ====\n";
  let foo1 = Ffi5.create_foo 42 in
  let foo2 = Ffi5.create_foo 1337 in
  Stdio.printf "%s\n%!" (Ffi5.foo_to_string foo1);
  Stdio.printf "%s\n%!" (Ffi5.foo_to_string foo2);
  [%expect {|
    ==== Test Custom Drop ====
    vFoo { v: 42 }
    vFoo { v: 1337 } |}];
  Caml.Gc.compact ();
  [%expect {| dropping foo 1337 |}];
  Stdio.printf "%s\n%!" (Ffi5.foo_to_string foo1);
  Caml.Gc.compact ();
  [%expect {|
    vFoo { v: 42 }
    dropping foo 42 |}]
