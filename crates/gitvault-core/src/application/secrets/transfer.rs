//! `.env` import/export and environment resolution for `run`.

use super::SecretService;
use crate::domain::label::parse_optional_label;
use crate::{
    domain::{
        audit::AuditKind,
        env_file::{format_env, is_valid_env_name, parse_env, EnvEntry},
        ids::SecretId,
        secret::{SecretKind, SecretPayload, SecretValue},
    },
    dto::{EnvFilter, ImportEnvInput, ImportPreviewEntry, ImportReport, ImportStatus},
    error::CoreError,
};

const MASK_VISIBLE: usize = 4;

/// Shows only the last characters of a value.
#[must_use]
pub fn mask(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= MASK_VISIBLE * 2 {
        return "•".repeat(chars.len().max(4));
    }
    let tail: String = chars[chars.len() - MASK_VISIBLE..].iter().collect();
    format!("{}{tail}", "•".repeat(8))
}

fn matches_filter(payload: &SecretPayload, filter: &EnvFilter) -> bool {
    let eq = |field: Option<&str>, want: &Option<String>| {
        want.as_deref()
            .is_none_or(|w| field.is_some_and(|f| f.eq_ignore_ascii_case(w)))
    };
    eq(payload.environment.as_deref(), &filter.environment)
        && filter
            .tag
            .as_deref()
            .is_none_or(|t| payload.tags.iter().any(|x| x.eq_ignore_ascii_case(t)))
}

impl SecretService {
    /// What an import would do, without changing anything.
    ///
    /// # Errors
    /// Locked vault or a storage error.
    pub fn preview_env(
        &self,
        input: &ImportEnvInput,
    ) -> Result<Vec<ImportPreviewEntry>, CoreError> {
        let (vault, key) = self.open(&input.vault_id)?;
        let environment = parse_optional_label(input.environment.as_deref(), "environment")?;
        let all = self.load_all(&vault, &key)?;
        Ok(parse_env(&input.content)
            .into_iter()
            .map(|entry| {
                let exists = Self::find(&all, &entry.name, environment.as_deref()).is_some();
                ImportPreviewEntry {
                    masked_value: mask(&entry.value),
                    name: entry.name,
                    status: if exists {
                        ImportStatus::Duplicate
                    } else {
                        ImportStatus::New
                    },
                }
            })
            .collect())
    }

