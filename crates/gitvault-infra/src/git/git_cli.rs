use std::{
    fs,
    path::{Path, PathBuf},
};

use gitvault_core::{
    domain::ids::VaultId,
    ports::{Author, CommitInfo, ConflictChoice, RepoStatus, SyncReport, VersionControl},
    CoreError,
};

use super::{
    command::{classify, run, run_text, Options},
    parse::{is_safe_revision, parse_log, LOG_FORMAT},
};

/// Every vault is one git repository under `<root>/<vault-id>`.
pub struct GitCli {
    root: PathBuf,
}

/// All files are treated as binary so git never text-merges encrypted JSON.
const GIT_ATTRIBUTES: &str = "* binary\n";

impl GitCli {
    #[must_use]
    pub const fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn dir(&self, id: &VaultId) -> PathBuf {
        self.root.join(id.as_str())
    }

    fn text(&self, id: &VaultId, args: &[&str]) -> Result<String, CoreError> {
        run_text(&self.dir(id), args, &Options::default())
    }

    fn branch(&self, id: &VaultId) -> Result<String, CoreError> {
        self.text(id, &["symbolic-ref", "--short", "HEAD"])
    }

    fn count(&self, id: &VaultId, range: &str) -> u32 {
        self.text(id, &["rev-list", "--count", range])
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(0)
    }

    fn unmerged(&self, id: &VaultId) -> Vec<String> {
        self.text(id, &["diff", "--name-only", "--diff-filter=U"])
            .map(|out| out.lines().map(str::to_owned).collect())
            .unwrap_or_default()
    }

    fn has_commits(&self, id: &VaultId) -> bool {
        self.text(id, &["rev-parse", "--verify", "HEAD"]).is_ok()
    }

    fn remote_ref_exists(&self, id: &VaultId, remote_ref: &str) -> bool {
        self.text(id, &["rev-parse", "--verify", remote_ref])
            .is_ok()
    }
}

impl VersionControl for GitCli {
    fn init(&self, id: &VaultId) -> Result<(), CoreError> {
        let dir = self.dir(id);
        run_text(&dir, &["init", "-b", "main"], &Options::default())?;
        fs::write(dir.join(".gitattributes"), GIT_ATTRIBUTES)
            .map_err(|e| CoreError::Storage(e.to_string()))
    }

    fn set_remote(&self, id: &VaultId, url: &str) -> Result<(), CoreError> {
        let verb = if self.remote_url(id)?.is_some() {
            "set-url"
        } else {
            "add"
        };
        self.text(id, &["remote", verb, "origin", url]).map(|_| ())
    }

    fn remote_url(&self, id: &VaultId) -> Result<Option<String>, CoreError> {
        Ok(self
            .text(id, &["remote", "get-url", "origin"])
            .ok()
            .filter(|u| !u.is_empty()))
    }

    fn clone_repo(&self, id: &VaultId, url: &str, token: Option<&str>) -> Result<(), CoreError> {
        fs::create_dir_all(&self.root).map_err(|e| CoreError::Storage(e.to_string()))?;
        let target = self.dir(id);
        let target_str = target.to_string_lossy().into_owned();
        let output = run(
            &self.root,
            &["clone", "--quiet", "--", url, &target_str],
            &Options {
                token,
                author: None,
            },
        )?;
        if output.status.success() {
            return Ok(());
        }
        let _ = fs::remove_dir_all(&target);
        Err(classify(&String::from_utf8_lossy(&output.stderr)))
    }

    fn commit_all(
        &self,
        id: &VaultId,
        message: &str,
        author: &Author,
    ) -> Result<Option<CommitInfo>, CoreError> {
        let dir = self.dir(id);
        let options = Options {
            token: None,
            author: Some(author),
        };
        run_text(&dir, &["add", "-A"], &options)?;
        if run_text(&dir, &["status", "--porcelain"], &options)?.is_empty() {
            return Ok(None);
        }
        run_text(&dir, &["commit", "--quiet", "-m", message], &options)?;
        let log = run_text(&dir, &["log", "-1", LOG_FORMAT], &options)?;
        Ok(parse_log(&log).into_iter().next())
    }

    fn history(&self, id: &VaultId, path: &str) -> Result<Vec<CommitInfo>, CoreError> {
        if !self.has_commits(id) {
            return Ok(Vec::new());
        }
        let log = self.text(id, &["log", LOG_FORMAT, "--", path])?;
        Ok(parse_log(&log))
    }

