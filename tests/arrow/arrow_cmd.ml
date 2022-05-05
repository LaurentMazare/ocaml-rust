open! Base
open! Sexplib.Conv
module A = Arrow_lib.Arrow_core

let ok_exn = function
  | Ok ok -> ok
  | Error err -> Printf.failwithf "%s" err ()

let () =
  let path =
    match Sys.get_argv () with
    | [||] | [| _ |] -> "/tmp/foo.parquet"
    | argv -> argv.(1)
  in
  Stdio.printf "File: %s\n%!" path;
  Arrow_lib.Arrow_test.read_and_print path ~batch_size:(8 * 4096)
