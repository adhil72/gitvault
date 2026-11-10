//! `run` and `inject`: use secrets without ever printing them.

use std::{fs, process::Command};

use anyhow::{anyhow, bail, Result};
use gitvault_core::{domain::env_file::EnvEntry, CoreError};

use crate::{
    app::App,
    cli::{InjectArgs, RunArgs},
    commands::transfer::{to_filter, write_private},
};

const REDACTED: &str = "[REDACTED]";
const MIN_REDACT_LEN: usize = 4;

/// Runs a command with the matching secrets as environment variables.
/// Returns the child's exit code.
pub fn run(app: &App, args: &RunArgs) -> Result<u8> {
    let entries = app.services.secrets.resolve_env(
        &app.ensure_unlocked(args.filter.environment.as_deref())?,
        &to_filter(&args.filter),
    )?;
    let (program, rest) = args
        .command
        .split_first()
        .ok_or_else(|| anyhow!("no command given"))?;
    let mut command = Command::new(program);
    command
        .args(rest)
        .envs(entries.iter().map(|e| (&e.name, &e.value)));
    if args.redact {
        return run_redacted(command, &entries);
    }
    let status = command.status()?;
    Ok(exit_code(status.code()))
}

fn run_redacted(mut command: Command, entries: &[EnvEntry]) -> Result<u8> {
    let output = command.output()?;
    print!(
        "{}",
        redact(&String::from_utf8_lossy(&output.stdout), entries)
    );
    eprint!(
        "{}",
        redact(&String::from_utf8_lossy(&output.stderr), entries)
    );
    Ok(exit_code(output.status.code()))
}

fn exit_code(code: Option<i32>) -> u8 {
    code.and_then(|c| u8::try_from(c).ok()).unwrap_or(1)
}

/// Replaces every secret value in `text` (values shorter than 4 characters are left alone).
#[must_use]
pub fn redact(text: &str, entries: &[EnvEntry]) -> String {
    let mut values: Vec<&str> = entries
        .iter()
        .map(|e| e.value.as_str())
        .filter(|v| v.chars().count() >= MIN_REDACT_LEN)
        .collect();
    values.sort_by_key(|v| std::cmp::Reverse(v.len()));
    values
        .iter()
        .fold(text.to_owned(), |acc, value| acc.replace(value, REDACTED))
}

/// Fills `{{ NAME }}` / `{{ gitvault://NAME }}` placeholders and writes the result (0600).
pub fn inject(app: &App, args: &InjectArgs) -> Result<()> {
    let entries = app.services.secrets.resolve_env(
        &app.ensure_unlocked(args.filter.environment.as_deref())?,
        &to_filter(&args.filter),
    )?;
    let template = fs::read_to_string(&args.input)?;
    let rendered = render(&template, &entries)?;
    write_private(&args.output, &rendered)?;
    app.emit(&serde_json::json!({ "written": args.output }), || {
        format!("Wrote {}.", args.output.display())
    })
}

/// Replaces placeholders. Unknown names are an error (never silently empty).
pub fn render(template: &str, entries: &[EnvEntry]) -> Result<String> {
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            bail!("unterminated {{{{ placeholder")
        };
        let name = after[..end].trim().trim_start_matches("gitvault://");
        let entry = entries.iter().find(|e| e.name == name).ok_or_else(|| {
            anyhow!(CoreError::NotFound { entity: "secret" })
                .context(format!("no secret named {name}"))
        })?;
        out.push_str(&entry.value);
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, value: &str) -> EnvEntry {
        EnvEntry {
            name: name.into(),
            value: value.into(),
        }
    }

    #[test]
    fn redacts_longest_values_first_and_ignores_short_ones() {
        let entries = [
            entry("A", "secret"),
            entry("B", "secret-longer"),
            entry("C", "ab"),
        ];
        assert_eq!(
            redact("x secret-longer y secret ab", &entries),
            "x [REDACTED] y [REDACTED] ab"
        );
    }

    #[test]
    fn renders_placeholders() {
        let entries = [entry("DB_URL", "postgres://x")];
        assert_eq!(
            render("url={{ gitvault://DB_URL }} ok={{DB_URL}}", &entries).unwrap(),
            "url=postgres://x ok=postgres://x"
        );
        assert!(render("{{ MISSING }}", &entries).is_err());
        assert!(render("{{ DB_URL", &entries).is_err());
    }
}
