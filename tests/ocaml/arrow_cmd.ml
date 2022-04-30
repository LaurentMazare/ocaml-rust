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
  let str_indexes =
    schema.fields
    |> Array.filter_mapi ~f:(fun index (field : Arrow.schema_field) ->
           match field.data_type with
           | Utf8 -> Some (field.name, index)
           | _ -> None)
  in
  let f64_indexes =
    schema.fields
    |> Array.filter_mapi ~f:(fun index (field : Arrow.schema_field) ->
           match field.data_type with
           | Float64 -> Some (field.name, index)
           | _ -> None)
  in
  let record_reader = Arrow.get_record_reader file_reader (64 * 1024) |> ok_exn in
  let rec loop () =
    match Arrow.record_reader_next record_reader with
    | None -> Stdio.printf "done\n%!"
    | Some batch ->
      let batch = ok_exn batch in
      Stdio.printf
        "  batch %d %d\n%!"
        (Arrow.record_batch_num_rows batch)
        (Arrow.record_batch_num_columns batch);
      Array.iter f64_indexes ~f:(fun (name, index) ->
          let array = Arrow.record_batch_column batch index in
          let ba = Option.value_exn (Arrow.array_f64_values_ba array) in
          Stdio.printf "  >> %s (%d)\n" name (Bigarray.Array1.dim ba);
          let array = Option.value_exn (Arrow.array_f64_values array) in
          Array.iteri array ~f:(Stdio.printf "    %5d %f\n"));
      Array.iter str_indexes ~f:(fun (name, index) ->
          let array = Arrow.record_batch_column batch index in
          Stdio.printf "  >> %s\n" name;
          let array = Option.value_exn (Arrow.array_string_values array) in
          Array.iteri array ~f:(fun i v ->
              Stdio.printf "    %5d %s\n" i (Option.value v ~default:"<none>")));
      loop ()
  in
  Stdio.printf "reading batches\n%!";
  loop ()
