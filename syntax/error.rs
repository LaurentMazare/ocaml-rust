use thiserror::Error;

/// Main library error type.
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Syn error.
    #[error(transparent)]
    Syn(#[from] syn::Error),
}
