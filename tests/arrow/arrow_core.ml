(* More idiomatic wrapper around the Arrow api.
   - Use OCaml native types rather than the specialized versions.
   - Handle time/date types.
   *)
open! Core
module A = Arrow_gen.Arrow

let zone = ref "utc"

module Data_type = struct
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

module Column = struct
  type 'a t =
    { data : A.array_ref
    ; data_type : 'a Data_type.t
    }

  type packed = P : _ t -> packed

  let data t = t.data

  let of_data data =
    match A.array_data_type data with
    | Int32 | Int64 -> P { data; data_type = Int }
    | Float32 | Float64 -> P { data; data_type = Float }
    | Utf8 | LargeUtf8 -> P { data; data_type = String }
    | Date32 -> P { data; data_type = Date }
    | Timestamp _ -> P { data; data_type = Time }
    | Time32 _ | Time64 _ -> P { data; data_type = Ofday }
    | Duration _ -> P { data; data_type = Span }
    | data_type -> [%message "unsupported data type" (data_type : A.data_type)] |> raise_s

  let data_type t = t.data_type
  let len t = A.array_len t.data
  let null_count t = A.array_null_count t.data

  let of_array (type a) (data_type : a Data_type.t) (data : a array) =
    let data =
      match data_type with
      | Int -> Array.map data ~f:Int64.of_int_exn |> A.array_i64_from
      | Float -> A.array_f64_from data
      | Date ->
        Array.map data ~f:(fun d -> Date.(diff d unix_epoch) |> Int32.of_int_exn)
        |> A.array_date32_from
      | Time ->
        let data =
          Array.map data ~f:(fun ts ->
              Time_ns.to_int_ns_since_epoch ts |> Int64.of_int_exn)
        in
        A.array_timestamp_ns_from_with_zone data (Some !zone)
      | Ofday ->
        Array.map data ~f:(fun od ->
            Time_ns.Ofday.to_span_since_start_of_day od
            |> Time_ns.Span.to_int_ns
            |> Int64.of_int_exn)
        |> A.array_time64_ns_from
      | Span ->
        Array.map data ~f:(fun sp -> Time_ns.Span.to_int_ns sp |> Int64.of_int_exn)
        |> A.array_duration_ns_from
      | String ->
        let sum_length = Array.sum (module Int) data ~f:String.length in
        if sum_length > 2_000_000_000
        then A.array_large_string_from data
        else A.array_string_from data
    in
    { data_type; data }

  let time_unit_mult : A.time_unit -> int = function
    | Second -> 1_000_000_000
    | Millisecond -> 1_000_000
    | Microsecond -> 1_000
    | Nanosecond -> 1

  let to_array (type a) (t : a t) =
    let res : a array =
      match A.array_data_type t.data, t.data_type with
      | Int32, Int ->
        Option.value_exn (A.array_i32_values t.data) |> Array.map ~f:Int32.to_int_exn
      | Int64, Int ->
        Option.value_exn (A.array_i64_values t.data) |> Array.map ~f:Int64.to_int_exn
      | Float32, Float -> Option.value_exn (A.array_f32_values t.data)
      | Float64, Float -> Option.value_exn (A.array_f64_values t.data)
      | Utf8, String ->
        Option.value_exn (A.array_string_values t.data)
        |> Array.map ~f:(fun v -> Option.value_exn v)
      | LargeUtf8, String ->
        Option.value_exn (A.array_large_string_values t.data)
        |> Array.map ~f:(fun v -> Option.value_exn v)
      | Date32, Date ->
        Option.value_exn (A.array_date32_values t.data)
        |> Array.map ~f:(fun d -> Int32.to_int_exn d |> Date.(add_days unix_epoch))
      | Time64 time_unit, Ofday ->
        let time_unit_mult = time_unit_mult time_unit in
        Option.value_exn (A.array_time64_ns_values t.data)
        |> Array.map ~f:(fun d ->
               Int64.to_int_exn d * time_unit_mult
               |> Time_ns.Span.of_int_ns
               |> Time_ns.Ofday.of_span_since_start_of_day_exn)
      | Duration time_unit, Span ->
        let time_unit_mult = time_unit_mult time_unit in
        Option.value_exn (A.array_time64_ns_values t.data)
        |> Array.map ~f:(fun d ->
               Int64.to_int_exn d * time_unit_mult |> Time_ns.Span.of_int_ns)
      | data_type, _data_type ->
        [%message "unsupported data type" (data_type : A.data_type)] |> raise_s
    in
    res

  let sexp_of (type a) (t : a t) =
    match t.data_type with
    | Int -> to_array t |> [%sexp_of: int array]
    | Float -> to_array t |> [%sexp_of: float array]
    | Date -> to_array t |> [%sexp_of: Date.t array]
    | Time -> to_array t |> [%sexp_of: Time_ns_unix.t array]
    | Ofday -> to_array t |> [%sexp_of: Time_ns.Ofday.t array]
    | Span -> to_array t |> [%sexp_of: Time_ns.Span.t array]
    | String -> to_array t |> [%sexp_of: string array]

  let sexp_of_packed (P t) = sexp_of t
end
