(* More idiomatic wrapper around the Arrow api.
   - Use OCaml native types rather than the specialized versions.
   - Handle time/date types.
   *)
open! Core
module A = Arrow_gen.Arrow

let zone = ref "utc"

module Data_type = struct
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

module Column = struct
  type 'a t =
    { data : A.array_ref
    ; data_type : 'a Data_type.t
    }

  type packed = P : _ t -> packed

  let data t = t.data

  let of_data data =
    match A.array_data_type data with
    | Int32 -> P { data; data_type = Int32 }
    | data_type -> [%message "unsupported data type" (data_type : A.data_type)] |> raise_s

  let data_type t = t.data_type
  let len t = A.array_len t.data
  let null_count t = A.array_null_count t.data

  let of_array (type a) (data_type : a Data_type.t) (data : a array) =
    let data =
      match data_type with
      | Int32 -> Array.map data ~f:Int32.of_int_exn |> A.array_i32_from
      | Int64 -> Array.map data ~f:Int64.of_int_exn |> A.array_i64_from
      | Float32 -> A.array_f32_from data
      | Float64 -> A.array_f64_from data
      | Date32 ->
        Array.map data ~f:(fun d -> Date.(diff d unix_epoch) |> Int32.of_int_exn)
        |> A.array_date32_from
      | Timestamp ->
        let data =
          Array.map data ~f:(fun ts ->
              Time_ns.to_int_ns_since_epoch ts |> Int64.of_int_exn)
        in
        A.array_timestamp_ns_from_with_zone data (Some !zone)
      | Time64 ->
        Array.map data ~f:(fun od ->
            Time_ns.Ofday.to_span_since_start_of_day od
            |> Time_ns.Span.to_int_ns
            |> Int64.of_int_exn)
        |> A.array_time64_ns_from
      | Duration ->
        Array.map data ~f:(fun sp -> Time_ns.Span.to_int_ns sp |> Int64.of_int_exn)
        |> A.array_duration_ns_from
      | String ->
        let sum_length = Array.sum (module Int) data ~f:String.length in
        if sum_length > 2_000_000_000
        then A.array_large_string_from data
        else A.array_string_from data
    in
    { data_type; data }

  let to_array _ = failwith "TODO"
end
