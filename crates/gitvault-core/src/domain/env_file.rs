//! Parsing and writing of `.env` files.

/// One `NAME=value` pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvEntry {
    pub name: String,
    pub value: String,
}

/// Whether `name` is a valid environment variable name.
#[must_use]
pub fn is_valid_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    let first_ok = chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
    first_ok && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Parses `.env` text. Comments, blank lines and invalid lines are skipped.
#[must_use]
pub fn parse_env(content: &str) -> Vec<EnvEntry> {
    content.lines().filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<EnvEntry> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let line = line.strip_prefix("export ").unwrap_or(line);
    let (name, raw_value) = line.split_once('=')?;
    let name = name.trim();
    if !is_valid_env_name(name) {
        return None;
    }
    Some(EnvEntry {
        name: name.to_owned(),
        value: parse_value(raw_value.trim()),
    })
}

fn parse_value(raw: &str) -> String {
    if let Some(rest) = raw.strip_prefix('"') {
        return unescape_double_quoted(rest);
    }
    if let Some(rest) = raw.strip_prefix('\'') {
        return rest.split('\'').next().unwrap_or_default().to_owned();
    }
    match raw.find(" #") {
        Some(index) => raw[..index].trim_end().to_owned(),
        None => raw.to_owned(),
    }
}

fn unescape_double_quoted(rest: &str) -> String {
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => break,
            '\\' => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            },
            other => out.push(other),
        }
    }
    out
}

/// Writes entries as `.env` text, quoting values only when needed.
#[must_use]
pub fn format_env(entries: &[EnvEntry]) -> String {
    let mut out = String::new();
    for entry in entries {
        out.push_str(&entry.name);
        out.push('=');
        out.push_str(&quote_if_needed(&entry.value));
        out.push('\n');
    }
    out
}

fn quote_if_needed(value: &str) -> String {
    let needs_quotes = value.is_empty()
        || value
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '#' | '"' | '\'' | '=' | '\\' | '$'));
    if !needs_quotes {
        return value.to_owned();
    }
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
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
    fn parses_plain_quoted_and_exported_values() {
        let text = "# comment\nA=1\nexport B=two words # trailing\nC=\"x y\\n\"\nD='raw $value'\n\nbad line\n1X=no\n";
        assert_eq!(
            parse_env(text),
            vec![
                entry("A", "1"),
                entry("B", "two words"),
                entry("C", "x y\n"),
                entry("D", "raw $value")
            ]
        );
    }

    #[test]
    fn keeps_equals_signs_inside_values() {
        assert_eq!(
            parse_env("URL=postgres://u:p@h/db?x=1"),
            vec![entry("URL", "postgres://u:p@h/db?x=1")]
        );
    }

    #[test]
    fn format_round_trips_tricky_values() {
        let entries = vec![
            entry("A", "plain"),
            entry("B", "has space"),
            entry("C", "quo\"te"),
            entry("D", ""),
        ];
        assert_eq!(parse_env(&format_env(&entries)), entries);
    }

    #[test]
    fn validates_names() {
        assert!(is_valid_env_name("STRIPE_KEY"));
        assert!(!is_valid_env_name("1ABC"));
        assert!(!is_valid_env_name("A-B"));
    }
}
