(* More idiomatic wrapper around the Arrow api.
   - Use OCaml native types rather than the specialized versions.
   - Handle time/date types.
   *)
open! Core
open Result.Monad_infix
module A = Arrow_gen.Arrow

type ('a, 'b) ba = ('a, 'b, Bigarray.c_layout) Bigarray.Array1.t
type 'a result = ('a, string) Result.t

let ok_exn = function
  | Ok ok -> ok
  | Error err -> failwith err

let default_zone = ref Time_ns_unix.Zone.utc
let set_default_zone z = default_zone := z

module Schema = struct
  module Field = struct
    type t = A.schema_field

    let sexp_of_t = A.sexp_of_schema_field
    let t_of_sexp = A.schema_field_of_sexp
  end

  type t = A.schema

  let sexp_of_t = A.sexp_of_schema
  let t_of_sexp = A.schema_of_sexp
end

module Data_type = struct
  type _ t =
    | Int : int t
    | Float : float t
    | Date : Date.t t
    | Time : Time_ns.t t
    | Ofday : Time_ns.Ofday.t t
    | Span : Time_ns.Span.t t
    | String : string t
  [@@deriving sexp_of]

  let sexp_of t = sexp_of_t (fun _ -> Sexp.List []) t

  let equal_ (type a b) (t1 : a t) (t2 : b t) : (a, b) Type_equal.t option =
    match t1, t2 with
    | Int, Int -> Some T
    | Float, Float -> Some T
    | Date, Date -> Some T
    | Time, Time -> Some T
    | Ofday, Ofday -> Some T
    | Span, Span -> Some T
    | String, String -> Some T
    | _ -> None

  let equal (type a b) (t1 : a t) (t2 : b t) = equal_ t1 t2 |> Option.is_some
end

