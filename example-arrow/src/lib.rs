// TODO: Improve handling of null values.
use arrow::array::{ArrayRef as ArrowArrayRef, TimestampNanosecondArray};
use arrow::datatypes::DataType as DT;
use arrow::record_batch::RecordBatch as ArrowRecordBatch;
use ocaml_rust::{BigArray1, Custom, CustomConst, RustResult};
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use parquet::arrow::{ArrowReader, ArrowWriter, ParquetFileArrowReader};
use parquet::file::reader::SerializedFileReader;
use std::fs::File;
use std::sync::Arc;

impl Schema {
    fn of_arrow(schema: &arrow::datatypes::Schema) -> Schema {
        let fields: Vec<_> = schema
            .fields()
            .iter()
            .map(|field| SchemaField {
                name: field.name().to_string(),
                data_type: DataType::of_arrow(field.data_type()),
                nullable: field.is_nullable(),
            })
            .collect();
        let metadata: Vec<(String, String)> =
            schema.metadata().iter().map(|(x, y)| (x.to_string(), y.to_string())).collect();
        Schema { fields, metadata }
    }
}

// TODO: Add an explicit close function rather than rely on the GC
// collecting the file to trigger the close.
fn file_reader(path: String) -> RustResult<FileReader> {
    let file = File::open(&path)?;
    let file_reader = SerializedFileReader::new(file)?;
    let file_reader = Arc::new(file_reader);
    let file_reader = ParquetFileArrowReader::new(file_reader);
    Ok(Custom::new(file_reader))
}

fn metadata_as_string(reader: &FileReader) -> String {
    let mut reader = reader.inner().lock().unwrap();
    format!("{:?}", reader.get_metadata())
}

fn parquet_metadata(reader: &FileReader) -> Metadata {
    let mut reader = reader.inner().lock().unwrap();
    let metadata = reader.get_metadata();
    let f = metadata.file_metadata();
    let row_groups: Vec<_> = metadata
        .row_groups()
        .iter()
        .map(|r| RowGroupMetadata {
            num_columns: r.num_columns() as isize,
            num_rows: r.num_rows() as isize,
            total_byte_size: r.total_byte_size() as isize,
        })
        .collect();
    Metadata {
        num_rows: f.num_rows() as isize,
        version: f.version() as isize,
        created_by: f.created_by().clone(),
        row_groups,
    }
}

fn schema(reader: &FileReader) -> RustResult<Schema> {
    let mut reader = reader.inner().lock().unwrap();
    let schema = reader.get_schema()?;
    Ok(Schema::of_arrow(&schema))
}

fn get_record_reader(reader: &FileReader, batch_size: usize) -> RustResult<RecordReader> {
    let mut reader = reader.inner().lock().unwrap();
    Ok(Custom::new(reader.get_record_reader(batch_size)?))
}

fn get_record_reader_by_columns(
    reader: &FileReader,
    columns: Vec<usize>,
    batch_size: usize,
) -> RustResult<RecordReader> {
    let mut reader = reader.inner().lock().unwrap();
    Ok(Custom::new(reader.get_record_reader_by_columns(columns.into_iter(), batch_size)?))
}

fn record_reader_next(record_reader: &RecordReader) -> Option<RustResult<RecordBatch>> {
    let mut record_reader = record_reader.inner().lock().unwrap();
    record_reader.next().map(|x| x.map_err(|err| err.into()).map(CustomConst::new))
}

fn record_batch_create(columns: Vec<(String, ArrayRef)>) -> RustResult<RecordBatch> {
    let columns: Vec<_> =
        columns.into_iter().map(|(col_name, array)| (col_name, array.inner().clone())).collect();
    let record_batch = ArrowRecordBatch::try_from_iter(columns)?;
    Ok(CustomConst::new(record_batch))
}

fn record_batch_debug(record_batch: &RecordBatch) -> String {
    let record_batch = record_batch.inner();
    format!("{:?}", record_batch)
}

