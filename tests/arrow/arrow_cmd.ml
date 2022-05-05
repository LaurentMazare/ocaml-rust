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
  A.Reader.with_reader path ~batch_size:(8 * 4096) ~f:(fun reader ->
      let schema = A.Reader.schema reader |> ok_exn in
      Stdio.printf "%s\n%!" (A.Schema.sexp_of_t schema |> Sexp.to_string_hum);
      let rec loop batch_index =
        match A.Reader.next reader with
        | `Eof -> Stdio.printf "done\n%!"
        | `Batch rb ->
          let rb = ok_exn rb in
          Stdio.printf
            "  batch %d: %d rows, %d columns\n%!"
            batch_index
            (A.Record_batch.num_rows rb)
            (A.Record_batch.num_columns rb);
          A.Record_batch.columns rb
          |> List.iter ~f:(fun (name, packed) ->
                 Stdio.printf
                   "    %s: %s\n%!"
                   name
                   (A.Column.sexp_of_packed packed |> Sexp.to_string_mach));
          loop (batch_index + 1)
      in
      loop 1)
  |> ok_exn
