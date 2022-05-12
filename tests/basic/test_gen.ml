module Ffi = struct
  external add_one
    : int -> int
    = "__ocaml_ffifoo__add_one"
  ;;

  external add_i64
    : Int64.t -> Int64.t -> Int64.t
    = "__ocaml_ffi_add_i64"
  ;;

  external str_format
    : (int * int) -> string -> string
    = "__ocaml_ffi_str_format"
  ;;

  external pair
    : (string * float * (int * int)) -> string
    = "__ocaml_ffi_pair"
  ;;

  external option_result
    : int option -> string -> (int, string) Result.t
    = "__ocaml_ffi_option_result"
  ;;

  external vec_add
    : int array -> int -> int array
    = "__ocaml_ffi_vec_add"
  ;;

end
module Ffi2 = struct
  type my_vec;;
  external vec_new
    : unit -> my_vec
    = "__ocaml_ffi2_vec_new"
  ;;

  external vec_push
    : my_vec -> int -> unit
    = "__ocaml_ffi2_vec_push"
  ;;

  external vec_content
    : my_vec -> Int64.t array
    = "__ocaml_ffi2_vec_content"
  ;;

end
module Ffi3 = struct
open! Sexplib.Conv
  type my_enum =
  | NoArg
  | OneArg of int
  | TwoArgs of int * string
  | StructArgs of { x: int; y: string }
  | Rec of my_enum
  [@@boxed];;
  type my_struct = {
    x: int;
    y: string;
    z: (int * string option * float);
    zs: float array;
  } [@@boxed][@@deriving sexp];;
  external mystruct_to_string
    : my_struct -> string
    = "__ocaml_ffi3_mystruct_to_string"
  ;;

  external mystruct_add_x
    : my_struct -> int -> my_struct
    = "__ocaml_ffi3_mystruct_add_x"
  ;;

  external myenum_to_string
    : my_enum -> string
    = "__ocaml_ffi3_myenum_to_string"
  ;;

  external myenum_add_x
    : my_enum -> int -> my_enum
    = "__ocaml_ffi3_myenum_add_x"
  ;;

end
module Ffi4 = struct
  external map_callback
    : int array -> ((int) -> (string)) -> string array
    = "__ocaml_ffi4_map_callback"
  ;;

  external sum_n
    : int -> (unit -> (int)) -> int
    = "__ocaml_ffi4_sum_n"
  ;;

end
module Ffi6 = struct
  type c;;
  external create_foo2
    : int -> c
    = "__ocaml_ffi6_create_foo2"
  ;;

  external foo2_to_string
    : c -> string
    = "__ocaml_ffi6_foo2_to_string"
  ;;

end
module Ffi7 = struct
  type compact;;
  external generate
    : int -> ((((Int64.t * Int64.t) * compact) * Int64.t) * Int64.t)
    = "__ocaml_ffi7_generate"
  ;;

end
module Ffi_double_array = struct
open! Sexplib.Conv
  type quaternion = {
    a: float;
    b: float;
    c: float;
    d: float;
  } [@@boxed][@@deriving sexp];;
  external add_ones
    : float array -> float array
    = "__ocaml_ffi_double_array_add_ones"
  ;;

  external add_quat
    : quaternion -> quaternion -> quaternion
    = "__ocaml_ffi_double_array_add_quat"
  ;;

  external create_quat
    : float -> float -> float -> float -> quaternion
    = "__ocaml_ffi_double_array_create_quat"
  ;;

end
