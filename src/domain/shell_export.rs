use std::collections::BTreeMap;

use crate::{domain::validate_var_key, error::AppError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellKind {
    Posix,
    Fish,
}

impl ShellKind {
    #[must_use]
    pub fn detect(explicit: Option<&str>) -> Self {
        let name = explicit
            .map(str::to_owned)
            .or_else(|| std::env::var("SHELL").ok())
            .unwrap_or_default();
        if name.to_ascii_lowercase().contains("fish") {
            Self::Fish
        } else {
            Self::Posix
        }
    }
}

fn shell_escape(val: &str, shell: ShellKind) -> String {
    match shell {
        ShellKind::Posix => format!("'{}'", val.replace('\'', "'\\''")),
        ShellKind::Fish => format!("'{}'", val.replace('\\', "\\\\").replace('\'', "\\'")),
    }
}

fn push_export(out: &mut String, key: &str, val: &str, shell: ShellKind) {
    let escaped = shell_escape(val, shell);
    match shell {
        ShellKind::Posix => out.push_str(&format!("export {key}={escaped}\n")),
        ShellKind::Fish => out.push_str(&format!("set -gx {key} {escaped};\n")),
    }
}

/// Formats decrypted secrets as safe shell export statements (never touching disk).
pub fn format_shell_exports(
    project: &str,
    env: &str,
    vars: &BTreeMap<String, String>,
    shell: ShellKind,
) -> Result<String, AppError> {
    let mut out = String::new();
    let mut keys = Vec::with_capacity(vars.len());
    for (k, v) in vars {
        let key = validate_var_key(k)?;
        keys.push(key.clone());
        push_export(&mut out, &key, v, shell);
    }
    push_export(&mut out, "GITVAULT_LOADED_KEYS", &keys.join(" "), shell);
    push_export(&mut out, "GITVAULT_ACTIVE_PROJECT", project, shell);
    push_export(&mut out, "GITVAULT_ACTIVE_ENV", env, shell);
    Ok(out)
}

/// Formats shell `unset` statements for previously loaded variables.
pub fn format_shell_unsets(loaded_keys: &str, shell: ShellKind) -> Result<String, AppError> {
    let mut keys = Vec::new();
    for token in loaded_keys.split_whitespace() {
        keys.push(validate_var_key(token)?);
    }
    keys.push("GITVAULT_LOADED_KEYS".to_owned());
    keys.push("GITVAULT_ACTIVE_PROJECT".to_owned());
    keys.push("GITVAULT_ACTIVE_ENV".to_owned());
    let joined = keys.join(" ");
    Ok(match shell {
        ShellKind::Posix => format!("unset {joined}\n"),
        ShellKind::Fish => format!("set -e {joined};\n"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_exports_escape_quotes_and_specials_safely() {
        let mut vars = BTreeMap::new();
        vars.insert("API_KEY".into(), "a'b$(rm -rf /)".into());
        let posix = format_shell_exports("app", "dev", &vars, ShellKind::Posix).unwrap();
        assert!(posix.contains("export API_KEY='a'\\''b$(rm -rf /)'"));
        assert!(posix.contains("export GITVAULT_LOADED_KEYS='API_KEY'"));

        let fish = format_shell_exports("app", "dev", &vars, ShellKind::Fish).unwrap();
        assert!(fish.contains(r"set -gx API_KEY 'a\'b$(rm -rf /)';"));

        let unsets = format_shell_unsets("API_KEY", ShellKind::Posix).unwrap();
        assert!(unsets.contains("unset API_KEY GITVAULT_LOADED_KEYS"));
    }
}
