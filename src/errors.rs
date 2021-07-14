//! Module for defining errors for this application
pub use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("argument error: {0}")]
    ArgumentError(String),

    #[error("file not found: {0}")]
    FileNotFoundError(String),

    #[error("path contains invalid character: {0}")]
    FilePathError(String),

    #[error("index file not found: {0}")]
    IndexNotFoundError(String),

    #[error("Both reference and alternate must not be empty.")]
    InvalidRefAltError,

    #[error("Chromosomes not found: {0}")]
    NoChromosomeError(String),

    #[error("Chromosomes cannot be uniquely determined: {0}")]
    AmbiguousChromosomeError(String),

    #[error("Invalid reference ID.")]
    ReferenceIndexError,

    #[error("Reference not found for index={0}")]
    ReferenceSequenceNotFoundError(u32),

    #[error("Reference name not found for {0}")]
    ReferenceNameNotFoundError(String),

    #[error("Info definition not found for {0}")]
    InfoNotFoundError(String),

    #[error("Unsupported assembly version: {0}")]
    UnsupportedAssemblyError(String),

    #[error("The compression of {0} is not BGZF")]
    NotBgzipFileError(String),

    #[error("Can't create {0}")]
    BgzipCreateError(String),

    #[error("Close failed")]
    BgzipCloseError,

    #[error("Could not write {0} bytes")]
    BgzipWriteError(usize),

    #[error("Could not write index to {0}")]
    IndexWriteError(String),

    #[error("tbx_index_build failed: {0}")]
    IndexBuildFailedError(String),
}