module Column = struct
  type 'a t =
    { data : A.array_ref
    ; data_type : 'a Data_type.t
    }

  type packed = P : _ t -> packed

  let extract (type a) (P t) (data_type : a Data_type.t) : a t option =
    match Data_type.equal_ t.data_type data_type with
    | Some T -> Some (t : a t)
    | None -> None

  let extract_exn (type a) (P t) (data_type : a Data_type.t) : a t =
    match Data_type.equal_ t.data_type data_type with
    | Some T -> (t : a t)
    | None ->
      [%message
        "data-type mismatch"
          ~expected:(Data_type.sexp_of data_type : Sexp.t)
          ~got:(Data_type.sexp_of t.data_type : Sexp.t)]
      |> raise_s

  let array_ref t = t.data

  let of_array_ref data =
    match A.array_data_type data with
    | Int32 | Int64 -> P { data; data_type = Int }
    | Float32 | Float64 -> P { data; data_type = Float }
    | Utf8 | LargeUtf8 -> P { data; data_type = String }
    | Date32 -> P { data; data_type = Date }
    | Timestamp _ -> P { data; data_type = Time }
    | Time32 _ | Time64 _ -> P { data; data_type = Ofday }
    | Duration _ -> P { data; data_type = Span }
    | data_type -> [%message "unsupported data type" (data_type : A.data_type)] |> raise_s

  let data_type t = t.data_type
  let len t = A.array_len t.data
  let null_count t = A.array_null_count t.data

  module C = struct
    let time data ~zone =
      let data =
        Array.map data ~f:(fun ts -> Time_ns.to_int_ns_since_epoch ts |> Int64.of_int_exn)
      in
      let zone = Time_ns_unix.Zone.to_string zone in
      let data = A.array_timestamp_ns_from_with_zone data (Some zone) in
      { data; data_type = Time }

    let string data =
      let data =
        let sum_length = Array.sum (module Int) data ~f:String.length in
        if sum_length > 2_000_000_000
        then A.array_large_string_from data
        else A.array_string_from data
      in
      { data; data_type = String }

    let span data =
      let data =
        Array.map data ~f:(fun sp -> Time_ns.Span.to_int_ns sp |> Int64.of_int_exn)
        |> A.array_duration_ns_from
      in
      { data; data_type = Span }

    let ofday data =
      let data =
        Array.map data ~f:(fun od ->
            Time_ns.Ofday.to_span_since_start_of_day od
            |> Time_ns.Span.to_int_ns
            |> Int64.of_int_exn)
        |> A.array_time64_ns_from
      in
      { data; data_type = Ofday }

    let date data =
      let data =
        Array.map data ~f:(fun d -> Date.(diff d unix_epoch) |> Int32.of_int_exn)
        |> A.array_date32_from
      in
      { data; data_type = Date }

    let int64 data =
      { data = Array.map data ~f:Int64.of_int_exn |> A.array_i64_from; data_type = Int }

    let int32 data =
      { data = Array.map data ~f:Int32.of_int_exn |> A.array_i32_from; data_type = Int }

    let float64 data = { data = A.array_f64_from data; data_type = Float }
    let float32 data = { data = A.array_f32_from data; data_type = Float }
    let float64_ba d = { data = A.array_f64_from_ba d; data_type = Float }
    let float32_ba d = { data = A.array_f32_from_ba d; data_type = Float }
    let int64_ba d = { data = A.array_i64_from_ba d; data_type = Int }
    let int32_ba d = { data = A.array_i32_from_ba d; data_type = Int }
  end

  let of_array (type a) (data_type : a Data_type.t) (data : a array) =
    match data_type with
    | Int -> (C.int64 data : a t)
    | Float -> C.float64 data
    | Date -> C.date data
    | Time -> C.time data ~zone:!default_zone
    | Ofday -> C.ofday data
    | Span -> C.span data
    | String -> C.string data

  let time_unit_mult : A.time_unit -> int = function
    | Second -> 1_000_000_000
    | Millisecond -> 1_000_000
    | Microsecond -> 1_000
    | Nanosecond -> 1

  let to_array (type a) ?(default : a option) (t : a t) : a array =
    match A.array_data_type t.data, t.data_type with
    | Int32, Int ->
      let default = Option.value default ~default:0 |> Int32.of_int_exn in
      Option.value_exn (A.array_i32_values t.data default)
      |> Array.map ~f:Int32.to_int_exn
    | Int64, Int ->
      let default = Option.value default ~default:0 |> Int64.of_int_exn in
      Option.value_exn (A.array_i64_values t.data default)
      |> Array.map ~f:Int64.to_int_exn
    | Float32, Float ->
      let default = Option.value default ~default:Float.nan in
      Option.value_exn (A.array_f32_values t.data default)
    | Float64, Float ->
      let default = Option.value default ~default:Float.nan in
      Option.value_exn (A.array_f64_values t.data default)
    | Utf8, String ->
      let default = Option.value default ~default:"" in
      Option.value_exn (A.array_string_values t.data)
      |> Array.map ~f:(fun v -> Option.value v ~default)
    | LargeUtf8, String ->
      let default = Option.value default ~default:"" in
      Option.value_exn (A.array_large_string_values t.data)
      |> Array.map ~f:(fun v -> Option.value v ~default)
    | Date32, Date ->
      let default =
        Option.value_map
          default
          ~f:(fun d -> Date.(diff d unix_epoch) |> Int32.of_int_exn)
          ~default:Int32.zero
      in
      Option.value_exn (A.array_date32_values t.data default)
      |> Array.map ~f:(fun d -> Int32.to_int_exn d |> Date.(add_days unix_epoch))
    | Time64 time_unit, Ofday ->
      let time_unit_mult = time_unit_mult time_unit in
      let default =
        Option.value_map
          default
          ~f:(fun od ->
            Time_ns.Ofday.to_span_since_start_of_day od
            |> Time_ns.Span.to_int_ns
            |> fun v -> v / time_unit_mult |> Int64.of_int_exn)
          ~default:Int64.zero
      in
      Option.value_exn (A.array_time64_ns_values t.data default)
      |> Array.map ~f:(fun d ->
             Int64.to_int_exn d * time_unit_mult
             |> Time_ns.Span.of_int_ns
             |> Time_ns.Ofday.of_span_since_start_of_day_exn)
    | Duration time_unit, Span ->
      let time_unit_mult = time_unit_mult time_unit in
      let default =
        Option.value_map
          default
          ~f:(fun sp ->
            Time_ns.Span.to_int_ns sp |> fun v -> v / time_unit_mult |> Int64.of_int_exn)
          ~default:Int64.zero
      in
      Option.value_exn (A.array_time64_ns_values t.data default)
      |> Array.map ~f:(fun d ->
             Int64.to_int_exn d * time_unit_mult |> Time_ns.Span.of_int_ns)
    | Timestamp (time_unit, _zone), Time ->
      let time_unit_mult = time_unit_mult time_unit in
      let default =
        Option.value_map
          default
          ~f:(fun ts ->
            Time_ns.to_int_ns_since_epoch ts / time_unit_mult |> Int64.of_int_exn)
          ~default:Int64.zero
      in
      Option.value_exn (A.array_timestamp_ns_values t.data default)
      |> Array.map ~f:(fun ts ->
             Int64.to_int_exn ts * time_unit_mult |> Time_ns.of_int_ns_since_epoch)
    | data_type, _data_type ->
      [%message "unsupported data type" (data_type : A.data_type)] |> raise_s

  let to_array_opt (type a) (t : a t) : a option array =
    match A.array_data_type t.data, t.data_type with
    | Int32, Int ->
      Option.value_exn (A.array_i32_values_opt t.data)
      |> Array.map ~f:(Option.map ~f:Int32.to_int_exn)
    | Int64, Int ->
      Option.value_exn (A.array_i64_values_opt t.data)
      |> Array.map ~f:(Option.map ~f:Int64.to_int_exn)
    | Float32, Float -> Option.value_exn (A.array_f32_values_opt t.data)
    | Float64, Float -> Option.value_exn (A.array_f64_values_opt t.data)
    | Utf8, String -> Option.value_exn (A.array_string_values t.data)
    | LargeUtf8, String -> Option.value_exn (A.array_large_string_values t.data)
    | Date32, Date ->
      Option.value_exn (A.array_date32_values_opt t.data)
      |> Array.map
           ~f:(Option.map ~f:(fun d -> Int32.to_int_exn d |> Date.(add_days unix_epoch)))
    | Time64 time_unit, Ofday ->
      let time_unit_mult = time_unit_mult time_unit in
      Option.value_exn (A.array_time64_ns_values_opt t.data)
      |> Array.map
           ~f:
             (Option.map ~f:(fun d ->
                  Int64.to_int_exn d * time_unit_mult
                  |> Time_ns.Span.of_int_ns
                  |> Time_ns.Ofday.of_span_since_start_of_day_exn))
    | Duration time_unit, Span ->
      let time_unit_mult = time_unit_mult time_unit in
      Option.value_exn (A.array_time64_ns_values_opt t.data)
      |> Array.map
           ~f:
             (Option.map ~f:(fun d ->
                  Int64.to_int_exn d * time_unit_mult |> Time_ns.Span.of_int_ns))
    | Timestamp (time_unit, _zone), Time ->
      let time_unit_mult = time_unit_mult time_unit in
      Option.value_exn (A.array_timestamp_ns_values_opt t.data)
      |> Array.map
           ~f:
             (Option.map ~f:(fun ts ->
                  Int64.to_int_exn ts * time_unit_mult |> Time_ns.of_int_ns_since_epoch))
    | data_type, _data_type ->
      [%message "unsupported data type" (data_type : A.data_type)] |> raise_s

  let sexp_of (type a) (t : a t) =
    match t.data_type with
    | Int -> to_array t |> [%sexp_of: int array]
    | Float -> to_array t |> [%sexp_of: float array]
    | Date -> to_array t |> [%sexp_of: Date.t array]
    | Time -> to_array t |> [%sexp_of: Time_ns_unix.t array]
    | Ofday -> to_array t |> [%sexp_of: Time_ns.Ofday.t array]
    | Span -> to_array t |> [%sexp_of: Time_ns.Span.t array]
    | String -> to_array t |> [%sexp_of: string array]

  let sexp_of_packed (P t) = sexp_of t
