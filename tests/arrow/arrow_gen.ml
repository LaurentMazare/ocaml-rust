module Arrow = struct
open! Sexplib.Conv
  type file_reader;;
  type file_writer;;
  type csv_file_reader;;
  type csv_file_writer;;
  type record_reader;;
  type record_batch;;
  type array_ref;;
  type interval_unit =
  | YearMonth
  | DayTime
  | MonthDayNano
  [@@boxed][@@deriving sexp];;
  type time_unit =
  | Second
  | Millisecond
  | Microsecond
  | Nanosecond
  [@@boxed][@@deriving sexp];;
  type data_type =
  | Null
  | Boolean
  | Int8
  | Int16
  | Int32
  | Int64
  | UInt8
  | UInt16
  | UInt32
  | UInt64
  | Float16
  | Float32
  | Float64
  | Timestamp of time_unit * string option
  | Date32
  | Date64
  | Time32 of time_unit
  | Time64 of time_unit
  | Duration of time_unit
  | Interval of interval_unit
  | Binary
  | FixedSizeBinary of int
  | LargeBinary
  | Utf8
  | LargeUtf8
  | List
  | FixedSizeList
  | LargeList
  | Struct
  | Union
  | Dictionary of data_type * data_type
  | Decimal of int * int
  | Map
  [@@boxed][@@deriving sexp];;
  type row_group_metadata = {
    num_columns: int;
    num_rows: int;
    total_byte_size: int;
  } [@@boxed][@@deriving sexp];;
  type metadata = {
    num_rows: int;
    version: int;
    created_by: string option;
    row_groups: row_group_metadata array;
  } [@@boxed][@@deriving sexp];;
  type schema_field = {
    name: string;
    data_type: data_type;
    nullable: bool;
  } [@@boxed][@@deriving sexp];;
  type schema = {
    fields: schema_field array;
    metadata: (string * string) array;
  } [@@boxed][@@deriving sexp];;
  external file_reader
    : string -> (file_reader, string) Result.t
    = "__ocaml_arrow_file_reader"
  ;;

  external file_reader_metadata_as_string
    : file_reader -> (string, string) Result.t
    = "__ocaml_arrow_file_reader_metadata_as_string"
  ;;

  external file_reader_parquet_metadata
    : file_reader -> (metadata, string) Result.t
    = "__ocaml_arrow_file_reader_parquet_metadata"
  ;;

  external file_reader_schema
    : file_reader -> (schema, string) Result.t
    = "__ocaml_arrow_file_reader_schema"
  ;;

  external file_reader_close
    : file_reader -> unit
    = "__ocaml_arrow_file_reader_close"
  ;;

  external get_record_reader
    : file_reader -> int -> (record_reader, string) Result.t
    = "__ocaml_arrow_get_record_reader"
  ;;

  external get_record_reader_by_columns
    : file_reader -> int array -> int -> (record_reader, string) Result.t
    = "__ocaml_arrow_get_record_reader_by_columns"
  ;;

  external record_reader_next
    : record_reader -> (record_batch, string) Result.t option
    = "__ocaml_arrow_record_reader_next"
  ;;

  external record_reader_close
    : record_reader -> unit
    = "__ocaml_arrow_record_reader_close"
  ;;

  external record_batch_create
    : (string * array_ref) array -> (record_batch, string) Result.t
    = "__ocaml_arrow_record_batch_create"
  ;;

  external record_batch_debug
    : record_batch -> string
    = "__ocaml_arrow_record_batch_debug"
  ;;

  external record_batch_schema
    : record_batch -> schema
    = "__ocaml_arrow_record_batch_schema"
  ;;

  external record_batch_num_rows
    : record_batch -> int
    = "__ocaml_arrow_record_batch_num_rows"
  ;;

  external record_batch_num_columns
    : record_batch -> int
    = "__ocaml_arrow_record_batch_num_columns"
  ;;

  external record_batch_column
    : record_batch -> int -> array_ref
    = "__ocaml_arrow_record_batch_column"
  ;;

  external record_batch_write_parquet
    : record_batch -> string -> (unit, string) Result.t
    = "__ocaml_arrow_record_batch_write_parquet"
  ;;

  external record_batch_slice
    : record_batch -> int -> int -> record_batch
    = "__ocaml_arrow_record_batch_slice"
  ;;

  external record_batch_concat
    : record_batch array -> (record_batch, string) Result.t
    = "__ocaml_arrow_record_batch_concat"
  ;;

  external writer_new
    : record_batch -> string -> (file_writer, string) Result.t
    = "__ocaml_arrow_writer_new"
  ;;

  external writer_write
    : file_writer -> record_batch -> (unit, string) Result.t
    = "__ocaml_arrow_writer_write"
  ;;

  external writer_close
    : file_writer -> (unit, string) Result.t
    = "__ocaml_arrow_writer_close"
  ;;

  external csv_writer_new
    : string -> (csv_file_writer, string) Result.t
    = "__ocaml_arrow_csv_writer_new"
  ;;

  external csv_writer_write
    : csv_file_writer -> record_batch -> (unit, string) Result.t
    = "__ocaml_arrow_csv_writer_write"
  ;;

  external csv_writer_close
    : csv_file_writer -> unit
    = "__ocaml_arrow_csv_writer_close"
  ;;

  external csv_reader_new
    : string -> int -> int option -> (csv_file_reader, string) Result.t
    = "__ocaml_arrow_csv_reader_new"
  ;;

  external csv_reader_next
    : csv_file_reader -> (record_batch, string) Result.t option
    = "__ocaml_arrow_csv_reader_next"
  ;;

  external csv_reader_close
    : csv_file_reader -> unit
    = "__ocaml_arrow_csv_reader_close"
  ;;

  external array_data_type
    : array_ref -> data_type
    = "__ocaml_arrow_array_data_type"
  ;;

  external array_len
    : array_ref -> int
    = "__ocaml_arrow_array_len"
  ;;

  external array_null_count
    : array_ref -> int
    = "__ocaml_arrow_array_null_count"
  ;;

  external array_timestamp_ns_from_with_zone
    : Int64.t array -> string option -> array_ref
    = "__ocaml_arrow_array_timestamp_ns_from_with_zone"
  ;;

  external array_string_from
    : string array -> array_ref
    = "__ocaml_arrow_array_string_from"
  ;;

  external array_large_string_from
    : string array -> array_ref
    = "__ocaml_arrow_array_large_string_from"
  ;;

  external array_string_values
    : array_ref -> string -> string array option
    = "__ocaml_arrow_array_string_values"
  ;;

  external array_large_string_values
    : array_ref -> string -> string array option
    = "__ocaml_arrow_array_large_string_values"
  ;;

  external array_string_values_opt
    : array_ref -> string option array option
    = "__ocaml_arrow_array_string_values_opt"
  ;;

  external array_large_string_values_opt
    : array_ref -> string option array option
    = "__ocaml_arrow_array_large_string_values_opt"
  ;;

