use arrow::datatypes::DataType as DT;
use ocaml_rust::Custom;
use parquet::arrow::{ArrowReader, ParquetFileArrowReader};
use parquet::file::reader::SerializedFileReader;
use std::fs::File;

fn reader(path: String) -> ocaml_rust::RustResult<Reader> {
    let file = File::open(&path)?;
    let file_reader = SerializedFileReader::new(file)?;
    let file_reader = std::sync::Arc::new(file_reader);
    let reader = ParquetFileArrowReader::new(file_reader);
    Ok(Custom::new(reader))
}

fn metadata_as_string(reader: &Reader) -> String {
    let mut reader = reader.inner().lock().unwrap();
    format!("{:?}", reader.get_metadata())
}

fn parquet_metadata(reader: &Reader) -> Metadata {
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

fn schema(reader: &Reader) -> ocaml_rust::RustResult<Schema> {
    let mut reader = reader.inner().lock().unwrap();
    let schema = reader.get_schema()?;
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
    Ok(Schema { fields, metadata })
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
    type Reader = Custom<ParquetFileArrowReader>;

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
        fn reader(path: String) -> RustResult<Reader>;
        fn metadata_as_string(reader: &Reader) -> String;
        fn parquet_metadata(reader: &Reader) -> Metadata;
        fn schema(reader: &Reader) -> RustResult<Schema>;
    }
}