    fn read_at(
        &self,
        id: &VaultId,
        commit: &str,
        path: &str,
    ) -> Result<Option<Vec<u8>>, CoreError> {
        if !is_safe_revision(commit) {
            return Err(CoreError::Validation("invalid commit id".into()));
        }
        let spec = format!("{commit}:{path}");
        let output = run(
            &self.dir(id),
            &["cat-file", "-p", &spec],
            &Options::default(),
        )?;
        Ok(output.status.success().then_some(output.stdout))
    }

    fn status(&self, id: &VaultId) -> Result<RepoStatus, CoreError> {
        let has_remote = self.remote_url(id)?.is_some();
        if !has_remote {
            return Ok(RepoStatus::default());
        }
        let branch = self.branch(id)?;
        let upstream = format!("origin/{branch}");
        let pending = if self.remote_ref_exists(id, &upstream) {
            self.count(id, &format!("{upstream}..HEAD"))
        } else {
            self.count(id, "HEAD")
        };
        Ok(RepoStatus {
            has_remote,
            pending,
            conflicts: self.unmerged(id),
        })
    }

    fn sync(
        &self,
        id: &VaultId,
        token: Option<&str>,
        author: &Author,
    ) -> Result<SyncReport, CoreError> {
        if self.remote_url(id)?.is_none() {
            return Ok(SyncReport {
                no_remote: true,
                ..SyncReport::default()
            });
        }
        let existing = self.unmerged(id);
        if !existing.is_empty() {
            return Ok(SyncReport {
                conflicts: existing,
                ..SyncReport::default()
            });
        }
        let options = Options {
            token,
            author: Some(author),
        };
        run_text(&self.dir(id), &["fetch", "--quiet", "origin"], &options)?;
        self.merge_and_push(id, &options)
    }

    fn resolve(
        &self,
        id: &VaultId,
        path: &str,
        choice: ConflictChoice,
        author: &Author,
    ) -> Result<(), CoreError> {
        let options = Options {
            token: None,
            author: Some(author),
        };
        let dir = self.dir(id);
        let side = if choice == ConflictChoice::Local {
            "--ours"
        } else {
            "--theirs"
        };
        let restored = run(&dir, &["checkout", side, "--", path], &options)?;
        let staged = if restored.status.success() {
            run_text(&dir, &["add", "--", path], &options)
        } else {
            run_text(&dir, &["rm", "--quiet", "--", path], &options)
        };
        staged?;
        if self.unmerged(id).is_empty() {
            run_text(&dir, &["commit", "--quiet", "--no-edit"], &options)?;
        }
        Ok(())
    }
}

impl GitCli {
    /// Merges the remote branch. Returns `false` when conflicts remain.
    fn merge(&self, dir: &Path, remote_ref: &str, options: &Options<'_>) -> bool {
        run(dir, &["merge", "--no-edit", remote_ref], options).is_ok_and(|out| out.status.success())
    }
}

impl GitCli {
    /// After a fetch: merge the remote branch (if any), then push.
    fn merge_and_push(&self, id: &VaultId, options: &Options<'_>) -> Result<SyncReport, CoreError> {
        let dir = self.dir(id);
        let branch = self.branch(id)?;
        let remote_ref = format!("origin/{branch}");
        let remote_exists = self.remote_ref_exists(id, &remote_ref);
        let mut report = SyncReport::default();
        if remote_exists {
            report.pulled = self.count(id, &format!("HEAD..{remote_ref}"));
            if report.pulled > 0 && !self.merge(&dir, &remote_ref, options) {
                report.conflicts = self.unmerged(id);
                return Ok(report);
            }
            report.pushed = self.count(id, &format!("{remote_ref}..HEAD"));
        } else {
            report.pushed = self.count(id, "HEAD");
        }
        if report.pushed > 0 || !remote_exists {
            run_text(&dir, &["push", "--quiet", "-u", "origin", &branch], options)
                .map_err(|e| push_error(&e))?;
        }
        Ok(report)
    }
}

fn push_error(error: &CoreError) -> CoreError {
    match error {
        CoreError::Git(message)
            if message.contains("rejected") || message.contains("non-fast-forward") =>
        {
            CoreError::Conflict("GitHub has newer changes; sync again".into())
        }
        CoreError::Git(message) => CoreError::Git(message.clone()),
        CoreError::Network(message) => CoreError::Network(message.clone()),
        _ => CoreError::NotAuthenticated,
    }
}
