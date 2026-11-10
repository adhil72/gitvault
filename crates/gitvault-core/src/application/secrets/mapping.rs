//! Conversions between secret payloads and DTOs, plus input normalization.

use crate::{
    domain::{
        ids::SecretId,
        label::{parse_label, parse_optional_label},
        secret::{SecretName, SecretPayload},
    },
    dto::{SecretDetail, SecretSummary},
    error::CoreError,
};

const SECONDS_PER_DAY: i64 = 86_400;

/// Whole days until expiry (negative once expired).
#[must_use]
pub fn expires_in_days(expires_at: Option<i64>, now: i64) -> Option<i64> {
    expires_at.map(|at| (at - now).div_euclid(SECONDS_PER_DAY))
}

#[must_use]
pub fn summary(id: &SecretId, payload: &SecretPayload, now: i64) -> SecretSummary {
    SecretSummary {
        id: id.to_string(),
        name: payload.name.clone(),
        kind: payload.kind,
        environment: payload.environment.clone(),
        tags: payload.tags.clone(),
        favorite: payload.favorite,
        expires_at: payload.expires_at,
        expires_in_days: expires_in_days(payload.expires_at, now),
        created_at: payload.created_at,
        updated_at: payload.updated_at,
        version: payload.version,
    }
}

#[must_use]
pub fn detail(id: &SecretId, payload: &SecretPayload, now: i64) -> SecretDetail {
    SecretDetail {
        summary: summary(id, payload, now),
        notes: payload.notes.clone(),
        remind_before_days: payload.remind_before_days,
    }
}

/// Unvalidated user input for the descriptive fields of a secret.
pub struct RawFields<'a> {
    pub name: &'a str,
    pub environment: Option<&'a str>,
    pub tags: &'a [String],
    pub notes: Option<&'a str>,
}

/// Validated, trimmed descriptive fields.
pub struct Fields {
    pub name: String,
    pub environment: Option<String>,
    pub tags: Vec<String>,
    pub notes: String,
}

/// Validates and normalizes user input (trim, dedupe tags, parse the name).
///
/// # Errors
/// Validation error for any invalid field.
pub fn normalize(raw: &RawFields<'_>) -> Result<Fields, CoreError> {
    let mut tags: Vec<String> = Vec::new();
    for tag in raw.tags {
        let tag = parse_label(tag, "tag")?;
        if !tags.iter().any(|t| t.eq_ignore_ascii_case(&tag)) {
            tags.push(tag);
        }
    }
    Ok(Fields {
        name: SecretName::parse(raw.name)?.as_str().to_owned(),
        environment: parse_optional_label(raw.environment, "environment")?,
        tags,
        notes: raw.notes.unwrap_or_default().trim_end().to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expiry_counts_whole_days_and_goes_negative() {
        assert_eq!(expires_in_days(Some(86_400 * 10), 0), Some(10));
        assert_eq!(expires_in_days(Some(-1), 0), Some(-1));
        assert_eq!(expires_in_days(None, 0), None);
    }

    #[test]
    fn normalize_dedupes_tags_and_trims() {
        let tags = vec![
            "billing".to_owned(),
            " Billing ".to_owned(),
            "api".to_owned(),
        ];
        let fields = normalize(&RawFields {
            name: " STRIPE_KEY ",
            environment: Some(""),
            tags: &tags,
            notes: Some("hello\n"),
        })
        .unwrap();
        assert_eq!(fields.name, "STRIPE_KEY");
        assert_eq!(fields.environment, None);
        assert_eq!(fields.tags, vec!["billing", "api"]);
        assert_eq!(fields.notes, "hello");
    }
}
