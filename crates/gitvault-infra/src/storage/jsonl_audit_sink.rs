use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use gitvault_core::{domain::audit::AuditEvent, ports::AuditSink, CoreError};

use super::io::{read_optional, storage_error};

/// Append-only audit log, one JSON object per line.
pub struct JsonlAuditSink {
    path: PathBuf,
}

impl JsonlAuditSink {
    #[must_use]
    pub const fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl AuditSink for JsonlAuditSink {
    fn append(&self, event: &AuditEvent) -> Result<(), CoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(storage_error)?;
        }
        let mut line = serde_json::to_string(event).map_err(storage_error)?;
        line.push('\n');
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(storage_error)?;
        file.write_all(line.as_bytes()).map_err(storage_error)
    }

    fn recent(&self, limit: usize) -> Result<Vec<AuditEvent>, CoreError> {
        let Some(bytes) = read_optional(&self.path)? else {
            return Ok(Vec::new());
        };
        let text = String::from_utf8_lossy(&bytes);
        Ok(text
            .lines()
            .rev()
            .filter_map(|line| serde_json::from_str(line).ok())
            .take(limit)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use gitvault_core::domain::audit::AuditKind;

    use super::*;

    fn event(at: i64) -> AuditEvent {
        AuditEvent {
            at,
            kind: AuditKind::Viewed,
            vault_id: "v".into(),
            secret_id: None,
            secret_name: None,
            device: "test".into(),
            detail: None,
        }
    }

    #[test]
    fn returns_newest_first_and_respects_limit() {
        let dir = tempfile::tempdir().unwrap();
        let sink = JsonlAuditSink::new(dir.path().join("audit.jsonl"));
        for at in 1..=5 {
            sink.append(&event(at)).unwrap();
        }
        let recent = sink.recent(3).unwrap();
        assert_eq!(
            recent.iter().map(|e| e.at).collect::<Vec<_>>(),
            vec![5, 4, 3]
        );
    }

    #[test]
    fn missing_file_means_empty_log() {
        let dir = tempfile::tempdir().unwrap();
        assert!(JsonlAuditSink::new(dir.path().join("none"))
            .recent(10)
            .unwrap()
            .is_empty());
    }
}
