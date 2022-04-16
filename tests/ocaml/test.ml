module Ffi = Test_gen.Ffi
module Ffi2 = Test_gen.Ffi2
module Ffi3 = Test_gen.Ffi3
module Ffi4 = Test_gen.Ffi4

let r_to_string = function
  | Ok ok -> string_of_int ok
  | Error err -> err

let () =
  Stdio.printf "%d\n%!" (Ffi.add_one 41);
  Stdio.printf
    "%d\n%!"
    (Ffi.add_i64 (Int64.of_int 1234) (Int64.of_int 5678) |> Int64.to_int);
  let res = Ffi.pair ("foobar", 3.14159265358979, (1337, 299792458)) in
  Stdio.printf "<%s>\n%!" res;
  let v = Ffi2.vec_new () in
  for i = 1 to 10 do
    Ffi2.vec_push v i
  done;
  let array = Ffi2.vec_content v in
  Stdio.printf "vec<%d>: " (Array.length array);
  Array.iter (fun i64 -> Stdio.printf "%d " (Int64.to_int i64)) array;
  Stdio.printf "\n%!";
  Stdio.printf "opt-result %s\n%!" (Ffi.option_result (Some 1) "foo" |> r_to_string);
  Stdio.printf "opt-result %s\n%!" (Ffi.option_result None "foo" |> r_to_string);
  Stdio.printf "= %s =\n%!" (Ffi.str_format (42, -1337) "bar baz");
  let v =
    Ffi.vec_add [| 3; 1; 4; 1; 5; 9; 2; 6; 5 |] (-1)
    |> Array.to_list
    |> List.map string_of_int
    |> String.concat ","
  in
  Stdio.printf "<%s>\n%!" v

let () =
  Stdio.printf "\n==== Test Struct ====\n";
  let t =
    { Ffi3.x = 42; y = "foo"; z = 1337, None, 3.14; zs = [| 3.14; 2.71828182846 |] }
  in
  Stdio.printf "<%s>\n%!" (Ffi3.mystruct_to_string t);
  let t = Ffi3.mystruct_add_x t 1337 in
  Stdio.printf "<%s>\n%!" (Ffi3.mystruct_to_string t)

let () =
  Stdio.printf "\n==== Test Enum ====\n";
  let myenum m =
    let s1 = Ffi3.myenum_to_string m in
    let s2 = Ffi3.myenum_add_x m 42 |> Ffi3.myenum_to_string in
    Stdio.printf "<%s> <%s>\n%!" s1 s2
  in
  myenum NoArg;
  myenum (OneArg 42);
  myenum (StructArgs { x = 1337; y = "FooBar" });
  myenum (TwoArgs (1337, "FooBar"))

let () =
  Stdio.printf "\n==== Test Closures ====\n";
  Ffi4.map_callback [| 3; 1; 4; 1; 5; 9; 2 |] (Printf.sprintf "<%d>")
  |> Array.to_list
  |> String.concat ","
  |> Stdio.printf "%s\n%!";
  let r = ref 0 in
  Ffi4.sum_n 20 (fun () ->
      incr r;
      !r)
  |> Stdio.printf "%d\n%!"
