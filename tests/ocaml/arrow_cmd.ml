open! Base
open! Sexplib.Conv
open! Arrow_gen

let ok_exn = function
  | Ok ok -> ok
  | Error err -> Printf.failwithf "%s" err ()

let () =
  let path =
    match Sys.get_argv () with
    | [||] | [| _ |] -> "/tmp/foo.parquet"
    | argv -> argv.(1)
  in
  let reader = Arrow.reader path |> ok_exn in
  Stdio.printf "File: %s\n%!" path;
  Stdio.printf "%s\n%!" (Arrow.metadata_as_string reader)
