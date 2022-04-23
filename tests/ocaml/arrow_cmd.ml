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
  Stdio.printf "%s\n%!" (Arrow.metadata_as_string reader);
  let metadata = Arrow.parquet_metadata reader in
  Stdio.printf "%s\n%!" (Arrow.sexp_of_metadata metadata |> Sexp.to_string_hum);
  let schema = Arrow.schema reader |> ok_exn in
  Stdio.printf "%s\n%!" (Arrow.sexp_of_schema schema |> Sexp.to_string_hum)