module Array_char = struct
  external from_ba
    : (char, Bigarray.int8_unsigned_elt, Bigarray.c_layout) Bigarray.Array1.t -> array_ref
    = "__ocaml_arrowarray_char__from_ba"
  ;;

  external from
    : char array -> array_ref
    = "__ocaml_arrowarray_char__from"
  ;;

  external values
    : array_ref -> char -> char array option
    = "__ocaml_arrowarray_char__values"
  ;;

  external values_ba
    : array_ref -> char -> (char, Bigarray.int8_unsigned_elt, Bigarray.c_layout) Bigarray.Array1.t option
    = "__ocaml_arrowarray_char__values_ba"
  ;;

end
module Array_date32 = struct
  external from_ba
    : (int, Bigarray.int32_elt, Bigarray.c_layout) Bigarray.Array1.t -> array_ref
    = "__ocaml_arrowarray_date32__from_ba"
  ;;

  external from
    : Int32.t array -> array_ref
    = "__ocaml_arrowarray_date32__from"
  ;;

  external values
    : array_ref -> Int32.t -> Int32.t array option
    = "__ocaml_arrowarray_date32__values"
  ;;

  external values_opt
    : array_ref -> Int32.t option array option
    = "__ocaml_arrowarray_date32__values_opt"
  ;;

  external values_ba
    : array_ref -> Int32.t -> (int, Bigarray.int32_elt, Bigarray.c_layout) Bigarray.Array1.t option
    = "__ocaml_arrowarray_date32__values_ba"
  ;;