fn record_batch_schema(record_batch: &RecordBatch) -> Schema {
    let record_batch = record_batch.inner();
    Schema::of_arrow(record_batch.schema().as_ref())
}

fn record_batch_num_rows(record_batch: &RecordBatch) -> usize {
    let record_batch = record_batch.inner();
    record_batch.num_rows()
}

fn record_batch_num_columns(record_batch: &RecordBatch) -> usize {
    let record_batch = record_batch.inner();
    record_batch.num_columns()
}

fn record_batch_column(record_batch: &RecordBatch, index: usize) -> ArrayRef {
    let record_batch = record_batch.inner();
    CustomConst::new(record_batch.column(index).clone())
}

fn record_batch_write_parquet(record_batch: &RecordBatch, path: String) -> RustResult<()> {
    let record_batch = record_batch.inner();
    let file = File::create(&path)?;
    let props = parquet::file::properties::WriterProperties::builder()
        .set_writer_version(parquet::file::properties::WriterVersion::PARQUET_2_0)
        .set_compression(parquet::basic::Compression::SNAPPY)
        .build();

    let mut writer = ArrowWriter::try_new(file, record_batch.schema(), Some(props))?;

    writer.write(record_batch)?;

    // writer must be closed to write footer
    writer.close()?;
    Ok(())
}
fn record_batch_slice(record_batch: &RecordBatch, offset: isize, length: isize) -> RecordBatch {
    let record_batch = record_batch.inner();
    let record_batch = record_batch.slice(offset as usize, length as usize);
    CustomConst::new(record_batch)
}

fn record_batch_concat(batches: Vec<RecordBatch>) -> RustResult<RecordBatch> {
    if batches.is_empty() {
        return Err("empty batch list in record_batch_concat".into());
    }
    let schema = batches[0].inner().schema();
    if let Some((i, _)) =
        batches.iter().enumerate().find(|&(_, batch)| batch.inner().schema() != schema)
    {
        return Err(arrow::error::ArrowError::InvalidArgumentError(format!(
            "batches[{}] schema is different with argument schema.",
            i
        ))
        .into());
    }
    let field_num = schema.fields().len();
    let mut arrays = Vec::with_capacity(field_num);
    for i in 0..field_num {
        let array = arrow::compute::concat(
            &batches.iter().map(|batch| batch.inner().column(i).as_ref()).collect::<Vec<_>>(),
        )?;
        arrays.push(array);
    }
    let rb = ArrowRecordBatch::try_new(schema.clone(), arrays)?;
    Ok(CustomConst::new(rb))
}

fn writer_new(record_batch: &RecordBatch, path: String) -> RustResult<FileWriter> {
    let record_batch = record_batch.inner();
    let file = File::create(&path)?;
    let props = parquet::file::properties::WriterProperties::builder()
        .set_writer_version(parquet::file::properties::WriterVersion::PARQUET_2_0)
        .set_compression(parquet::basic::Compression::SNAPPY)
        .build();

    let mut writer = ArrowWriter::try_new(file, record_batch.schema(), Some(props))?;
    writer.write(record_batch)?;
    Ok(Custom::new(writer))
}

fn writer_write(w: &FileWriter, record_batch: &RecordBatch) -> RustResult<()> {
    let mut w = w.inner().lock().unwrap();
    let record_batch = record_batch.inner();
    w.write(record_batch)?;
    Ok(())
}

fn writer_close(w: &FileWriter) -> RustResult<()> {
    let mut w = w.inner().lock().unwrap();
    let _metadata = w.close()?;
    Ok(())
}

fn array_data_type(array: &ArrayRef) -> DataType {
    let array = array.inner();
    DataType::of_arrow(array.data_type())
}

fn array_len(array: &ArrayRef) -> usize {
    let array = array.inner();
    array.len()
}

fn array_null_count(array: &ArrayRef) -> usize {
    let array = array.inner();
    array.null_count()
}

