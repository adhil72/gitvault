use std::{
    fs,
    path::{Path, PathBuf},
};

use gitvault_core::{
    domain::{
        ids::{SecretId, VaultId},
        vault::{
            layout::{secret_path, HEADER_PATH, SECRETS_DIR, SECRET_EXTENSION},
            VaultHeader,
        },
    },
    ports::VaultStorage,
    CoreError,
};

use super::io::{read_optional, storage_error, write_atomic};

/// Vault clones on disk: `<root>/<vault-id>/` (the git working directory).
pub struct FsVaultStorage {
    root: PathBuf,
}

impl FsVaultStorage {
    #[must_use]
    pub const fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn dir(&self, id: &VaultId) -> PathBuf {
        self.root.join(id.as_str())
    }
}

impl VaultStorage for FsVaultStorage {
    fn vault_ids(&self) -> Result<Vec<VaultId>, CoreError> {
        let Ok(entries) = fs::read_dir(&self.root) else {
            return Ok(Vec::new());
        };
        let mut ids: Vec<VaultId> = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.path().join(HEADER_PATH).is_file())
            .filter_map(|entry| VaultId::parse(&entry.file_name().to_string_lossy()).ok())
            .collect();
        ids.sort();
        Ok(ids)
    }

    fn exists(&self, id: &VaultId) -> bool {
        self.dir(id).exists()
    }

    fn create(&self, id: &VaultId) -> Result<(), CoreError> {
        fs::create_dir_all(&self.root).map_err(storage_error)?;
        fs::create_dir(self.dir(id)).map_err(storage_error)
    }

    fn read_header(&self, id: &VaultId) -> Result<VaultHeader, CoreError> {
        let bytes = read_optional(&self.dir(id).join(HEADER_PATH))?
            .ok_or(CoreError::NotFound { entity: "vault" })?;
        serde_json::from_slice(&bytes)
            .map_err(|e| CoreError::Storage(format!("malformed vault header: {e}")))
    }

    fn write_header(&self, id: &VaultId, header: &VaultHeader) -> Result<(), CoreError> {
        let json = serde_json::to_vec_pretty(header).map_err(storage_error)?;
        write_atomic(&self.dir(id).join(HEADER_PATH), &json)
    }

    fn secret_ids(&self, id: &VaultId) -> Result<Vec<SecretId>, CoreError> {
        let Ok(entries) = fs::read_dir(self.dir(id).join(SECRETS_DIR)) else {
            return Ok(Vec::new());
        };
        let mut ids: Vec<SecretId> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == SECRET_EXTENSION))
            .filter_map(|path| path.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .filter_map(|stem| SecretId::parse(&stem).ok())
            .collect();
        ids.sort();
        Ok(ids)
    }

    fn read_secret(&self, id: &VaultId, secret: &SecretId) -> Result<Option<Vec<u8>>, CoreError> {
        read_optional(&self.dir(id).join(secret_path(secret)))
    }

    fn write_secret(&self, id: &VaultId, secret: &SecretId, bytes: &[u8]) -> Result<(), CoreError> {
        write_atomic(&self.dir(id).join(secret_path(secret)), bytes)
    }

    fn delete_secret(&self, id: &VaultId, secret: &SecretId) -> Result<(), CoreError> {
        match fs::remove_file(self.dir(id).join(secret_path(secret))) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(storage_error(e)),
        }
    }

    fn rename(&self, from: &VaultId, to: &VaultId) -> Result<(), CoreError> {
        let target = self.dir(to);
        if target.exists() {
            return Err(CoreError::Conflict("vault already exists".into()));
        }
        fs::rename(self.dir(from), target).map_err(storage_error)
    }

    fn remove(&self, id: &VaultId) -> Result<(), CoreError> {
        let dir: &Path = &self.dir(id);
        if dir.exists() {
            fs::remove_dir_all(dir).map_err(storage_error)?;
        }
        Ok(())
    }
}
