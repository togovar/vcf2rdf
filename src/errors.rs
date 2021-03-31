//! Module for defining errors for this application
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    FFINulError(#[from] std::ffi::NulError),
    #[error(transparent)]
    RustHTSlibError(#[from] rust_htslib::errors::Error),
    #[error("file not found: {path:?}")]
    FileNotFoundError { path: String },
    #[error("path contains invalid character: {path:?}")]
    FilePathError { path: String },
    #[error("index file not found: {path:?}")]
    IndexNotFoundError { path: String },
}