macro_rules! value_fns {
    ($from_fn: ident, $from_fn_ba: ident, $value_fn: ident, $value_fn_ba: ident, $typ: ident, $array_typ: ident) => {
        fn $from_fn(array: Vec<$typ>) -> ArrayRef {
            let array = arrow::array::$array_typ::from_iter_values(array.into_iter());
            CustomConst::new(Arc::new(array))
        }

        fn $from_fn_ba(array: BigArray1<$typ>) -> ArrayRef {
            let array = arrow::array::$array_typ::from_iter_values(array.data().iter().map(|&x| x));
            CustomConst::new(Arc::new(array))
        }

        fn $value_fn(array: &ArrayRef) -> Option<Vec<$typ>> {
            let array = array.inner();
            array.as_any().downcast_ref::<arrow::array::$array_typ>().map(|x| x.values().to_vec())
        }

        fn $value_fn_ba(array: &ArrayRef) -> Option<BigArray1<$typ>> {
            let array = array.inner();
            array
                .as_any()
                .downcast_ref::<arrow::array::$array_typ>()
                .map(|x| BigArray1::new(x.values()))
        }
    };
}

value_fns!(
    array_duration_ns_from,
    array_duration_ns_from_ba,
    array_duration_ns_values,
    array_duration_ns_values_ba,
    i64,
    DurationNanosecondArray
);
value_fns!(
    array_time_ns_from,
    array_time_ns_from_ba,
    array_time_ns_values,
    array_time_ns_values_ba,
    i64,
    Time64NanosecondArray
);
value_fns!(
    array_timestamp_ns_from,
    array_timestamp_ns_from_ba,
    array_timestamp_ns_values,
    array_timestamp_ns_values_ba,
    i64,
    TimestampNanosecondArray
);
value_fns!(
    array_date32_from,
    array_date32_from_ba,
    array_date32_values,
    array_date32_values_ba,
    i32,
    Date32Array
);
value_fns!(
    array_date64_from,
    array_date64_from_ba,
    array_date64_values,
    array_date64_values_ba,
    i64,
    Date64Array
);
value_fns!(
    array_char_from,
    array_char_from_ba,
    array_char_values,
    array_char_values_ba,
    u8,
    UInt8Array
);
value_fns!(
    array_i32_from,
    array_i32_from_ba,
    array_i32_values,
    array_i32_values_ba,
    i32,
    Int32Array
);
value_fns!(
    array_i64_from,
    array_i64_from_ba,
    array_i64_values,
    array_i64_values_ba,
    i64,
    Int64Array
);
value_fns!(
    array_f32_from,
    array_f32_from_ba,
    array_f32_values,
    array_f32_values_ba,
    f32,
    Float32Array
);
value_fns!(
    array_f64_from,
    array_f64_from_ba,
    array_f64_values,
    array_f64_values_ba,
    f64,
    Float64Array
);

fn array_timestamp_ns_from_with_zone(vec: Vec<i64>, zone: Option<String>) -> ArrayRef {
    let array: TimestampNanosecondArray = arrow::array::PrimitiveArray::from_vec(vec, zone);
    CustomConst::new(Arc::new(array))
}

fn array_string_from(vec: Vec<String>) -> ArrayRef {
    let array = arrow::array::StringArray::from_iter_values(vec.into_iter());
    CustomConst::new(Arc::new(array))
}

fn array_large_string_from(vec: Vec<String>) -> ArrayRef {
    let array = arrow::array::LargeStringArray::from_iter_values(vec.into_iter());
    CustomConst::new(Arc::new(array))
}

fn array_string_values(array: &ArrayRef) -> Option<Vec<Option<String>>> {
    let array = array.inner();
    array
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .map(|array| array.iter().map(|s| s.map(|s| s.to_string())).collect())
}

fn array_large_string_values(array: &ArrayRef) -> Option<Vec<Option<String>>> {
    let array = array.inner();
    array
        .as_any()
        .downcast_ref::<arrow::array::LargeStringArray>()
        .map(|array| array.iter().map(|s| s.map(|s| s.to_string())).collect())
}

