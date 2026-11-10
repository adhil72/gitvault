use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What a secret holds. Decides masking and which fields the UI shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum SecretKind {
    ApiKey,
    Password,
    Certificate,
    Note,
    EnvFile,
}

impl SecretKind {
    /// Parses the wire/CLI name (`api_key`, `password`, ...).
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().replace('-', "_").as_str() {
            "api_key" | "apikey" | "api" => Some(Self::ApiKey),
            "password" => Some(Self::Password),
            "certificate" | "cert" => Some(Self::Certificate),
            "note" => Some(Self::Note),
            "env_file" | "env" => Some(Self::EnvFile),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SecretKind;

    #[test]
    fn parses_aliases() {
        assert_eq!(SecretKind::parse("api-key"), Some(SecretKind::ApiKey));
        assert_eq!(SecretKind::parse("CERT"), Some(SecretKind::Certificate));
        assert_eq!(SecretKind::parse("nope"), None);
    }
}
