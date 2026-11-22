use std::collections::BTreeMap;

use crate::{domain::validate_var_key, error::AppError};

/// Parses a legacy `.env` file into validated `KEY -> VALUE` pairs for migration into GitVault.
pub fn parse_dotenv(content: &str) -> Result<BTreeMap<String, String>, AppError> {
    let mut vars = BTreeMap::new();
    for (idx, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let body = trimmed
            .strip_prefix("export")
            .filter(|rest| rest.starts_with(char::is_whitespace))
            .map_or(trimmed, str::trim_start);
        let Some((raw_key, raw_val)) = body.split_once('=') else {
            return Err(AppError::Validation(format!(
                "invalid .env syntax at line {}: missing '='",
                idx + 1
            )));
        };
        let key = validate_var_key(raw_key)?;
        vars.insert(key, parse_dotenv_value(raw_val.trim_start()));
    }
    Ok(vars)
}

fn parse_dotenv_value(raw: &str) -> String {
    if let Some(rest) = raw.strip_prefix('"') {
        if let Some(end) = find_closing_quote(rest, '"') {
            return unescape_double_quoted(&rest[..end]);
        }
    }
    if let Some(rest) = raw.strip_prefix('\'') {
        if let Some(end) = rest.find('\'') {
            return rest[..end].to_owned();
        }
    }
    strip_inline_comment(raw).trim_end().to_owned()
}

fn find_closing_quote(s: &str, quote: char) -> Option<usize> {
    let mut escaped = false;
    for (idx, ch) in s.char_indices() {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == quote {
            return Some(idx);
        }
    }
    None
}

fn unescape_double_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        push_escape(&mut out, chars.next());
    }
    out
}

fn push_escape(out: &mut String, next: Option<char>) {
    match next {
        Some('n') => out.push('\n'),
        Some('r') => out.push('\r'),
        Some('t') => out.push('\t'),
        Some('"') => out.push('"'),
        Some('\\') => out.push('\\'),
        Some(other) => {
            out.push('\\');
            out.push(other);
        }
        None => out.push('\\'),
    }
}

fn strip_inline_comment(raw: &str) -> &str {
    let mut prev_ws = false;
    for (idx, ch) in raw.char_indices() {
        if ch == '#' && prev_ws {
            return &raw[..idx];
        }
        prev_ws = ch.is_whitespace();
    }
    raw
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dotenv_safely() {
        let input = r#"
# Comment
PORT=8080 # inline comment
export DATABASE_URL="postgres://user:p@ss/db?opt=1&x=2"
PRIVATE_KEY='line1\nraw'
ESCAPED="hello\nworld\"!"
"#;
        let parsed = parse_dotenv(input).unwrap();
        assert_eq!(parsed.get("PORT").unwrap(), "8080");
        assert_eq!(
            parsed.get("DATABASE_URL").unwrap(),
            "postgres://user:p@ss/db?opt=1&x=2"
        );
        assert_eq!(parsed.get("PRIVATE_KEY").unwrap(), "line1\\nraw");
        assert_eq!(parsed.get("ESCAPED").unwrap(), "hello\nworld\"!");
    }
}
