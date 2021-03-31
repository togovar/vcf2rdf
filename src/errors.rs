//! Module for defining errors for this application
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    RustHtslibError(#[from] rust_htslib::errors::Error),
}
