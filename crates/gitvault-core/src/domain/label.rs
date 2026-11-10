//! Short free-text labels: environments and tags.

use crate::error::CoreError;

const MAX_LABEL_LEN: usize = 64;

/// A trimmed, non-empty label without control characters.
///
/// # Errors
/// Returns a validation error when the label is empty, too long or has control characters.
pub fn parse_label(raw: &str, what: &str) -> Result<String, CoreError> {
    let label = raw.trim();
    if label.is_empty() {
        return Err(CoreError::Validation(format!("{what} must not be empty")));
    }
    if label.chars().count() > MAX_LABEL_LEN {
        return Err(CoreError::Validation(format!(
            "{what} must be at most {MAX_LABEL_LEN} characters"
        )));
    }
    if label.chars().any(char::is_control) {
        return Err(CoreError::Validation(format!(
            "{what} contains control characters"
        )));
    }
    Ok(label.to_owned())
}

/// Parses an optional label: blank input means "none".
///
/// # Errors
/// Same as [`parse_label`] for non-blank input.
pub fn parse_optional_label(raw: Option<&str>, what: &str) -> Result<Option<String>, CoreError> {
    match raw.map(str::trim) {
        None | Some("") => Ok(None),
        Some(text) => parse_label(text, what).map(Some),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_label, parse_optional_label};

    #[test]
    fn trims_and_accepts() {
        assert_eq!(parse_label("  Stripe ", "label").unwrap(), "Stripe");
    }

    #[test]
    fn blank_optional_is_none() {
        assert_eq!(parse_optional_label(Some("  "), "label").unwrap(), None);
        assert_eq!(parse_optional_label(None, "label").unwrap(), None);
    }

    #[test]
    fn rejects_long_and_control_characters() {
        assert!(parse_label(&"a".repeat(65), "tag").is_err());
        assert!(parse_label("a\nb", "tag").is_err());
    }
}
