use arrow::array::{Array, ArrayRef as ArrowArrayRef, TimestampNanosecondArray};
use arrow::csv::reader::Reader as ArrowCsvReader;
use arrow::csv::writer::Writer as ArrowCsvWriter;
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

fn file_reader(path: String) -> RustResult<FileReader> {
    let file = File::open(&path)?;
    let file_reader = SerializedFileReader::new(file)?;
    let file_reader = Arc::new(file_reader);
    let file_reader = ParquetFileArrowReader::new(file_reader);
    Ok(Custom::new(Some(file_reader)))
}

fn file_reader_close(reader: &FileReader) {
    let mut reader = reader.inner().lock().unwrap();
    *reader = None
}

fn file_reader_metadata_as_string(reader: &FileReader) -> RustResult<String> {
    let mut reader = reader.inner().lock().unwrap();
    let reader = reader.as_mut().map_or_else(|| Err("already closed"), Ok)?;
    Ok(format!("{:?}", reader.get_metadata()))
}

fn file_reader_parquet_metadata(reader: &FileReader) -> RustResult<Metadata> {
    let mut reader = reader.inner().lock().unwrap();
    let reader = reader.as_mut().map_or_else(|| Err("already closed"), Ok)?;
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
    let metadata = Metadata {
        num_rows: f.num_rows() as isize,
        version: f.version() as isize,
        created_by: f.created_by().map(|x| x.to_string()),
        row_groups,
    };
    Ok(metadata)
}

fn file_reader_schema(reader: &FileReader) -> RustResult<Schema> {
    let mut reader = reader.inner().lock().unwrap();
    let reader = reader.as_mut().map_or_else(|| Err("already closed"), Ok)?;
    let schema = reader.get_schema()?;
    Ok(Schema::of_arrow(&schema))
}

fn get_record_reader(reader: &FileReader, batch_size: usize) -> RustResult<RecordReader> {
    let mut reader = reader.inner().lock().unwrap();
    let reader = reader.as_mut().map_or_else(|| Err("already closed"), Ok)?;
    Ok(Custom::new(Some(reader.get_record_reader(batch_size)?)))
}

fn get_record_reader_by_columns(
    reader: &FileReader,
    columns: Vec<usize>,
    batch_size: usize,
) -> RustResult<RecordReader> {
    let mut reader = reader.inner().lock().unwrap();
    let reader = reader.as_mut().map_or_else(|| Err("already closed"), Ok)?;
    let metadata = reader.get_metadata();
    let f = metadata.file_metadata();
    let schema_descr = f.schema_descr();
    let mask = parquet::arrow::ProjectionMask::leaves(&schema_descr, columns.into_iter());
    let reader = reader.get_record_reader_by_columns(mask, batch_size)?;
    Ok(Custom::new(Some(reader)))
}

fn record_reader_next(record_reader: &RecordReader) -> Option<RustResult<RecordBatch>> {
    let mut record_reader = record_reader.inner().lock().unwrap();
    match record_reader.as_mut() {
        None => None,
        Some(record_reader) => {
            record_reader.next().map(|x| x.map_err(|err| err.into()).map(CustomConst::new))
        }
    }
}

fn record_reader_close(record_reader: &RecordReader) {
    let mut record_reader = record_reader.inner().lock().unwrap();
    *record_reader = None
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
    let rb = ArrowRecordBatch::try_new(schema, arrays)?;
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
    Ok(Custom::new(Some(writer)))
}

fn writer_write(w: &FileWriter, record_batch: &RecordBatch) -> RustResult<()> {
    let mut w = w.inner().lock().unwrap();
    let w = w.as_mut().map_or_else(|| Err("already closed"), Ok)?;
    let record_batch = record_batch.inner();
    w.write(record_batch)?;
    Ok(())
}

fn writer_close(w: &FileWriter) -> RustResult<()> {
    let mut w = w.inner().lock().unwrap();
    if let Some(w) = std::mem::replace(&mut *w, None) {
        let _metadata = w.close()?;
    }
    Ok(())
}

