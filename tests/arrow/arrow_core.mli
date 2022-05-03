open! Core
module A = Arrow_gen.Arrow

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
