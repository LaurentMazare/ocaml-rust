open! Base
open! Sexplib.Conv
open! Arrow_gen

let ok_exn = function
  | Ok ok -> ok
  | Error err -> Printf.failwithf "%s" err ()

let%expect_test _ =
  let array_len = 10 in
  let rb =
    let array_foo =
      Array.init array_len ~f:(fun i -> Float.of_int i |> Float.sqrt)
      |> Arrow.array_f64_from
    in
    let array_foo_ba =
      Array.init array_len ~f:(fun i -> 1. /. (1. +. Float.of_int i))
      |> Bigarray.Array1.of_array Float64 C_layout
      |> Arrow.array_f64_from_ba
    in
    let array_bar =
      Array.init array_len ~f:(Printf.sprintf "b<%d>") |> Arrow.array_string_from
    in
    Arrow.record_batch_create
      [| "foo", array_foo; "foo_ba", array_foo_ba; "bar", array_bar |]
    |> ok_exn
  in
  Stdio.printf
    "%s\n%!"
    (Arrow.record_batch_schema rb |> [%sexp_of: Arrow.schema] |> Sexp.to_string_hum);
  [%expect {|
    ((fields
      (((name foo) (data_type Float64) (nullable false))
       ((name foo_ba) (data_type Float64) (nullable false))
       ((name bar) (data_type Utf8) (nullable false))))
     (metadata ())) |}];
  Stdio.printf "%s\n%!" (Arrow.record_batch_debug rb);
  [%expect {|
    RecordBatch { schema: Schema { fields: [Field { name: "foo", data_type: Float64, nullable: false, dict_id: 0, dict_is_ordered: false, metadata: None }, Field { name: "foo_ba", data_type: Float64, nullable: false, dict_id: 0, dict_is_ordered: false, metadata: None }, Field { name: "bar", data_type: Utf8, nullable: false, dict_id: 0, dict_is_ordered: false, metadata: None }], metadata: {} }, columns: [PrimitiveArray<Float64>
    [
      0.0,
      1.0,
      1.4142135623730951,
      1.7320508075688772,
      2.0,
      2.23606797749979,
      2.449489742783178,
      2.6457513110645907,
      2.8284271247461903,
      3.0,
    ], PrimitiveArray<Float64>
    [
      1.0,
      0.5,
      0.3333333333333333,
      0.25,
      0.2,
      0.16666666666666666,
      0.14285714285714285,
      0.125,
      0.1111111111111111,
      0.1,
    ], StringArray
    [
      "b<0>",
      "b<1>",
      "b<2>",
      "b<3>",
      "b<4>",
      "b<5>",
      "b<6>",
      "b<7>",
      "b<8>",
      "b<9>",
    ]], row_count: 10 } |}]
