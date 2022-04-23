module Arrow = struct
open! Sexplib.Conv
type reader
  external reader
    : string -> (reader, string) Result.t
    = "__ocaml_arrow_reader"
  ;;

  external metadata_as_string
    : reader -> string
    = "__ocaml_arrow_metadata_as_string"
  ;;

end
