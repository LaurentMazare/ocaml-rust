use ocaml_rust::Custom;
use parquet::arrow::{ArrowReader, ParquetFileArrowReader};
use parquet::file::reader::{FileReader, SerializedFileReader};
use std::fs::File;

struct ParquetReader {
    file_reader: std::sync::Arc<SerializedFileReader<File>>,
    arrow_reader: ParquetFileArrowReader,
}

type Reader = Custom<ParquetReader>;

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

#[ocaml_rust::bridge]
mod arrow {
    ocaml_include!("open! Sexplib.Conv");
    // TODO: Get this generated automatically.
    ocaml_include!("type reader");

    extern "Rust" {
        fn reader(path: String) -> Result<Reader, String>;
        fn metadata_as_string(reader: &Reader) -> String;
    }
}
