open! Core
module A = Arrow_gen.Arrow

val set_default_zone : Time_ns_unix.Zone.t -> unit

type 'a result = ('a, string) Result.t

val ok_exn : 'a result -> 'a

module Data_type : sig
  type _ t =
    | Int : int t
    | Float : float t
    | Date : Date.t t
    | Time : Time_ns.t t
    | Ofday : Time_ns.Ofday.t t
    | Span : Time_ns.Span.t t
    | String : string t
  [@@deriving sexp_of]
end

module Column : sig
  type _ t
  type packed = P : _ t -> packed [@@deriving sexp_of]

  (* Conversion from/to OCaml data *)
  val of_array : 'a Data_type.t -> 'a array -> 'a t
  val to_array : 'a t -> 'a array

  (* array_ref conversion *)
  val data : _ t -> A.array_ref
  val of_data : A.array_ref -> packed

  (* Accessors *)
  val data_type : 'a t -> 'a Data_type.t
  val len : _ t -> int
  val null_count : _ t -> int
end

module Record_batch : sig
  type t

  val create : (string * Column.packed) list -> t result
  val debug_string : t -> string
  val schema : t -> A.schema
  val concat : t list -> t result

  (* Parquet read/write. *)
  val write_parquet : t -> string -> unit result
end
