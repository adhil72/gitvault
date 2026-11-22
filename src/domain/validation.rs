use crate::error::AppError;

pub const MIN_PASSWORD_LEN: usize = 8;

/// Validates a project or environment identifier (safe for paths and AAD).
pub fn validate_slug(raw: &str, kind: &str) -> Result<String, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return Err(AppError::Validation(format!(
            "{kind} name must be 1..=64 characters"
        )));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    {
        return Err(AppError::Validation(format!(
            "{kind} name '{trimmed}' may only contain letters, digits, '-' and '_'"
        )));
    }
    Ok(trimmed.to_owned())
}

/// Validates an environment variable key (POSIX identifier).
pub fn validate_var_key(raw: &str) -> Result<String, AppError> {
    let trimmed = raw.trim();
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return Err(AppError::Validation("variable name cannot be empty".into()));
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(AppError::Validation(format!(
            "variable name '{trimmed}' must start with a letter or '_'"
        )));
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(AppError::Validation(format!(
            "variable name '{trimmed}' may only contain letters, digits, and '_'"
        )));
    }
    Ok(trimmed.to_owned())
}

pub fn check_password(password: &str) -> Result<(), AppError> {
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(AppError::Validation(format!(
            "master password must be at least {MIN_PASSWORD_LEN} characters"
        )));
    }
    Ok(())
}

/// Masks a secret value for display in terminal menus and tables.
#[must_use]
pub fn mask_value(value: &str) -> String {
    let count = value.chars().count();
    if count >= 12 {
        let prefix: String = value.chars().take(2).collect();
        let suffix: String = value.chars().skip(count - 2).collect();
        format!("{prefix}••••••{suffix}")
    } else {
        "••••••••".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_traversal_and_invalid_keys() {
        assert!(validate_slug("../etc", "project").is_err());
        assert!(validate_slug("a/b", "project").is_err());
        assert!(validate_slug("", "project").is_err());
        assert!(validate_slug("shop_api-1", "project").is_ok());

        assert!(validate_var_key("123BAD").is_err());
        assert!(validate_var_key("BAD-KEY").is_err());
        assert!(validate_var_key("KEY;rm").is_err());
        assert!(validate_var_key("DATABASE_URL").is_ok());
    }
}
