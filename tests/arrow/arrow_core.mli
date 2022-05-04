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

  val equal : _ t -> _ t -> bool
end

module Column : sig
  type _ t
  type packed = P : _ t -> packed [@@deriving sexp_of]

  val extract : packed -> 'a Data_type.t -> 'a t option
  val extract_exn : packed -> 'a Data_type.t -> 'a t

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

  (* Creation *)
  module C : sig
    val time : Time_ns.t array -> zone:Time_ns_unix.Zone.t -> Time_ns.t t
    val date : Date.t array -> Date.t t
    val span : Time_ns.Span.t array -> Time_ns.Span.t t
    val ofday : Time_ns.Ofday.t array -> Time_ns.Ofday.t t
    val int64 : int array -> int t
    val int32 : int array -> int t
    val float64 : float array -> float t
    val float32 : float array -> float t
  end
end

module Record_batch : sig
  type t

  val create : (string * Column.packed) list -> t result
  val debug_string : t -> string
  val schema : t -> A.schema
  val concat : t list -> t result
  val mem : t -> string -> bool
  val column : t -> string -> Column.packed

  (* Parquet read/write. *)
  val write_parquet : t -> string -> unit result
  val read_parquet : ?column_names:string list -> string -> t result

  (* [record_batch] conversion. *)
  val of_record_batch : A.record_batch -> t
  val record_batch : t -> A.record_batch
end

module Reader : sig
  type t

  val create : ?column_names:string list -> string -> batch_size:int -> t result
  val next : t -> [ `Eof | `Batch of Record_batch.t result ]
  val close : t -> unit
end

module Writer : sig
  type t

  val create : string -> t
  val append : t -> Record_batch.t -> unit result
  val close : t -> unit result
end
