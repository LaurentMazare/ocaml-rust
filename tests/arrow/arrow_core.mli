open! Core
module A = Arrow_gen.Arrow

module Schema : sig
  module Field : sig
    type t = A.schema_field [@@deriving sexp]
  end

  type t = A.schema [@@deriving sexp]
end

val set_default_zone : Time_ns_unix.Zone.t -> unit

type ('a, 'b) ba = ('a, 'b, Bigarray.c_layout) Bigarray.Array1.t
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
    | Null : unit t
  [@@deriving sexp_of]

  val equal : _ t -> _ t -> bool
end

module Column : sig
  type _ t
  type packed = P : _ t -> packed [@@deriving sexp_of]

  val extract : packed -> 'a Data_type.t -> 'a t option
  val extract_exn : packed -> 'a Data_type.t -> 'a t

  (* Conversion from/to OCaml data. *)
  val of_array : 'a Data_type.t -> 'a array -> 'a t

  (* [to_array ~default t] uses [default] for null values from the
     Arrow array. *)
  val to_array : ?default:'a -> 'a t -> 'a array
  val to_array_opt : 'a t -> 'a option array

  (* array_ref conversion *)
  val array_ref : _ t -> A.array_ref
  val of_array_ref : A.array_ref -> packed

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
    val int64_ba : (int, Bigarray.int64_elt) ba -> int t
    val int32_ba : (int, Bigarray.int32_elt) ba -> int t
    val float64_ba : (float, Bigarray.float64_elt) ba -> float t
    val float32_ba : (float, Bigarray.float32_elt) ba -> float t
  end

  module Bigarray : sig
    type t =
      | Int32 of (int, Bigarray.int32_elt) ba
      | Int64 of (int, Bigarray.int64_elt) ba
      | Float32 of (float, Bigarray.float32_elt) ba
      | Float64 of (float, Bigarray.float64_elt) ba

    val get : packed -> t option
  end
end

module Record_batch : sig
  type t

  val create : (string * Column.packed) list -> t result
  val debug_string : t -> string
  val schema : t -> Schema.t
  val concat : t list -> t result
  val mem : t -> string -> bool
  val columns : t -> (string * Column.packed) list
  val column : t -> string -> Column.packed
  val num_rows : t -> int
  val num_columns : t -> int

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
  val schema : t -> Schema.t result
  val parquet_metadata : t -> A.metadata result
  val next : t -> [ `Eof | `Batch of Record_batch.t result ]
  val close : t -> unit

  val with_reader
    :  ?column_names:string list
    -> string
    -> batch_size:int
    -> f:(t -> 'a)
    -> 'a result
end

module Writer : sig
  type t

  val create : string -> t
  val append : t -> Record_batch.t -> unit result
  val close : t -> unit result
  val with_writer : string -> f:(t -> 'a) -> 'a result
end

module Csv_reader : sig
  type t

  val create : ?infer_size:int -> string -> batch_size:int -> t result
  val next : t -> [ `Eof | `Batch of Record_batch.t result ]
  val close : t -> unit

  val with_reader
    :  ?infer_size:int
    -> string
    -> batch_size:int
    -> f:(t -> 'a)
    -> 'a result
end

module Csv_writer : sig
  type t

  val create : string -> t result
  val append : t -> Record_batch.t -> unit result
  val close : t -> unit
  val with_writer : string -> f:(t -> 'a) -> 'a result
end
