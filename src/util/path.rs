use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::errors::{Error, Result};

pub fn change_extension<P: AsRef<Path>, S: AsRef<OsStr>>(path: P, ext: S) -> Result<PathBuf> {
    let mut path = path.as_ref().to_path_buf();

    if !path.set_extension(ext) {
        return Err(Error::FilePathError(path.to_string_lossy().to_string()));
    }

    Ok(path)
}
