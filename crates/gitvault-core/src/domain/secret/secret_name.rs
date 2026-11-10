//! A validated secret name such as `STRIPE_SECRET_KEY`.

use crate::error::CoreError;

/// Parse-don't-validate: a `SecretName` can only exist if it is valid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretName(String);

impl SecretName {
    /// Longest accepted name, in characters.
    pub const MAX_LEN: usize = 128;

    /// Builds a name from user input.
    ///
    /// # Errors
    /// Returns [`CoreError::Validation`] when the name is empty, too long, or
    /// contains characters other than letters, digits, `_`, `-` and `.`.
    pub fn parse(raw: &str) -> Result<Self, CoreError> {
        let name = raw.trim();
        if name.is_empty() {
            return Err(CoreError::Validation(
                "secret name must not be empty".into(),
            ));
        }
        if name.chars().count() > Self::MAX_LEN {
            return Err(CoreError::Validation(format!(
                "secret name must be at most {} characters",
                Self::MAX_LEN
            )));
        }
        if !name.chars().all(is_allowed_char) {
            return Err(CoreError::Validation(
                "secret name may only contain letters, digits, '_', '-' and '.'".into(),
            ));
        }
        Ok(Self(name.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_allowed_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')
}

#[cfg(test)]
mod tests {
    use super::SecretName;

    #[test]
    fn accepts_typical_env_style_name() {
        assert_eq!(
            SecretName::parse("STRIPE_SECRET_KEY").unwrap().as_str(),
            "STRIPE_SECRET_KEY"
        );
    }

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(SecretName::parse("  API_KEY ").unwrap().as_str(), "API_KEY");
    }

    #[test]
    fn rejects_empty_spaces_and_overlong() {
        assert!(SecretName::parse("   ").is_err());
        assert!(SecretName::parse("MY KEY").is_err());
        assert!(SecretName::parse(&"A".repeat(SecretName::MAX_LEN + 1)).is_err());
    }
}
