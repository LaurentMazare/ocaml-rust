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
  let file_reader = Arrow.file_reader path |> ok_exn in
  Stdio.printf "File: %s\n%!" path;
  Stdio.printf "%s\n%!" (Arrow.metadata_as_string file_reader);
  let metadata = Arrow.parquet_metadata file_reader in
  Stdio.printf "%s\n%!" (Arrow.sexp_of_metadata metadata |> Sexp.to_string_hum);
  let schema = Arrow.schema file_reader |> ok_exn in
  Stdio.printf "%s\n%!" (Arrow.sexp_of_schema schema |> Sexp.to_string_hum);
  let record_reader = Arrow.get_record_reader file_reader (1024 * 1024) |> ok_exn in
  let rec loop () =
    match Arrow.record_reader_next record_reader with
    | None -> Stdio.printf "done\n%!"
    | Some batch ->
      Stdio.printf
        "  batch %d %d\n%!"
        (Arrow.record_batch_num_rows batch)
        (Arrow.record_batch_num_columns batch);
      loop ()
  in
  loop ()