fn csv_writer_new(path: String) -> RustResult<CsvFileWriter> {
    let file = File::create(&path)?;
    let writer = ArrowCsvWriter::new(file);
    Ok(Custom::new(Some(writer)))
}

fn csv_writer_write(w: &CsvFileWriter, record_batch: &RecordBatch) -> RustResult<()> {
    let mut w = w.inner().lock().unwrap();
    let w = w.as_mut().map_or_else(|| Err("already closed"), Ok)?;
    let record_batch = record_batch.inner();
    w.write(record_batch)?;
    Ok(())
}

fn csv_writer_close(w: &CsvFileWriter) {
    let mut w = w.inner().lock().unwrap();
    *w = None
}

fn csv_reader_new(
    path: String,
    batch_size: usize,
    infer_size: Option<usize>,
) -> RustResult<CsvFileReader> {
    let file = File::create(&path)?;
    let builder =
        arrow::csv::ReaderBuilder::new().infer_schema(infer_size).with_batch_size(batch_size);
    let reader = builder.build(file)?;
    Ok(Custom::new(Some(reader)))
}

fn csv_reader_next(r: &CsvFileReader) -> Option<RustResult<RecordBatch>> {
    let mut r = r.inner().lock().unwrap();
    match r.as_mut() {
        None => None,
        Some(r) => r.next().map(|x| x.map_err(|err| err.into()).map(CustomConst::new)),
    }
}

fn csv_reader_close(r: &CsvFileReader) {
    let mut r = r.inner().lock().unwrap();
    *r = None
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
    ($mod_ident: ident, $typ: ident, $array_typ: ident) => {
        mod $mod_ident {
            use super::*;
            #[allow(dead_code)]
            pub(crate) fn from(array: Vec<$typ>) -> ArrayRef {
                let array = arrow::array::$array_typ::from_iter_values(array.into_iter());
                CustomConst::new(Arc::new(array))
            }

            #[allow(dead_code)]
            pub(crate) fn from_ba(array: BigArray1<$typ>) -> ArrayRef {
                let array =
                    arrow::array::$array_typ::from_iter_values(array.data().iter().map(|&x| x));
                CustomConst::new(Arc::new(array))
            }

            #[allow(dead_code)]
            pub(crate) fn values(array: &ArrayRef, default: $typ) -> Option<Vec<$typ>> {
                let array = array.inner();
                array.as_any().downcast_ref::<arrow::array::$array_typ>().map(|x| {
                    if x.null_count() > 0 {
                        x.values()
                            .iter()
                            .enumerate()
                            .map(|(i, v)| if x.is_null(i) { default } else { *v })
                            .collect::<Vec<_>>()
                    } else {
                        x.values().to_vec()
                    }
                })
            }

            #[allow(dead_code)]
            pub(crate) fn values_opt(array: &ArrayRef) -> Option<Vec<Option<$typ>>> {
                let array = array.inner();
                array.as_any().downcast_ref::<arrow::array::$array_typ>().map(|x| {
                    if x.null_count() > 0 {
                        x.values()
                            .iter()
                            .enumerate()
                            .map(|(i, v)| if x.is_null(i) { None } else { Some(*v) })
                            .collect::<Vec<_>>()
                    } else {
                        x.values().iter().map(|v| Some(*v)).collect()
                    }
                })
            }

            #[allow(dead_code)]
            pub(crate) fn values_ba(array: &ArrayRef, default: $typ) -> Option<BigArray1<$typ>> {
                let array = array.inner();
                array.as_any().downcast_ref::<arrow::array::$array_typ>().map(|x| {
                    let mut ba = BigArray1::new(x.values());
                    if x.null_count() > 1 {
                        let data = ba.data_mut();
                        for (i, v) in data.iter_mut().enumerate() {
                            if x.is_null(i) {
                                *v = default
                            }
                        }
                    }
                    ba
                })
            }
        }
    };
}

