use ocaml_rust::Custom;
use parquet::arrow::{ArrowReader, ParquetFileArrowReader};
use parquet::file::reader::{FileReader, SerializedFileReader};
use std::fs::File;

struct ParquetReader {
    file_reader: std::sync::Arc<SerializedFileReader<File>>,
    arrow_reader: ParquetFileArrowReader,
}

fn reader(path: String) -> Result<Reader, String> {
    // TODO: Improve the error handling to avoid the [to_string] bit.
    let file = File::open(&path).map_err(|x| x.to_string())?;
    let file_reader = SerializedFileReader::new(file).map_err(|x| x.to_string())?;
    let file_reader = std::sync::Arc::new(file_reader);
    let arrow_reader = ParquetFileArrowReader::new(file_reader.clone());
    let reader = ParquetReader { file_reader, arrow_reader };
    Ok(Custom::new(reader))
}

fn metadata_as_string(reader: &Reader) -> String {
    let reader = reader.inner().lock().unwrap();
    format!("{:?}", reader.file_reader.metadata())
}

fn parquet_metadata(reader: &Reader) -> Metadata {
    let reader = reader.inner().lock().unwrap();
    let metadata = reader.file_reader.metadata();
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

fn schema(reader: &Reader) -> Result<Schema, String> {
    let mut reader = reader.inner().lock().unwrap();
    let schema = reader.arrow_reader.get_schema().map_err(|x| x.to_string())?;
    let fields: Vec<_> = schema
        .fields()
        .iter()
        .map(|field| SchemaField {
            name: field.name().to_string(),
            data_type: field.data_type().to_string(),
            nullable: field.is_nullable(),
        })
        .collect();
    let metadata: Vec<(String, String)> =
        schema.metadata().iter().map(|(x, y)| (x.to_string(), y.to_string())).collect();
    Ok(Schema { fields, metadata })
}

// TODO: These should be derived automatically when needed.
impl ocaml_rust::from_value::NotF64 for RowGroupMetadata {}
impl ocaml_rust::from_value::NotF64 for SchemaField {}

#[ocaml_rust::bridge]
mod arrow {
    ocaml_include!("open! Sexplib.Conv");
    type Reader = Custom<ParquetReader>;

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
        data_type: String,
        nullable: bool,
    }

    #[ocaml_deriving(sexp)]
    #[derive(Debug, Clone)]
    struct Schema {
        fields: Vec<SchemaField>,
        metadata: Vec<(String, String)>,
    }

    extern "Rust" {
        fn reader(path: String) -> Result<Reader, String>;
        fn metadata_as_string(reader: &Reader) -> String;
        fn parquet_metadata(reader: &Reader) -> Metadata;
        fn schema(reader: &Reader) -> Result<Schema, String>;
    }
}
