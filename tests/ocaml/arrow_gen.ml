module Arrow = struct
open! Sexplib.Conv
type reader
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
    data_type: string;
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
