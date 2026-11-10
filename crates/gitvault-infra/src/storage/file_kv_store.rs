use std::{collections::BTreeMap, path::PathBuf, sync::Mutex};

use gitvault_core::{ports::KeyValueStore, CoreError};

use super::io::{read_optional, storage_error, write_atomic};

/// Private key-value state in a single JSON file readable only by the user (0600).
///
/// The file is re-read on every operation so the CLI and the desktop app can
/// share it safely.
pub struct FileKeyValueStore {
    path: PathBuf,
    guard: Mutex<()>,
}

impl FileKeyValueStore {
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            guard: Mutex::new(()),
        }
    }

    fn read_map(&self) -> Result<BTreeMap<String, String>, CoreError> {
        match read_optional(&self.path)? {
            None => Ok(BTreeMap::new()),
            Some(bytes) => serde_json::from_slice(&bytes).map_err(storage_error),
        }
    }

    fn write_map(&self, map: &BTreeMap<String, String>) -> Result<(), CoreError> {
        let json = serde_json::to_vec_pretty(map).map_err(storage_error)?;
        write_atomic(&self.path, &json)
    }

    fn locked(&self) -> Result<std::sync::MutexGuard<'_, ()>, CoreError> {
        self.guard
            .lock()
            .map_err(|_| storage_error("state lock poisoned"))
    }
}

impl KeyValueStore for FileKeyValueStore {
    fn get(&self, key: &str) -> Result<Option<String>, CoreError> {
        let _lock = self.locked()?;
        Ok(self.read_map()?.remove(key))
    }

    fn set(&self, key: &str, value: &str) -> Result<(), CoreError> {
        let _lock = self.locked()?;
        let mut map = self.read_map()?;
        map.insert(key.to_owned(), value.to_owned());
        self.write_map(&map)
    }

    fn delete(&self, key: &str) -> Result<(), CoreError> {
        let _lock = self.locked()?;
        let mut map = self.read_map()?;
        if map.remove(key).is_some() {
            self.write_map(&map)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_reads_and_deletes_values() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileKeyValueStore::new(dir.path().join("state.json"));
        assert_eq!(store.get("a").unwrap(), None);
        store.set("a", "1").unwrap();
        store.set("b", "2").unwrap();
        assert_eq!(store.get("a").unwrap().as_deref(), Some("1"));
        store.delete("a").unwrap();
        store.delete("missing").unwrap();
        assert_eq!(store.get("a").unwrap(), None);
        assert_eq!(store.get("b").unwrap().as_deref(), Some("2"));
    }

    #[cfg(unix)]
    #[test]
    fn state_file_is_private() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        FileKeyValueStore::new(path.clone()).set("k", "v").unwrap();
        assert_eq!(
            std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