end
module Array_date64 = struct
  external from_ba
    : (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t -> array_ref
    = "__ocaml_arrowarray_date64__from_ba"
  ;;

  external from
    : Int64.t array -> array_ref
    = "__ocaml_arrowarray_date64__from"
  ;;

  external values
    : array_ref -> Int64.t -> Int64.t array option
    = "__ocaml_arrowarray_date64__values"
  ;;

  external values_opt
    : array_ref -> Int64.t option array option
    = "__ocaml_arrowarray_date64__values_opt"
  ;;

  external values_ba
    : array_ref -> Int64.t -> (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t option
    = "__ocaml_arrowarray_date64__values_ba"
  ;;

end
module Array_duration_ns = struct
  external from_ba
    : (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t -> array_ref
    = "__ocaml_arrowarray_duration_ns__from_ba"
  ;;

  external from
    : Int64.t array -> array_ref
    = "__ocaml_arrowarray_duration_ns__from"
  ;;

  external values
    : array_ref -> Int64.t -> Int64.t array option
    = "__ocaml_arrowarray_duration_ns__values"
  ;;

  external values_opt
    : array_ref -> Int64.t option array option
    = "__ocaml_arrowarray_duration_ns__values_opt"
  ;;

  external values_ba
    : array_ref -> Int64.t -> (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t option
    = "__ocaml_arrowarray_duration_ns__values_ba"
  ;;

end
module Array_f32 = struct
  external from_ba
    : (float, Bigarray.float32_elt, Bigarray.c_layout) Bigarray.Array1.t -> array_ref
    = "__ocaml_arrowarray_f32__from_ba"
  ;;

  external from
    : float array -> array_ref
    = "__ocaml_arrowarray_f32__from"
  ;;

  external values
    : array_ref -> float -> float array option
    = "__ocaml_arrowarray_f32__values"
  ;;

  external values_opt
    : array_ref -> float option array option
    = "__ocaml_arrowarray_f32__values_opt"
  ;;

  external values_ba
    : array_ref -> float -> (float, Bigarray.float32_elt, Bigarray.c_layout) Bigarray.Array1.t option
    = "__ocaml_arrowarray_f32__values_ba"
  ;;

end
module Array_f64 = struct
  external from_ba
    : (float, Bigarray.float64_elt, Bigarray.c_layout) Bigarray.Array1.t -> array_ref
    = "__ocaml_arrowarray_f64__from_ba"
  ;;

  external from
    : float array -> array_ref
    = "__ocaml_arrowarray_f64__from"
  ;;

  external values
    : array_ref -> float -> float array option
    = "__ocaml_arrowarray_f64__values"
  ;;

  external values_opt
    : array_ref -> float option array option
    = "__ocaml_arrowarray_f64__values_opt"
  ;;

  external values_ba
    : array_ref -> float -> (float, Bigarray.float64_elt, Bigarray.c_layout) Bigarray.Array1.t option
    = "__ocaml_arrowarray_f64__values_ba"
  ;;

end
module Array_i32 = struct
  external from_ba
    : (int, Bigarray.int32_elt, Bigarray.c_layout) Bigarray.Array1.t -> array_ref
    = "__ocaml_arrowarray_i32__from_ba"
  ;;

  external from
    : Int32.t array -> array_ref
    = "__ocaml_arrowarray_i32__from"
  ;;

  external values
    : array_ref -> Int32.t -> Int32.t array option
    = "__ocaml_arrowarray_i32__values"
  ;;

  external values_opt
    : array_ref -> Int32.t option array option
    = "__ocaml_arrowarray_i32__values_opt"
  ;;

  external values_ba
    : array_ref -> Int32.t -> (int, Bigarray.int32_elt, Bigarray.c_layout) Bigarray.Array1.t option
    = "__ocaml_arrowarray_i32__values_ba"
  ;;

end
module Array_i64 = struct
  external from_ba
    : (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t -> array_ref
    = "__ocaml_arrowarray_i64__from_ba"
  ;;

  external from
    : Int64.t array -> array_ref
    = "__ocaml_arrowarray_i64__from"
  ;;

  external values
    : array_ref -> Int64.t -> Int64.t array option
    = "__ocaml_arrowarray_i64__values"
  ;;

  external values_opt
    : array_ref -> Int64.t option array option
    = "__ocaml_arrowarray_i64__values_opt"
  ;;

  external values_ba
    : array_ref -> Int64.t -> (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t option
    = "__ocaml_arrowarray_i64__values_ba"
  ;;

end
module Array_time64_ns = struct
  external from_ba
    : (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t -> array_ref
    = "__ocaml_arrowarray_time64_ns__from_ba"
  ;;

  external from
    : Int64.t array -> array_ref
    = "__ocaml_arrowarray_time64_ns__from"
  ;;

  external values
    : array_ref -> Int64.t -> Int64.t array option
    = "__ocaml_arrowarray_time64_ns__values"
  ;;

  external values_opt
    : array_ref -> Int64.t option array option
    = "__ocaml_arrowarray_time64_ns__values_opt"
  ;;

  external values_ba
    : array_ref -> Int64.t -> (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t option
    = "__ocaml_arrowarray_time64_ns__values_ba"
  ;;

end
module Array_time64_us = struct
  external from_ba
    : (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t -> array_ref
    = "__ocaml_arrowarray_time64_us__from_ba"
  ;;

  external from
    : Int64.t array -> array_ref
    = "__ocaml_arrowarray_time64_us__from"
  ;;

  external values
    : array_ref -> Int64.t -> Int64.t array option
    = "__ocaml_arrowarray_time64_us__values"
  ;;

  external values_opt
    : array_ref -> Int64.t option array option
    = "__ocaml_arrowarray_time64_us__values_opt"
  ;;

  external values_ba
    : array_ref -> Int64.t -> (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t option
    = "__ocaml_arrowarray_time64_us__values_ba"
  ;;

end
module Array_timestamp_ms = struct
  external from_ba
    : (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t -> array_ref
    = "__ocaml_arrowarray_timestamp_ms__from_ba"
  ;;

  external from
    : Int64.t array -> array_ref
    = "__ocaml_arrowarray_timestamp_ms__from"
  ;;

  external values
    : array_ref -> Int64.t -> Int64.t array option
    = "__ocaml_arrowarray_timestamp_ms__values"
  ;;

  external values_opt
    : array_ref -> Int64.t option array option
    = "__ocaml_arrowarray_timestamp_ms__values_opt"
  ;;

  external values_ba
    : array_ref -> Int64.t -> (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t option
    = "__ocaml_arrowarray_timestamp_ms__values_ba"
  ;;

end
module Array_timestamp_ns = struct
  external from_ba
    : (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t -> array_ref
    = "__ocaml_arrowarray_timestamp_ns__from_ba"
  ;;

  external from
    : Int64.t array -> array_ref
    = "__ocaml_arrowarray_timestamp_ns__from"
  ;;

  external values
    : array_ref -> Int64.t -> Int64.t array option
    = "__ocaml_arrowarray_timestamp_ns__values"
  ;;

  external values_opt
    : array_ref -> Int64.t option array option
    = "__ocaml_arrowarray_timestamp_ns__values_opt"
  ;;

  external values_ba
    : array_ref -> Int64.t -> (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t option
    = "__ocaml_arrowarray_timestamp_ns__values_ba"
  ;;

end
module Array_timestamp_s = struct
  external from_ba
    : (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t -> array_ref
    = "__ocaml_arrowarray_timestamp_s__from_ba"
  ;;

  external from
    : Int64.t array -> array_ref
    = "__ocaml_arrowarray_timestamp_s__from"
  ;;

  external values
    : array_ref -> Int64.t -> Int64.t array option
    = "__ocaml_arrowarray_timestamp_s__values"
  ;;

  external values_opt
    : array_ref -> Int64.t option array option
    = "__ocaml_arrowarray_timestamp_s__values_opt"
  ;;

  external values_ba
    : array_ref -> Int64.t -> (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t option
    = "__ocaml_arrowarray_timestamp_s__values_ba"
  ;;

end
module Array_timestamp_us = struct
  external from_ba
    : (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t -> array_ref
    = "__ocaml_arrowarray_timestamp_us__from_ba"
  ;;

  external from
    : Int64.t array -> array_ref
    = "__ocaml_arrowarray_timestamp_us__from"
  ;;

  external values
    : array_ref -> Int64.t -> Int64.t array option
    = "__ocaml_arrowarray_timestamp_us__values"
  ;;

  external values_opt
    : array_ref -> Int64.t option array option
    = "__ocaml_arrowarray_timestamp_us__values_opt"
  ;;

  external values_ba
    : array_ref -> Int64.t -> (int, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t option
    = "__ocaml_arrowarray_timestamp_us__values_ba"
  ;;

end
end