impl IntervalUnit {
    fn of_arrow(unit: &arrow::datatypes::IntervalUnit) -> Self {
        match unit {
            arrow::datatypes::IntervalUnit::YearMonth => Self::YearMonth,
            arrow::datatypes::IntervalUnit::DayTime => Self::DayTime,
            arrow::datatypes::IntervalUnit::MonthDayNano => Self::MonthDayNano,
        }
    }
}

impl TimeUnit {
    fn of_arrow(unit: &arrow::datatypes::TimeUnit) -> Self {
        match unit {
            arrow::datatypes::TimeUnit::Second => Self::Second,
            arrow::datatypes::TimeUnit::Millisecond => Self::Millisecond,
            arrow::datatypes::TimeUnit::Microsecond => Self::Microsecond,
            arrow::datatypes::TimeUnit::Nanosecond => Self::Nanosecond,
        }
    }
}

impl DataType {
    fn of_arrow(data_type: &DT) -> Self {
        match data_type {
            DT::Null => Self::Null,
            DT::Boolean => Self::Boolean,
            DT::Int8 => Self::Int8,
            DT::Int16 => Self::Int16,
            DT::Int32 => Self::Int32,
            DT::Int64 => Self::Int64,
            DT::UInt8 => Self::UInt8,
            DT::UInt16 => Self::UInt16,
            DT::UInt32 => Self::UInt32,
            DT::UInt64 => Self::UInt64,
            DT::Float16 => Self::Float16,
            DT::Float32 => Self::Float32,
            DT::Float64 => Self::Float64,
            DT::Timestamp(unit, zone) => Self::Timestamp(TimeUnit::of_arrow(unit), zone.clone()),
            DT::Date32 => Self::Date32,
            DT::Date64 => Self::Date64,
            DT::Time32(unit) => Self::Time32(TimeUnit::of_arrow(unit)),
            DT::Time64(unit) => Self::Time64(TimeUnit::of_arrow(unit)),
            DT::Duration(unit) => Self::Duration(TimeUnit::of_arrow(unit)),
            DT::Interval(unit) => Self::Interval(IntervalUnit::of_arrow(unit)),
            DT::Binary => Self::Binary,
            DT::FixedSizeBinary(size) => Self::FixedSizeBinary(*size as isize),
            DT::LargeBinary => Self::LargeBinary,
            DT::Utf8 => Self::Utf8,
            DT::LargeUtf8 => Self::LargeUtf8,
            DT::List(_) => Self::List,
            DT::FixedSizeList(_, _) => Self::FixedSizeList,
            DT::LargeList(_) => Self::LargeList,
            DT::Struct(_) => Self::Struct,
            DT::Union(_, _) => Self::Union,
            DT::Dictionary(d1, d2) => {
                let d1 = Box::new(Self::of_arrow(d1));
                let d2 = Box::new(Self::of_arrow(d2));
                Self::Dictionary(d1, d2)
            }
            DT::Decimal(s1, s2) => Self::Decimal(*s1, *s2),
            DT::Map(_, _) => Self::Map,
        }
    }
}

#[ocaml_rust::bridge]
mod arrow {
    ocaml_include!("open! Sexplib.Conv");
    type FileReader = Custom<ParquetFileArrowReader>;
    type FileWriter = Custom<ArrowWriter<std::fs::File>>;
    type RecordReader = Custom<ParquetRecordBatchReader>;
    type RecordBatch = CustomConst<ArrowRecordBatch>;
    type ArrayRef = CustomConst<ArrowArrayRef>;

    #[ocaml_deriving(sexp)]
    #[derive(Debug, Clone)]
    enum IntervalUnit {
        YearMonth,
        DayTime,
        MonthDayNano,
    }

    #[ocaml_deriving(sexp)]
    #[derive(Debug, Clone)]
    enum TimeUnit {
        Second,
        Millisecond,
        Microsecond,
        Nanosecond,
    }

