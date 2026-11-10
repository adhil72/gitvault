//! Where a trusted device may unlock by itself. Kept on the device (never in the vault repo).
//!
//! A guardrail, not a cryptographic boundary: it stops accidental use from the wrong project or
//! environment, but code running as the same user could still read the device key.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// One trusted folder (it covers all subfolders), optionally limited to some environments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderRule {
    pub path: PathBuf,
    /// Empty means every environment.
    #[serde(default)]
    pub environments: Vec<String>,
}

/// Where the caller is running and what it wants to load.
#[derive(Debug, Clone, Copy, Default)]
pub struct TrustContext<'a> {
    /// The current directory (already resolved: no symlinks or `..`).
    pub folder: Option<&'a Path>,
    pub environment: Option<&'a str>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustPolicy {
    /// When false the device unlocks from anywhere (plain "trust this device").
    #[serde(default)]
    pub restricted: bool,
    #[serde(default)]
    pub folders: Vec<FolderRule>,
}

impl TrustPolicy {
    /// Whether a silent unlock is allowed in this context.
    #[must_use]
    pub fn allows(&self, context: &TrustContext<'_>) -> bool {
        if !self.restricted {
            return true;
        }
        let Some(folder) = context.folder else {
            return false;
        };
        self.folders.iter().any(|rule| {
            folder.starts_with(&rule.path)
                && (rule.environments.is_empty()
                    || context.environment.is_some_and(|wanted| {
                        rule.environments
                            .iter()
                            .any(|allowed| allowed.eq_ignore_ascii_case(wanted))
                    }))
        })
    }

    /// Adds (or replaces) a rule and restricts the device to the listed folders.
    pub fn add(&mut self, rule: FolderRule) {
        self.folders.retain(|existing| existing.path != rule.path);
        self.folders.push(rule);
        self.restricted = true;
    }

    /// Removes a rule. Returns whether it existed. The device stays restricted.
    pub fn remove(&mut self, path: &Path) -> bool {
        let before = self.folders.len();
        self.folders.retain(|rule| rule.path != path);
        self.folders.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(environments: &[&str]) -> TrustPolicy {
        let mut policy = TrustPolicy::default();
        policy.add(FolderRule {
            path: PathBuf::from("/work/shop"),
            environments: environments.iter().map(|e| (*e).to_owned()).collect(),
        });
        policy
    }

    fn at<'a>(folder: &'a str, environment: Option<&'a str>) -> TrustContext<'a> {
        TrustContext {
            folder: Some(Path::new(folder)),
            environment,
        }
    }

    #[test]
    fn an_unrestricted_device_unlocks_anywhere() {
        assert!(TrustPolicy::default().allows(&TrustContext::default()));
    }

    #[test]
    fn covers_the_folder_and_its_subfolders_only() {
        let policy = policy(&[]);
        assert!(policy.allows(&at("/work/shop", None)));
        assert!(policy.allows(&at("/work/shop/api/src", Some("production"))));
        assert!(
            !policy.allows(&at("/work/shop-other", None)),
            "a sibling with the same prefix"
        );
        assert!(!policy.allows(&at("/work", None)));
        assert!(
            !policy.allows(&TrustContext::default()),
            "no folder means no silent unlock"
        );
    }

    #[test]
    fn can_be_limited_to_environments() {
        let policy = policy(&["development"]);
        assert!(policy.allows(&at("/work/shop/api", Some("Development"))));
        assert!(!policy.allows(&at("/work/shop/api", Some("production"))));
        assert!(!policy.allows(&at("/work/shop/api", None)));
    }

    #[test]
    fn removing_the_last_rule_keeps_the_device_restricted() {
        let mut policy = policy(&[]);
        assert!(policy.remove(Path::new("/work/shop")));
        assert!(!policy.allows(&at("/work/shop", None)));
    }
}
