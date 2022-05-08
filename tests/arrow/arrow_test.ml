open! Base
open! Sexplib.Conv
open! Arrow_gen
module A = Arrow_core

let ok_exn = function
  | Ok ok -> ok
  | Error err -> Printf.failwithf "%s" err ()

let test_record_batch ~array_len =
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

let%expect_test _ =
  let rb = test_record_batch ~array_len:10 in
  Stdio.printf
    "%s\n%!"
    (Arrow.record_batch_schema rb |> [%sexp_of: Arrow.schema] |> Sexp.to_string_hum);
  [%expect
    {|
    ((fields
      (((name foo) (data_type Float64) (nullable false))
       ((name foo_ba) (data_type Float64) (nullable false))
       ((name bar) (data_type Utf8) (nullable false))))
     (metadata ())) |}];
  Stdio.printf "%s\n%!" (Arrow.record_batch_debug rb);
  [%expect
    {|
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
    ]], row_count: 10 } |}];
  let () =
    try
      let _column = Arrow.record_batch_column rb 123 in
      Stdio.printf "SHOULD HAVE FAILED!\n"
    with
    | Failure str -> Stdio.printf "failed as expected, %s\n%!" (String.prefix str 68)
  in
  [%expect
    {| failed as expected, panicked at 'index out of bounds: the len is 3 but the index is 123' |}];
  let column = (Arrow.record_batch_column rb 0 |> Arrow.array_i64_values) Int64.zero in
  Stdio.printf "%s\n%!" ([%sexp_of: Int64.t array option] column |> Sexp.to_string_mach);
  [%expect {| () |}];
  let column = (Arrow.record_batch_column rb 0 |> Arrow.array_f64_values) Float.nan in
  Stdio.printf "%s\n%!" ([%sexp_of: float array option] column |> Sexp.to_string_mach);
  let column = (Arrow.record_batch_column rb 1 |> Arrow.array_f64_values) Float.nan in
  Stdio.printf "%s\n%!" ([%sexp_of: float array option] column |> Sexp.to_string_mach);
  let column = Arrow.record_batch_column rb 2 |> Arrow.array_string_values_opt in
  Stdio.printf
    "%s\n%!"
    ([%sexp_of: string option array option] column |> Sexp.to_string_mach);
  [%expect
    {|
    ((0 1 1.4142135623730951 1.7320508075688772 2 2.23606797749979 2.4494897427831779 2.6457513110645907 2.8284271247461903 3))
    ((1 0.5 0.33333333333333331 0.25 0.2 0.16666666666666666 0.14285714285714285 0.125 0.1111111111111111 0.1))
    (((b<0>)(b<1>)(b<2>)(b<3>)(b<4>)(b<5>)(b<6>)(b<7>)(b<8>)(b<9>))) |}];
  [%expect {||}]

let%expect_test _ =
  let tmp_file =
    let tmp_file = Caml.Filename.temp_file "rb" ".parquet" in
    let rb = test_record_batch ~array_len:5 in
    Arrow.record_batch_write_parquet rb tmp_file |> ok_exn;
    tmp_file
  in
  let rb =
    let reader = Arrow.file_reader tmp_file |> ok_exn in
    let batch =
      Arrow.get_record_reader reader 4096 |> ok_exn |> Arrow.record_reader_next
    in
    Option.value_exn batch |> ok_exn
  in
  Stdio.printf
    "%s\n%!"
    (Arrow.record_batch_schema rb |> [%sexp_of: Arrow.schema] |> Sexp.to_string_hum);
  [%expect
    {|
    ((fields
      (((name foo) (data_type Float64) (nullable false))
       ((name foo_ba) (data_type Float64) (nullable false))
       ((name bar) (data_type Utf8) (nullable false))))
     (metadata ())) |}];
  let column = (Arrow.record_batch_column rb 0 |> Arrow.array_f64_values) Float.nan in
  Stdio.printf "%s\n%!" ([%sexp_of: float array option] column |> Sexp.to_string_mach);
  let column = (Arrow.record_batch_column rb 1 |> Arrow.array_f64_values) Float.nan in
  Stdio.printf "%s\n%!" ([%sexp_of: float array option] column |> Sexp.to_string_mach);
  let column = Arrow.record_batch_column rb 2 |> Arrow.array_string_values_opt in
  Stdio.printf
    "%s\n%!"
    ([%sexp_of: string option array option] column |> Sexp.to_string_mach);
  [%expect
    {|
    ((0 1 1.4142135623730951 1.7320508075688772 2))
    ((1 0.5 0.33333333333333331 0.25 0.2))
    (((b<0>)(b<1>)(b<2>)(b<3>)(b<4>))) |}]