value_fns!(array_duration_ns, i64, DurationNanosecondArray);
value_fns!(array_duration_us, i64, DurationMicrosecondArray);
value_fns!(array_duration_ms, i64, DurationMillisecondArray);
value_fns!(array_duration_s, i64, DurationSecondArray);
value_fns!(array_time64_ns, i64, Time64NanosecondArray);
value_fns!(array_time64_us, i64, Time64MicrosecondArray);
value_fns!(array_timestamp_ns, i64, TimestampNanosecondArray);
value_fns!(array_timestamp_us, i64, TimestampMicrosecondArray);
value_fns!(array_timestamp_ms, i64, TimestampMillisecondArray);
value_fns!(array_timestamp_s, i64, TimestampSecondArray);
value_fns!(array_date32, i32, Date32Array);
value_fns!(array_date64, i64, Date64Array);
value_fns!(array_char, u8, UInt8Array);
value_fns!(array_i32, i32, Int32Array);
value_fns!(array_i64, i64, Int64Array);
value_fns!(array_f32, f32, Float32Array);
value_fns!(array_f64, f64, Float64Array);

fn array_null(size: usize) -> ArrayRef {
    let array = arrow::array::NullArray::new(size);
    CustomConst::new(Arc::new(array))
}

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

fn array_string_values(array: &ArrayRef, default: String) -> Option<Vec<String>> {
    let array = array.inner();
    array.as_any().downcast_ref::<arrow::array::StringArray>().map(|array| {
        array.iter().map(|s| s.map_or_else(|| default.to_string(), |s| s.to_string())).collect()
    })
}

fn array_large_string_values(array: &ArrayRef, default: String) -> Option<Vec<String>> {
    let array = array.inner();
    array.as_any().downcast_ref::<arrow::array::LargeStringArray>().map(|array| {
        array.iter().map(|s| s.map_or_else(|| default.to_string(), |s| s.to_string())).collect()
    })
}

fn array_string_values_opt(array: &ArrayRef) -> Option<Vec<Option<String>>> {
    let array = array.inner();
    array
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .map(|array| array.iter().map(|s| s.map(|s| s.to_string())).collect())
}