    fn find<'a>(
        all: &'a super::Loaded,
        name: &str,
        environment: Option<&str>,
    ) -> Option<&'a SecretId> {
        all.iter()
            .find(|(_, p)| {
                p.name.eq_ignore_ascii_case(name)
                    && p.environment.as_deref().map(str::to_lowercase)
                        == environment.map(str::to_lowercase)
            })
            .map(|(id, _)| id)
    }

    /// Imports `.env` text as secrets in a single commit.
    ///
    /// # Errors
    /// Locked vault, validation errors, or a storage error.
    pub fn import_env(&self, input: &ImportEnvInput) -> Result<ImportReport, CoreError> {
        let (vault, key) = self.open(&input.vault_id)?;
        let environment = parse_optional_label(input.environment.as_deref(), "environment")?;
        let mut all = self.load_all(&vault, &key)?;
        let mut report = ImportReport {
            created: 0,
            updated: 0,
            skipped: 0,
        };
        let now = self.ctx.now();
        for entry in parse_env(&input.content) {
            match Self::find(&all, &entry.name, environment.as_deref()).cloned() {
                Some(id) if input.overwrite => {
                    self.overwrite_value(&vault, &key, &mut all, &id, &entry, now)?;
                    report.updated += 1;
                }
                Some(_) => report.skipped += 1,
                None => {
                    let id = SecretId::parse(&self.ctx.ids.new_id())?;
                    let payload = new_env_payload(&entry, environment.clone(), now);
                    self.write_file(&vault, &key, &id, &payload)?;
                    all.push((id, payload));
                    report.created += 1;
                }
            }
        }
        if report.created + report.updated > 0 {
            self.commit(
                &vault,
                &format!("import: {} secrets", report.created + report.updated),
            )?;
            self.ctx.record(
                &vault,
                AuditKind::Imported,
                None,
                Some(&format!("{} secrets", report.created + report.updated)),
            );
        }
        Ok(report)
    }

    #[allow(clippy::too_many_arguments)] // vault, key, id and the change description belong together
    fn overwrite_value(
        &self,
        vault: &crate::domain::ids::VaultId,
        key: &gitvault_crypto::VaultKey,
        all: &mut super::Loaded,
        id: &SecretId,
        entry: &EnvEntry,
        now: i64,
    ) -> Result<(), CoreError> {
        if let Some((_, payload)) = all.iter_mut().find(|(existing, _)| existing == id) {
            payload.value = SecretValue::new(entry.value.clone());
            payload.version += 1;
            payload.updated_at = now;
            self.write_file(vault, key, id, payload)?;
        }
        Ok(())
    }

    /// Secrets matching the filter whose names are valid environment variables.
    ///
    /// # Errors
    /// Locked vault or a storage error.
    pub fn resolve_env(
        &self,
        vault_id: &str,
        filter: &EnvFilter,
    ) -> Result<Vec<EnvEntry>, CoreError> {
        let (vault, key) = self.open(vault_id)?;
        let mut entries: Vec<EnvEntry> = self
            .load_all(&vault, &key)?
            .into_iter()
            .filter(|(_, p)| {
                p.kind != SecretKind::Certificate
                    && is_valid_env_name(&p.name)
                    && matches_filter(p, filter)
            })
            .map(|(_, p)| EnvEntry {
                name: p.name,
                value: p.value.expose().to_owned(),
            })
            .collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries.dedup_by(|a, b| a.name == b.name);
        Ok(entries)
    }

    /// The variables an app should load for one environment: that environment's secrets plus
    /// shared ones (no environment set), with the environment's own value winning.
    /// Recorded in the audit log.
    ///
    /// # Errors
    /// Locked vault or a storage error.
    pub fn resolve_environment(
        &self,
        vault_id: &str,
        environment: &str,
    ) -> Result<Vec<EnvEntry>, CoreError> {
        let (vault, key) = self.open(vault_id)?;
        let mut merged: std::collections::BTreeMap<String, (bool, String)> = Default::default();
        for (_, payload) in self.load_all(&vault, &key)? {
            let own = payload
                .environment
                .as_deref()
                .is_some_and(|e| e.eq_ignore_ascii_case(environment));
            let shared = payload.environment.is_none();
            if !(own || shared) || payload.kind == SecretKind::Certificate {
                continue;
            }
            if !is_valid_env_name(&payload.name) {
                continue;
            }
            let wins = merged
                .get(&payload.name)
                .is_none_or(|(was_own, _)| own || !was_own);
            if wins {
                merged.insert(payload.name, (own, payload.value.expose().to_owned()));
            }
        }
        self.ctx.record(
            &vault,
            AuditKind::Exported,
            None,
            Some(&format!(
                "environment {environment}: {} variables",
                merged.len()
            )),
        );
        Ok(merged
            .into_iter()
            .map(|(name, (_, value))| EnvEntry { name, value })
            .collect())
    }

    /// `.env` text for the matching secrets. Recorded in the audit log.
    ///
    /// # Errors
    /// Locked vault or a storage error.
    pub fn export_env(&self, vault_id: &str, filter: &EnvFilter) -> Result<String, CoreError> {
        let entries = self.resolve_env(vault_id, filter)?;
        let (vault, _) = self.open(vault_id)?;
        self.ctx.record(
            &vault,
            AuditKind::Exported,
            None,
            Some(&format!("{} variables", entries.len())),
        );
        Ok(format_env(&entries))
    }
}

fn new_env_payload(entry: &EnvEntry, environment: Option<String>, now: i64) -> SecretPayload {
    SecretPayload {
        name: entry.name.clone(),
        kind: SecretKind::ApiKey,
        environment,
        tags: Vec::new(),
        value: SecretValue::new(entry.value.clone()),
        notes: String::new(),
        expires_at: None,
        remind_before_days: None,
        favorite: false,
        created_at: now,
        updated_at: now,
        version: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::mask;

    #[test]
    fn masks_all_but_the_tail() {
        assert_eq!(mask("sk_live_51Hx2b9aQz"), "••••••••9aQz");
        assert_eq!(mask("abc"), "••••");
    }
}
