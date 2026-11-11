//! Identifier newtypes. Ids become file and directory names, so they are
//! restricted to a safe alphabet (no path separators, no dots).

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

const MAX_ID_LEN: usize = 64;

fn validate(raw: &str, what: &str) -> Result<(), CoreError> {
    let valid_char = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
    if raw.is_empty() || raw.len() > MAX_ID_LEN || !raw.chars().all(valid_char) {
        return Err(CoreError::Validation(format!("invalid {what} id")));
    }
    Ok(())
}

macro_rules! id_type {
    ($name:ident, $what:literal) => {
        // `try_from` makes deserialization (e.g. a cloned repo's vault header) go through the
        // same validation as `parse`, so an untrusted id can never contain path separators.
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl TryFrom<String> for $name {
            type Error = CoreError;

            fn try_from(raw: String) -> Result<Self, CoreError> {
                validate(&raw, $what)?;
                Ok(Self(raw))
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl $name {
            /// Validates and wraps an id.
            ///
            /// # Errors
            /// Returns a validation error for empty, long or unsafe ids.
            pub fn parse(raw: &str) -> Result<Self, CoreError> {
                validate(raw, $what)?;
                Ok(Self(raw.to_owned()))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// First 8 characters, used in commit messages and short displays.
            #[must_use]
            pub fn short(&self) -> &str {
                let end = self.0.len().min(8);
                self.0.get(..end).unwrap_or(&self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

id_type!(VaultId, "vault");
id_type!(SecretId, "secret");

#[cfg(test)]
mod tests {
    use super::{SecretId, VaultId};

    #[test]
    fn accepts_uuids() {
        assert!(VaultId::parse("7f3c9a10-2b4d-4e8f-9a11-0c5d6e7f8a9b").is_ok());
    }

    #[test]
    fn rejects_path_traversal_and_separators() {
        for bad in ["", "..", "a/b", "a\\b", "a.b", "../etc", &"x".repeat(65)] {
            assert!(SecretId::parse(bad).is_err(), "{bad} should be rejected");
        }
    }

    #[test]
    fn deserializing_rejects_unsafe_ids() {
        // A cloned repo controls its header, so deserialization must validate too.
        for bad in ["\"../../escape\"", "\"a/b\"", "\"\"", "\"a.b\""] {
            assert!(serde_json::from_str::<VaultId>(bad).is_err(), "{bad}");
            assert!(serde_json::from_str::<SecretId>(bad).is_err(), "{bad}");
        }
        let ok: VaultId = serde_json::from_str("\"7f3c-9a10\"").unwrap();
        assert_eq!(serde_json::to_string(&ok).unwrap(), "\"7f3c-9a10\"");
    }

    #[test]
    fn short_is_at_most_eight_chars() {
        assert_eq!(
            VaultId::parse("abcdef0123456789").unwrap().short(),
            "abcdef01"
        );
        assert_eq!(VaultId::parse("abc").unwrap().short(), "abc");
    }
}
