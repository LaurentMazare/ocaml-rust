open! Core
module A = Arrow_gen.Arrow

module Data_type : sig
  type _ t =
    | Int32 : int t
    | Int64 : int t
    | Float32 : float t
    | Float64 : float t
    | Date32 : Date.t t
    | Timestamp : Time_ns.t t
    | Time64 : Time_ns.Ofday.t t
    | Duration : Time_ns.Span.t t
    | String : string t
end

module Column : sig
  type _ t
  type packed = P : _ t -> packed

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