    #[ocaml_deriving(sexp)]
    #[derive(Debug, Clone)]
    enum DataType {
        Null,
        Boolean,
        Int8,
        Int16,
        Int32,
        Int64,
        UInt8,
        UInt16,
        UInt32,
        UInt64,
        Float16,
        Float32,
        Float64,
        Timestamp(TimeUnit, Option<String>),
        Date32,
        Date64,
        Time32(TimeUnit),
        Time64(TimeUnit),
        Duration(TimeUnit),
        Interval(IntervalUnit),
        Binary,
        FixedSizeBinary(isize),
        LargeBinary,
        Utf8,
        LargeUtf8,
        List,
        FixedSizeList,
        LargeList,
        Struct,
        Union,
        Dictionary(Box<DataType>, Box<DataType>),
        Decimal(usize, usize),
        Map,
    }

    #[ocaml_deriving(sexp)]
    #[derive(Debug, Clone)]
    struct RowGroupMetadata {
        num_columns: isize,
        num_rows: isize,
        total_byte_size: isize,
    }

    #[ocaml_deriving(sexp)]
    #[derive(Debug, Clone)]
    struct Metadata {
        num_rows: isize,
        version: isize,
        created_by: Option<String>,
        row_groups: Vec<RowGroupMetadata>,
    }

    #[ocaml_deriving(sexp)]
    #[derive(Debug, Clone)]
    struct SchemaField {
        name: String,
        data_type: DataType,
        nullable: bool,
    }

    #[ocaml_deriving(sexp)]
    #[derive(Debug, Clone)]
    struct Schema {
        fields: Vec<SchemaField>,
        metadata: Vec<(String, String)>,
    }

