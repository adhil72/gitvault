use crate::error::CoreError;

const MAX_NAME_LEN: usize = 80;

/// A validated vault display name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultName(String);

impl VaultName {
    /// # Errors
    /// Returns a validation error for empty, long or control-character names.
    pub fn parse(raw: &str) -> Result<Self, CoreError> {
        let name = raw.trim();
        if name.is_empty()
            || name.chars().count() > MAX_NAME_LEN
            || name.chars().any(char::is_control)
        {
            return Err(CoreError::Validation(format!(
                "vault name must be 1-{MAX_NAME_LEN} characters without control characters"
            )));
        }
        Ok(Self(name.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::VaultName;

    #[test]
    fn validates() {
        assert!(VaultName::parse("Personal vault").is_ok());
        assert!(VaultName::parse(" ").is_err());
        assert!(VaultName::parse(&"x".repeat(81)).is_err());
    }
}