let read_and_print path ~batch_size =
  A.Reader.with_reader path ~batch_size ~f:(fun reader ->
      let schema = A.Reader.schema reader |> ok_exn in
      Stdio.printf "%s\n%!" (A.Schema.sexp_of_t schema |> Sexp.to_string_hum);
      let rec loop batch_index =
        match A.Reader.next reader with
        | `Eof -> Stdio.printf "done\n%!"
        | `Batch rb ->
          let rb = ok_exn rb in
          Stdio.printf
            "  batch %d: %d rows, %d columns\n%!"
            batch_index
            (A.Record_batch.num_rows rb)
            (A.Record_batch.num_columns rb);
          A.Record_batch.columns rb
          |> List.iter ~f:(fun (name, packed) ->
                 Stdio.printf
                   "    %s: %s\n%!"
                   name
                   (A.Column.sexp_of_packed packed |> Sexp.to_string_mach));
          loop (batch_index + 1)
      in
      loop 1)
  |> ok_exn

let%expect_test _ =
  read_and_print "test.parquet" ~batch_size:4096;
  [%expect
    {|
    ((fields
      (((name x) (data_type Float64) (nullable true))
       ((name y) (data_type Utf8) (nullable true))
       ((name z) (data_type Float64) (nullable true))))
     (metadata
      ((pandas
        "{\"index_columns\": [], \"column_indexes\": [], \"columns\": [{\"name\": \"x\", \"field_name\": \"x\", \"pandas_type\": \"float64\", \"numpy_type\": \"float64\", \"metadata\": null}, {\"name\": \"y\", \"field_name\": \"y\", \"pandas_type\": \"unicode\", \"numpy_type\": \"object\", \"metadata\": null}, {\"name\": \"z\", \"field_name\": \"z\", \"pandas_type\": \"float64\", \"numpy_type\": \"float64\", \"metadata\": null}], \"creator\": {\"library\": \"pyarrow\", \"version\": \"4.0.1\"}, \"pandas_version\": \"1.0.5\"}"))))
      batch 1: 2 rows, 3 columns
        x: (3.1415 NAN)
        y: (foobar"")
        z: (NAN 12)
    done |}];
  let rb = A.Record_batch.read_parquet "test.parquet" |> ok_exn in
  let rb = A.Record_batch.concat [ rb; rb; rb; rb ] |> ok_exn in
  A.Record_batch.write_parquet rb "tmp-test.parquet" |> ok_exn;
  read_and_print "tmp-test.parquet" ~batch_size:3;
  Unix.unlink "tmp-test.parquet";
  [%expect
    {|
    ((fields
      (((name x) (data_type Float64) (nullable true))
       ((name y) (data_type Utf8) (nullable true))
       ((name z) (data_type Float64) (nullable true))))
     (metadata ()))
      batch 1: 3 rows, 3 columns
        x: (3.1415 NAN 3.1415)
        y: (foobar""foobar)
        z: (NAN 12 NAN)
      batch 2: 3 rows, 3 columns
        x: (NAN 3.1415 NAN)
        y: (""foobar"")
        z: (12 NAN 12)
      batch 3: 2 rows, 3 columns
        x: (3.1415 NAN)
        y: (foobar"")
        z: (NAN 12)
    done |}]
