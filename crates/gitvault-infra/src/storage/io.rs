//! Small file helpers shared by the storage adapters.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use gitvault_core::CoreError;

pub fn storage_error(error: impl std::fmt::Display) -> CoreError {
    CoreError::Storage(error.to_string())
}

/// Writes a file atomically (temp file + rename). On Unix the file is private (0600).
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), CoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(storage_error)?;
    }
    let temp: PathBuf = path.with_extension("tmp");
    let mut file = open_private(&temp)?;
    file.write_all(bytes).map_err(storage_error)?;
    file.sync_all().map_err(storage_error)?;
    fs::rename(&temp, path).map_err(storage_error)
}

#[cfg(unix)]
fn open_private(path: &Path) -> Result<fs::File, CoreError> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .map_err(storage_error)
}

#[cfg(not(unix))]
fn open_private(path: &Path) -> Result<fs::File, CoreError> {
    fs::File::create(path).map_err(storage_error)
}

pub fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, CoreError> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(storage_error(error)),
    }
}