end

module Record_batch = struct
  type t =
    { data : A.record_batch
    ; schema : A.schema
    ; column_indexes : int String.Map.t
    }

  let of_record_batch data =
    let schema = A.record_batch_schema data in
    let column_indexes =
      Array.to_list schema.fields
      |> List.mapi ~f:(fun index field -> field.A.name, index)
      |> String.Map.of_alist_exn
    in
    { data; schema; column_indexes }

  let record_batch t = t.data
  let num_rows t = A.record_batch_num_rows t.data
  let num_columns t = A.record_batch_num_columns t.data

  let create columns =
    Array.of_list_map columns ~f:(fun (name, Column.P column) -> name, column.data)
    |> A.record_batch_create
    |> Result.map ~f:of_record_batch

  let debug_string t = A.record_batch_debug t.data
  let schema t = t.schema

  let concat ts =
    Array.of_list_map ts ~f:(fun t -> t.data)
    |> A.record_batch_concat
    |> Result.map ~f:of_record_batch

  let write_parquet t filename = A.record_batch_write_parquet t.data filename

  let read_parquet ?column_names filename =
    A.file_reader filename
    >>= fun file_reader ->
    let metadata = A.file_reader_parquet_metadata file_reader |> ok_exn in
    let record_reader =
      match column_names with
      | None -> A.get_record_reader file_reader metadata.num_rows
      | Some column_names ->
        let column_names = String.Hash_set.of_list column_names in
        A.file_reader_schema file_reader
        >>= fun schema ->
        let column_indexes =
          Array.filter_mapi schema.fields ~f:(fun index field ->
              if Hash_set.mem column_names field.A.name
              then (
                Hash_set.remove column_names field.A.name;
                Some index)
              else None)
        in
        if Hash_set.is_empty column_names
        then A.get_record_reader_by_columns file_reader column_indexes metadata.num_rows
        else
          Error
            (sprintf
               "missing column names %s"
               ([%sexp_of: String.Hash_set.t] column_names |> Sexp.to_string_mach))
    in
    record_reader
    >>= fun record_reader ->
    let rec loop acc =
      match A.record_reader_next record_reader with
      | None -> List.rev acc |> Array.of_list |> A.record_batch_concat
      | Some (Error _ as err) -> err
      | Some (Ok ok) -> loop (ok :: acc)
    in
    loop [] |> Result.map ~f:of_record_batch

  let mem t column_name = Map.mem t.column_indexes column_name

  let column t column_name =
    match Map.find t.column_indexes column_name with
    | Some index -> A.record_batch_column t.data index |> Column.of_array_ref
    | None ->
      [%message
        "unable to find column"
          (column_name : string)
          ~existing_columns:(Map.keys t.column_indexes : string list)]
      |> raise_s

  let columns t =
    Array.to_list t.schema.fields
    |> List.mapi ~f:(fun index field ->
           field.A.name, A.record_batch_column t.data index |> Column.of_array_ref)
