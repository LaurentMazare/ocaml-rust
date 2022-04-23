module Arrow = struct
open! Sexplib.Conv
  type reader;;
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
  | Dictionary
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
  external reader
    : string -> (reader, string) Result.t
    = "__ocaml_arrow_reader"
  ;;

  external metadata_as_string
    : reader -> string
    = "__ocaml_arrow_metadata_as_string"
  ;;

  external parquet_metadata
    : reader -> metadata
    = "__ocaml_arrow_parquet_metadata"
  ;;

  external schema
    : reader -> (schema, string) Result.t
    = "__ocaml_arrow_schema"
  ;;

end