    extern "Rust" {
        // TODO: provide better scoping/module.
        fn file_reader(path: String) -> RustResult<FileReader>;
        fn metadata_as_string(reader: &FileReader) -> String;
        fn parquet_metadata(reader: &FileReader) -> Metadata;
        fn schema(reader: &FileReader) -> RustResult<Schema>;
        fn get_record_reader(reader: &FileReader, batch_size: usize) -> RustResult<RecordReader>;
        fn get_record_reader_by_columns(
            reader: &FileReader,
            columns: Vec<usize>,
            batch_size: usize,
        ) -> RustResult<RecordReader>;

        fn record_reader_next(record_reader: &RecordReader) -> Option<RustResult<RecordBatch>>;

        fn record_batch_create(columns: Vec<(String, ArrayRef)>) -> RustResult<RecordBatch>;
        fn record_batch_debug(record_batch: &RecordBatch) -> String;
        fn record_batch_schema(record_batch: &RecordBatch) -> Schema;
        fn record_batch_num_rows(record_batch: &RecordBatch) -> usize;
        fn record_batch_num_columns(record_batch: &RecordBatch) -> usize;
        fn record_batch_column(record_batch: &RecordBatch, index: usize) -> ArrayRef;
        fn record_batch_write_parquet(record_batch: &RecordBatch, path: String) -> RustResult<()>;
        fn record_batch_slice(
            record_batch: &RecordBatch,
            offset: isize,
            length: isize,
        ) -> RecordBatch;
        fn record_batch_concat(batches: Vec<RecordBatch>) -> RustResult<RecordBatch>;

        fn writer_new(record_batch: &RecordBatch, path: String) -> RustResult<FileWriter>;
        fn writer_write(w: &FileWriter, record_batch: &RecordBatch) -> RustResult<()>;
        fn writer_close(file_writer: &FileWriter) -> RustResult<()>;

        fn array_data_type(array: &ArrayRef) -> DataType;
        fn array_len(array: &ArrayRef) -> usize;
        fn array_null_count(array: &ArrayRef) -> usize;

        fn array_duration_ns_from_ba(v: BigArray1<i64>) -> ArrayRef;
        fn array_time_ns_from_ba(v: BigArray1<i64>) -> ArrayRef;
        fn array_timestamp_ns_from_ba(v: BigArray1<i64>) -> ArrayRef;
        fn array_date32_from_ba(v: BigArray1<i32>) -> ArrayRef;
        fn array_date64_from_ba(v: BigArray1<i64>) -> ArrayRef;
        fn array_char_from_ba(v: BigArray1<u8>) -> ArrayRef;
        fn array_i32_from_ba(v: BigArray1<i32>) -> ArrayRef;
        fn array_i64_from_ba(v: BigArray1<i64>) -> ArrayRef;
        fn array_f32_from_ba(v: BigArray1<f32>) -> ArrayRef;
        fn array_f64_from_ba(v: BigArray1<f64>) -> ArrayRef;

        fn array_duration_ns_from(v: Vec<i64>) -> ArrayRef;
        fn array_time_ns_from(v: Vec<i64>) -> ArrayRef;
        fn array_timestamp_ns_from(v: Vec<i64>) -> ArrayRef;
        fn array_date32_from(v: Vec<i32>) -> ArrayRef;
        fn array_date64_from(v: Vec<i64>) -> ArrayRef;
        fn array_char_from(v: Vec<u8>) -> ArrayRef;
        fn array_i32_from(v: Vec<i32>) -> ArrayRef;
        fn array_i64_from(v: Vec<i64>) -> ArrayRef;
        fn array_f32_from(v: Vec<f32>) -> ArrayRef;
        fn array_f64_from(v: Vec<f64>) -> ArrayRef;

        fn array_duration_ns_values(array: &ArrayRef) -> Option<Vec<i64>>;
        fn array_time_ns_values(array: &ArrayRef) -> Option<Vec<i64>>;
        fn array_timestamp_ns_values(array: &ArrayRef) -> Option<Vec<i64>>;
        fn array_date32_values(array: &ArrayRef) -> Option<Vec<i32>>;
        fn array_date64_values(array: &ArrayRef) -> Option<Vec<i64>>;
        fn array_char_values(array: &ArrayRef) -> Option<Vec<u8>>;
        fn array_i32_values(array: &ArrayRef) -> Option<Vec<i32>>;
        fn array_i64_values(array: &ArrayRef) -> Option<Vec<i64>>;
        fn array_f32_values(array: &ArrayRef) -> Option<Vec<f32>>;
        fn array_f64_values(array: &ArrayRef) -> Option<Vec<f64>>;

        fn array_duration_ns_values_ba(array: &ArrayRef) -> Option<BigArray1<i64>>;
        fn array_time_ns_values_ba(array: &ArrayRef) -> Option<BigArray1<i64>>;
        fn array_timestamp_ns_values_ba(array: &ArrayRef) -> Option<BigArray1<i64>>;
        fn array_date32_values_ba(array: &ArrayRef) -> Option<BigArray1<i32>>;
        fn array_date64_values_ba(array: &ArrayRef) -> Option<BigArray1<i64>>;
        fn array_char_values_ba(array: &ArrayRef) -> Option<BigArray1<u8>>;
        fn array_i32_values_ba(array: &ArrayRef) -> Option<BigArray1<i32>>;
        fn array_i64_values_ba(array: &ArrayRef) -> Option<BigArray1<i64>>;
        fn array_f32_values_ba(array: &ArrayRef) -> Option<BigArray1<f32>>;
        fn array_f64_values_ba(array: &ArrayRef) -> Option<BigArray1<f64>>;

        fn array_timestamp_ns_from_with_zone(v: Vec<i64>, zone: Option<String>) -> ArrayRef;

        fn array_string_from(v: Vec<String>) -> ArrayRef;
        fn array_large_string_from(v: Vec<String>) -> ArrayRef;
        fn array_string_values(array: &ArrayRef) -> Option<Vec<Option<String>>>;
        fn array_large_string_values(array: &ArrayRef) -> Option<Vec<Option<String>>>;
    }
}