fn array_large_string_values_opt(array: &ArrayRef) -> Option<Vec<Option<String>>> {
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
            DT::Union(..) => Self::Union,
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
    type FileReader = Custom<Option<ParquetFileArrowReader>>;
    type FileWriter = Custom<Option<ArrowWriter<std::fs::File>>>;
    type CsvFileReader = Custom<Option<ArrowCsvReader<std::fs::File>>>;
    type CsvFileWriter = Custom<Option<ArrowCsvWriter<std::fs::File>>>;
    type RecordReader = Custom<Option<ParquetRecordBatchReader>>;
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
        #[release_runtime_lock]
        fn file_reader(path: String) -> RustResult<FileReader>;
        #[release_runtime_lock]
        fn file_reader_metadata_as_string(reader: &FileReader) -> RustResult<String>;
        #[release_runtime_lock]
        fn file_reader_parquet_metadata(reader: &FileReader) -> RustResult<Metadata>;
        #[release_runtime_lock]
        fn file_reader_schema(reader: &FileReader) -> RustResult<Schema>;
        #[release_runtime_lock]
        fn file_reader_close(reader: &FileReader);
        #[release_runtime_lock]
        fn get_record_reader(reader: &FileReader, batch_size: usize) -> RustResult<RecordReader>;
        #[release_runtime_lock]
        fn get_record_reader_by_columns(
            reader: &FileReader,
            columns: Vec<usize>,
            batch_size: usize,
        ) -> RustResult<RecordReader>;

        #[release_runtime_lock]
        fn record_reader_next(record_reader: &RecordReader) -> Option<RustResult<RecordBatch>>;
        #[release_runtime_lock]
        fn record_reader_close(record_reader: &RecordReader);

        fn record_batch_create(columns: Vec<(String, ArrayRef)>) -> RustResult<RecordBatch>;
        fn record_batch_debug(record_batch: &RecordBatch) -> String;
        fn record_batch_schema(record_batch: &RecordBatch) -> Schema;
        fn record_batch_num_rows(record_batch: &RecordBatch) -> usize;
        fn record_batch_num_columns(record_batch: &RecordBatch) -> usize;
        fn record_batch_column(record_batch: &RecordBatch, index: usize) -> ArrayRef;

        #[release_runtime_lock]
        fn record_batch_write_parquet(record_batch: &RecordBatch, path: String) -> RustResult<()>;

        fn record_batch_slice(
            record_batch: &RecordBatch,
            offset: isize,
            length: isize,
        ) -> RecordBatch;
        fn record_batch_concat(batches: Vec<RecordBatch>) -> RustResult<RecordBatch>;

        #[release_runtime_lock]
        fn writer_new(record_batch: &RecordBatch, path: String) -> RustResult<FileWriter>;
        #[release_runtime_lock]
        fn writer_write(w: &FileWriter, record_batch: &RecordBatch) -> RustResult<()>;
        #[release_runtime_lock]
        fn writer_close(file_writer: &FileWriter) -> RustResult<()>;

        #[release_runtime_lock]
        fn csv_writer_new(path: String) -> RustResult<CsvFileWriter>;
        #[release_runtime_lock]
        fn csv_writer_write(w: &CsvFileWriter, record_batch: &RecordBatch) -> RustResult<()>;
        #[release_runtime_lock]
        fn csv_writer_close(file_writer: &CsvFileWriter);

        #[release_runtime_lock]
        fn csv_reader_new(
            path: String,
            batch_size: usize,
            infer_size: Option<usize>,
        ) -> RustResult<CsvFileReader>;
        #[release_runtime_lock]
        fn csv_reader_next(r: &CsvFileReader) -> Option<RustResult<RecordBatch>>;
        #[release_runtime_lock]
        fn csv_reader_close(r: &CsvFileReader);

        fn array_data_type(array: &ArrayRef) -> DataType;
        fn array_len(array: &ArrayRef) -> usize;
        fn array_null_count(array: &ArrayRef) -> usize;

        #[namespace = "array_duration_ns"]
        fn from_ba(v: BigArray1<i64>) -> ArrayRef;
        #[namespace = "array_duration_ns"]
        fn from(v: Vec<i64>) -> ArrayRef;
        #[namespace = "array_duration_ns"]
        fn values(array: &ArrayRef, default: i64) -> Option<Vec<i64>>;
        #[namespace = "array_duration_ns"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<i64>>>;
        #[namespace = "array_duration_ns"]
        fn values_ba(array: &ArrayRef, default: i64) -> Option<BigArray1<i64>>;

        #[namespace = "array_duration_us"]
        fn from_ba(v: BigArray1<i64>) -> ArrayRef;
        #[namespace = "array_duration_us"]
        fn from(v: Vec<i64>) -> ArrayRef;
        #[namespace = "array_duration_us"]
        fn values(array: &ArrayRef, default: i64) -> Option<Vec<i64>>;
        #[namespace = "array_duration_us"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<i64>>>;
        #[namespace = "array_duration_us"]
        fn values_ba(array: &ArrayRef, default: i64) -> Option<BigArray1<i64>>;

        #[namespace = "array_duration_ms"]
        fn from_ba(v: BigArray1<i64>) -> ArrayRef;
        #[namespace = "array_duration_ms"]
        fn from(v: Vec<i64>) -> ArrayRef;
        #[namespace = "array_duration_ms"]
        fn values(array: &ArrayRef, default: i64) -> Option<Vec<i64>>;
        #[namespace = "array_duration_ms"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<i64>>>;
        #[namespace = "array_duration_ms"]
        fn values_ba(array: &ArrayRef, default: i64) -> Option<BigArray1<i64>>;

        #[namespace = "array_duration_s"]
        fn from_ba(v: BigArray1<i64>) -> ArrayRef;
        #[namespace = "array_duration_s"]
        fn from(v: Vec<i64>) -> ArrayRef;
        #[namespace = "array_duration_s"]
        fn values(array: &ArrayRef, default: i64) -> Option<Vec<i64>>;
        #[namespace = "array_duration_s"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<i64>>>;
        #[namespace = "array_duration_s"]
        fn values_ba(array: &ArrayRef, default: i64) -> Option<BigArray1<i64>>;

        #[namespace = "array_time64_ns"]
        fn from_ba(v: BigArray1<i64>) -> ArrayRef;
        #[namespace = "array_time64_ns"]
        fn from(v: Vec<i64>) -> ArrayRef;
        #[namespace = "array_time64_ns"]
        fn values(array: &ArrayRef, default: i64) -> Option<Vec<i64>>;
        #[namespace = "array_time64_ns"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<i64>>>;
        #[namespace = "array_time64_ns"]
        fn values_ba(array: &ArrayRef, default: i64) -> Option<BigArray1<i64>>;

        #[namespace = "array_time64_us"]
        fn from_ba(v: BigArray1<i64>) -> ArrayRef;
        #[namespace = "array_time64_us"]
        fn from(v: Vec<i64>) -> ArrayRef;
        #[namespace = "array_time64_us"]
        fn values(array: &ArrayRef, default: i64) -> Option<Vec<i64>>;
        #[namespace = "array_time64_us"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<i64>>>;
        #[namespace = "array_time64_us"]
        fn values_ba(array: &ArrayRef, default: i64) -> Option<BigArray1<i64>>;

        #[namespace = "array_timestamp_ns"]
        fn from_ba(v: BigArray1<i64>) -> ArrayRef;
        #[namespace = "array_timestamp_ns"]
        fn from(v: Vec<i64>) -> ArrayRef;
        #[namespace = "array_timestamp_ns"]
        fn values(array: &ArrayRef, default: i64) -> Option<Vec<i64>>;
        #[namespace = "array_timestamp_ns"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<i64>>>;
        #[namespace = "array_timestamp_ns"]
        fn values_ba(array: &ArrayRef, default: i64) -> Option<BigArray1<i64>>;

        #[namespace = "array_timestamp_us"]
        fn from_ba(v: BigArray1<i64>) -> ArrayRef;
        #[namespace = "array_timestamp_us"]
        fn from(v: Vec<i64>) -> ArrayRef;
        #[namespace = "array_timestamp_us"]
        fn values(array: &ArrayRef, default: i64) -> Option<Vec<i64>>;
        #[namespace = "array_timestamp_us"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<i64>>>;
        #[namespace = "array_timestamp_us"]
        fn values_ba(array: &ArrayRef, default: i64) -> Option<BigArray1<i64>>;

        #[namespace = "array_timestamp_ms"]
        fn from_ba(v: BigArray1<i64>) -> ArrayRef;
        #[namespace = "array_timestamp_ms"]
        fn from(v: Vec<i64>) -> ArrayRef;
        #[namespace = "array_timestamp_ms"]
        fn values(array: &ArrayRef, default: i64) -> Option<Vec<i64>>;
        #[namespace = "array_timestamp_ms"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<i64>>>;
        #[namespace = "array_timestamp_ms"]
        fn values_ba(array: &ArrayRef, default: i64) -> Option<BigArray1<i64>>;

        #[namespace = "array_timestamp_s"]
        fn from_ba(v: BigArray1<i64>) -> ArrayRef;
        #[namespace = "array_timestamp_s"]
        fn from(v: Vec<i64>) -> ArrayRef;
        #[namespace = "array_timestamp_s"]
        fn values(array: &ArrayRef, default: i64) -> Option<Vec<i64>>;
        #[namespace = "array_timestamp_s"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<i64>>>;
        #[namespace = "array_timestamp_s"]
        fn values_ba(array: &ArrayRef, default: i64) -> Option<BigArray1<i64>>;

        #[namespace = "array_date32"]
        fn from_ba(v: BigArray1<i32>) -> ArrayRef;
        #[namespace = "array_date32"]
        fn from(v: Vec<i32>) -> ArrayRef;
        #[namespace = "array_date32"]
        fn values(array: &ArrayRef, default: i32) -> Option<Vec<i32>>;
        #[namespace = "array_date32"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<i32>>>;
        #[namespace = "array_date32"]
        fn values_ba(array: &ArrayRef, default: i32) -> Option<BigArray1<i32>>;

        #[namespace = "array_date64"]
        fn from_ba(v: BigArray1<i64>) -> ArrayRef;
        #[namespace = "array_date64"]
        fn from(v: Vec<i64>) -> ArrayRef;
        #[namespace = "array_date64"]
        fn values(array: &ArrayRef, default: i64) -> Option<Vec<i64>>;
        #[namespace = "array_date64"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<i64>>>;
        #[namespace = "array_date64"]
        fn values_ba(array: &ArrayRef, default: i64) -> Option<BigArray1<i64>>;

        #[namespace = "array_char"]
        fn from_ba(v: BigArray1<u8>) -> ArrayRef;
        #[namespace = "array_char"]
        fn from(v: Vec<u8>) -> ArrayRef;
        #[namespace = "array_char"]
        fn values(array: &ArrayRef, default: u8) -> Option<Vec<u8>>;
        #[namespace = "array_char"]
        fn values_ba(array: &ArrayRef, default: u8) -> Option<BigArray1<u8>>;

        #[namespace = "array_i32"]
        fn from_ba(v: BigArray1<i32>) -> ArrayRef;
        #[namespace = "array_i32"]
        fn from(v: Vec<i32>) -> ArrayRef;
        #[namespace = "array_i32"]
        fn values(array: &ArrayRef, default: i32) -> Option<Vec<i32>>;
        #[namespace = "array_i32"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<i32>>>;
        #[namespace = "array_i32"]
        fn values_ba(array: &ArrayRef, default: i32) -> Option<BigArray1<i32>>;

        #[namespace = "array_i64"]
        fn from_ba(v: BigArray1<i64>) -> ArrayRef;
        #[namespace = "array_i64"]
        fn from(v: Vec<i64>) -> ArrayRef;
        #[namespace = "array_i64"]
        fn values(array: &ArrayRef, default: i64) -> Option<Vec<i64>>;
        #[namespace = "array_i64"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<i64>>>;
        #[namespace = "array_i64"]
        fn values_ba(array: &ArrayRef, default: i64) -> Option<BigArray1<i64>>;

        #[namespace = "array_f32"]
        fn from_ba(v: BigArray1<f32>) -> ArrayRef;
        #[namespace = "array_f32"]
        fn from(v: Vec<f32>) -> ArrayRef;
        #[namespace = "array_f32"]
        fn values(array: &ArrayRef, default: f32) -> Option<Vec<f32>>;
        #[namespace = "array_f32"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<f32>>>;
        #[namespace = "array_f32"]
        fn values_ba(array: &ArrayRef, default: f32) -> Option<BigArray1<f32>>;

        #[namespace = "array_f64"]
        fn from_ba(v: BigArray1<f64>) -> ArrayRef;
        #[namespace = "array_f64"]
        fn from(v: Vec<f64>) -> ArrayRef;
        #[namespace = "array_f64"]
        fn values(array: &ArrayRef, default: f64) -> Option<Vec<f64>>;
        #[namespace = "array_f64"]
        fn values_opt(array: &ArrayRef) -> Option<Vec<Option<f64>>>;
        #[namespace = "array_f64"]
        fn values_ba(array: &ArrayRef, default: f64) -> Option<BigArray1<f64>>;

        fn array_null(size: usize) -> ArrayRef;
        fn array_timestamp_ns_from_with_zone(v: Vec<i64>, zone: Option<String>) -> ArrayRef;

        fn array_string_from(v: Vec<String>) -> ArrayRef;
        fn array_large_string_from(v: Vec<String>) -> ArrayRef;
        fn array_string_values(array: &ArrayRef, default: String) -> Option<Vec<String>>;
        fn array_large_string_values(array: &ArrayRef, default: String) -> Option<Vec<String>>;
        fn array_string_values_opt(array: &ArrayRef) -> Option<Vec<Option<String>>>;
        fn array_large_string_values_opt(array: &ArrayRef) -> Option<Vec<Option<String>>>;
    }
}
