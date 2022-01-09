//! Module for defining errors for this application
use thiserror::Error;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    RustHtslibError(#[from] rust_htslib::errors::Error),

    #[error(transparent)]
    NulError(#[from] std::ffi::NulError),

    #[error(transparent)]
    SerdeYamlError(#[from] serde_yaml::Error),

    #[error("File not found: {0}")]
    FileNotFoundError(String),

    #[error("Path contains invalid character: {0}")]
    FilePathError(String),

    #[error("Index file not found: {0}")]
    IndexNotFoundError(String),

    #[error("Both reference and alternate must not be empty.")]
    InvalidRefAltError,

    #[error("Invalid reference ID.")]
    ReferenceIndexError,

    #[error("The compression of {0} is not BGZF")]
    NotBgzipFileError(String),

    #[error("Can't create {0}")]
    BgzipCreateError(String),

    #[error("Close failed")]
    BgzipCloseError,

    #[error("Could not write {0} bytes")]
    BgzipWriteError(usize),

    #[error("tbx_index_build failed: {0}")]
    IndexBuildFailedError(String),

    #[error("Missing configuration: {0}")]
    ConfigurationNotFoundError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfigurationError(String),
}
