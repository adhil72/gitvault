//! Parsing of `git log` output.

use gitvault_core::ports::CommitInfo;

pub const LOG_FORMAT: &str = "--format=%H%x1f%s%x1f%an%x1f%at%x1e";

/// Parses output produced with [`LOG_FORMAT`].
pub fn parse_log(output: &str) -> Vec<CommitInfo> {
    output
        .split('\u{1e}')
        .filter_map(|record| {
            let mut fields = record.trim().split('\u{1f}');
            Some(CommitInfo {
                sha: fields.next()?.to_owned(),
                message: fields.next()?.to_owned(),
                author: fields.next()?.to_owned(),
                at: fields.next()?.parse().ok()?,
            })
        })
        .collect()
}

/// Accepts only hex commit ids or `HEAD` (blocks option injection into git).
pub fn is_safe_revision(text: &str) -> bool {
    text == "HEAD"
        || (text.len() >= 4 && text.len() <= 64 && text.chars().all(|c| c.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_records() {
        let out = "abc123\u{1f}secret: add 1\u{1f}octocat\u{1f}1700000000\u{1e}\ndef456\u{1f}vault: create\u{1f}octocat\u{1f}1690000000\u{1e}";
        let commits = parse_log(out);
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].sha, "abc123");
        assert_eq!(commits[1].at, 1_690_000_000);
    }

    #[test]
    fn revision_safety() {
        assert!(is_safe_revision("HEAD"));
        assert!(is_safe_revision("a1b2c3d4"));
        assert!(!is_safe_revision("--output=/tmp/x"));
        assert!(!is_safe_revision("abc"));
    }
}
