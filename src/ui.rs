use std::io::{self, BufRead, IsTerminal, Write};

use zeroize::Zeroizing;

use crate::error::AppError;

fn use_color() -> bool {
    io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

fn paint(code: &str, text: &str) -> String {
    if use_color() {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_owned()
    }
}

pub fn header(text: &str) {
    eprintln!("\n{}", paint("1;36", text));
}

pub fn info(text: &str) {
    eprintln!("{} {text}", paint("36", "◆"));
}

pub fn success(text: &str) {
    eprintln!("{} {text}", paint("32", "✓"));
}

pub fn warn(text: &str) {
    eprintln!("{} {text}", paint("33", "!"));
}

pub fn dim(text: &str) -> String {
    paint("2", text)
}

pub fn bold(text: &str) -> String {
    paint("1", text)
}

fn read_line_stderr(prompt: &str) -> Result<String, AppError> {
    eprint!("{prompt}");
    io::stderr().flush()?;
    let mut line = String::new();
    let bytes = io::stdin().lock().read_line(&mut line)?;
    if bytes == 0 {
        return Err(AppError::Validation(
            "unexpected end of input while reading prompt".into(),
        ));
    }
    Ok(line.trim().to_owned())
}

/// Interactive numbered selector printed to stderr. Press Enter for `default_idx`.
pub fn select(title: &str, options: &[String], default_idx: usize) -> Result<usize, AppError> {
    if options.is_empty() {
        return Err(AppError::Validation(
            "no options available to choose".into(),
        ));
    }
    let default_idx = default_idx.min(options.len() - 1);
    eprintln!("\n{}", bold(title));
    for (i, opt) in options.iter().enumerate() {
        let marker = if i == default_idx {
            paint("36", "›")
        } else {
            " ".to_owned()
        };
        eprintln!("  {marker} {}) {opt}", i + 1);
    }
    loop {
        let prompt = format!("Choose 1-{} [{}]: ", options.len(), default_idx + 1);
        let answer = read_line_stderr(&prompt)?;
        if answer.is_empty() {
            return Ok(default_idx);
        }
        if let Ok(idx) = answer.parse::<usize>() {
            if (1..=options.len()).contains(&idx) {
                return Ok(idx - 1);
            }
        }
        warn(&format!("Enter a number between 1 and {}.", options.len()));
    }
}

/// Interactive line prompt with optional default value.
pub fn input(prompt: &str, default: Option<&str>) -> Result<String, AppError> {
    loop {
        let formatted = match default {
            Some(d) => format!("{} [{}]: ", bold(prompt), dim(d)),
            None => format!("{}: ", bold(prompt)),
        };
        let answer = read_line_stderr(&formatted)?;
        if answer.is_empty() {
            if let Some(d) = default {
                return Ok(d.to_owned());
            }
            warn("Value cannot be empty.");
            continue;
        }
        return Ok(answer);
    }
}

/// Interactive yes/no confirmation prompt.
pub fn confirm(prompt: &str, default_yes: bool) -> Result<bool, AppError> {
    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    let answer = read_line_stderr(&format!("{} {suffix}: ", bold(prompt)))?.to_lowercase();
    if answer.is_empty() {
        return Ok(default_yes);
    }
    Ok(matches!(answer.as_str(), "y" | "yes"))
}

/// Reads a secret/password without echoing on a terminal (falls back to stdin when piped).
pub fn password(prompt: &str) -> Result<Zeroizing<String>, AppError> {
    eprint!("{}: ", bold(prompt));
    io::stderr().flush()?;
    if io::stdin().is_terminal() {
        let pass = rpassword::read_password()?;
        Ok(Zeroizing::new(
            pass.trim_end_matches(['\r', '\n']).to_owned(),
        ))
    } else {
        read_stdin_secret()
    }
}

/// Reads a single password/value line from stdin (for `--password-stdin` / `--value-stdin`).
pub fn read_stdin_secret() -> Result<Zeroizing<String>, AppError> {
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(Zeroizing::new(
        line.trim_end_matches(['\r', '\n']).to_owned(),
    ))
}
