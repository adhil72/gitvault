//! Where things live inside a vault repository (see `docs/vault-format.md`).
//!
//! ```text
//! .gitvault/vault.json     plaintext header (KDF settings, wrapped keys)
//! secrets/<secret-id>.enc  one encrypted file per secret
//! ```

use crate::domain::ids::SecretId;

pub const HEADER_PATH: &str = ".gitvault/vault.json";
pub const SECRETS_DIR: &str = "secrets";
pub const SECRET_EXTENSION: &str = "enc";

/// Repository-relative path of a secret file.
#[must_use]
pub fn secret_path(secret: &SecretId) -> String {
    format!("{SECRETS_DIR}/{secret}.{SECRET_EXTENSION}")
}

#[cfg(test)]
mod tests {
    use super::secret_path;
    use crate::domain::ids::SecretId;

    #[test]
    fn builds_relative_path() {
        assert_eq!(
            secret_path(&SecretId::parse("abc").unwrap()),
            "secrets/abc.enc"
        );
    }
}