end

module Reader = struct
  type t =
    { record_reader : A.record_reader
    ; file_reader : A.file_reader
    }

  let create ?column_names filename ~batch_size =
    A.file_reader filename
    >>= fun file_reader ->
    let record_reader =
      match column_names with
      | None -> A.get_record_reader file_reader batch_size
      | Some column_names ->
        let column_names = String.Hash_set.of_list column_names in
        A.file_reader_schema file_reader
        >>= fun schema ->
        let column_indexes =
          Array.filter_mapi schema.fields ~f:(fun index field ->
              if Hash_set.mem column_names field.A.name
              then (
                Hash_set.remove column_names field.A.name;
                Some index)
              else None)
        in
        if Hash_set.is_empty column_names
        then A.get_record_reader_by_columns file_reader column_indexes batch_size
        else
          Error
            (sprintf
               "missing column names %s"
               ([%sexp_of: String.Hash_set.t] column_names |> Sexp.to_string_mach))
    in
    record_reader >>= fun record_reader -> Ok { record_reader; file_reader }

  let schema t = A.file_reader_schema t.file_reader

  let next t =
    match A.record_reader_next t.record_reader with
    | None -> `Eof
    | Some record_batch ->
      `Batch (Result.map record_batch ~f:Record_batch.of_record_batch)

  let close t =
    A.file_reader_close t.file_reader;
    A.record_reader_close t.record_reader

  let with_reader ?column_names filename ~batch_size ~f =
    create ?column_names filename ~batch_size
    >>= fun t -> Exn.protect ~f:(fun () -> Ok (f t)) ~finally:(fun () -> close t)
end

module Writer = struct
  module Writer = struct
    type t =
      | Not_initialized
      | Writer of A.file_writer
      | Closed
  end

  type t =
    { mutable writer : Writer.t
    ; filename : string
    }

  let create filename = { writer = Not_initialized; filename }

  let append t record_batch =
    match t.writer with
    | Writer writer -> A.writer_write writer record_batch.Record_batch.data
    | Closed -> Error "writer has been closed"
    | Not_initialized ->
      A.writer_new record_batch.data t.filename
      >>| fun writer -> t.writer <- Writer writer

  let close t =
    (match t.writer with
    | Writer writer -> A.writer_close writer
    | Not_initialized | Closed -> Ok ())
    >>| fun () -> t.writer <- Closed

  let with_writer filename ~f =
    let t = create filename in
    Exn.protect ~f:(fun () -> Ok (f t)) ~finally:(fun () -> ignore (close t : _ Result.t))
end

module Csv_reader = struct
  type t = A.csv_file_reader

  let create ?infer_size filename ~batch_size =
    A.csv_reader_new filename batch_size infer_size

  let next t =
    match A.csv_reader_next t with
    | None -> `Eof
    | Some record_batch ->
      `Batch (Result.map record_batch ~f:Record_batch.of_record_batch)

  let close t = A.csv_reader_close t

  let with_reader ?infer_size filename ~batch_size ~f =
    create ?infer_size filename ~batch_size
    >>= fun t -> Exn.protect ~f:(fun () -> Ok (f t)) ~finally:(fun () -> close t)
end

module Csv_writer = struct
  type t = A.csv_file_writer

  let create filename = A.csv_writer_new filename
  let append t record_batch = A.csv_writer_write t record_batch.Record_batch.data
  let close t = A.csv_writer_close t

  let with_writer filename ~f =
    create filename
    >>= fun t -> Exn.protect ~f:(fun () -> Ok (f t)) ~finally:(fun () -> close t)
end
