open! Base
open! Sexplib.Conv
open! Test_lib.Test_gen

let () =
  for i = 1 to 100 do
    Stdio.printf "%d: %d\n%!" i (Ffi.add_one 41);
    Stdio.printf
      "%d\n%!"
      (Ffi.add_i64 (Int64.of_int 1234) (Int64.of_int 5678) |> Int64.to_int_exn);
    let res = Ffi.pair ("foobar", 3.14159265358979, (1337, 299792458)) in
    Stdio.printf "<%s>\n%!" res;
    Caml.Gc.compact ()
  done
