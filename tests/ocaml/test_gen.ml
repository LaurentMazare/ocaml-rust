
type isize = int;;
type i64 = Int64.t;;
type f64 = float;;

module Ffi = struct
  external add_one
    : isize -> isize
    = "__ocaml_ffi_add_one"
  ;;

  external add_i64
    : i64 -> i64 -> i64
    = "__ocaml_ffi_add_i64"
  ;;

  external str_format
    : (isize * isize) -> string -> string
    = "__ocaml_ffi_str_format"
  ;;

  external pair
    : (string * f64 * (isize * isize)) -> string
    = "__ocaml_ffi_pair"
  ;;

  external option_result
    : isize option -> string -> (isize, string) Result.t
    = "__ocaml_ffi_option_result"
  ;;

  external vec_add
    : isize array -> isize -> isize array
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
    : my_vec -> isize -> unit
    = "__ocaml_ffi2_vec_push"
  ;;

  external vec_content
    : my_vec -> i64 array
    = "__ocaml_ffi2_vec_content"
  ;;

end
module Ffi3 = struct
  type my_enum =
  | NoArg
  | OneArg of isize
  | TwoArgs of isize * string
  | StructArgs of { x: isize; y: string }
  [@@boxed];;
  type my_struct = {
    x: isize;
    y: string;
    z: (isize * string option * f64);
    zs: f64 array;
  } [@@boxed];;
  external mystruct_to_string
    : my_struct -> string
    = "__ocaml_ffi3_mystruct_to_string"
  ;;

  external mystruct_add_x
    : my_struct -> isize -> my_struct
    = "__ocaml_ffi3_mystruct_add_x"
  ;;

  external myenum_to_string
    : my_enum -> string
    = "__ocaml_ffi3_myenum_to_string"
  ;;

  external myenum_add_x
    : my_enum -> isize -> my_enum
    = "__ocaml_ffi3_myenum_add_x"
  ;;

end
module Ffi4 = struct
  external map_callback
    : isize array -> ((isize) -> (string)) -> string array
    = "__ocaml_ffi4_map_callback"
  ;;

  external sum_n
    : isize -> (unit -> (isize)) -> isize
    = "__ocaml_ffi4_sum_n"
  ;;

end
