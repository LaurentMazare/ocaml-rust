module Arrow = struct
open! Sexplib.Conv
  type file_reader;;
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

  external metadata_as_string
    : file_reader -> string
    = "__ocaml_arrow_metadata_as_string"
  ;;

  external parquet_metadata
    : file_reader -> metadata
    = "__ocaml_arrow_parquet_metadata"
  ;;

  external schema
    : file_reader -> (schema, string) Result.t
    = "__ocaml_arrow_schema"
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

  external array_f32_values
    : array_ref -> float array option
    = "__ocaml_arrow_array_f32_values"
  ;;

  external array_f64_values
    : array_ref -> float array option
    = "__ocaml_arrow_array_f64_values"
  ;;

  external array_string_values
    : array_ref -> string option array option
    = "__ocaml_arrow_array_string_values"
  ;;

  external array_large_string_values
    : array_ref -> string option array option
    = "__ocaml_arrow_array_large_string_values"
  ;;

end
