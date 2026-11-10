//! Reading passwords and secret values without ever taking them as arguments.

use std::{
    fs,
    io::{self, Read},
};

use anyhow::{bail, Result};

use crate::{app::App, cli::ValueSource};

fn read_stdin_all() -> Result<String> {
    let mut text = String::new();
    io::stdin().read_to_string(&mut text)?;
    Ok(text)
}

fn first_line(text: &str) -> String {
    text.lines()
        .next()
        .unwrap_or_default()
        .trim_end()
        .to_owned()
}

/// A password from stdin (`--password-stdin`) or a hidden prompt.
pub fn password(app: &App, prompt: &str, from_stdin: bool) -> Result<String> {
    if from_stdin {
        return Ok(first_line(&read_stdin_all()?));
    }
    if !app.interactive {
        bail!("no terminal to ask for the password: pass --password-stdin");
    }
    Ok(rpassword::prompt_password(prompt)?)
}

/// A new password, typed twice when interactive.
pub fn new_password(app: &App, from_stdin: bool) -> Result<String> {
    let first = password(app, "Master password: ", from_stdin)?;
    if from_stdin {
        return Ok(first);
    }
    if password(app, "Repeat master password: ", false)? != first {
        bail!("the passwords do not match");
    }
    Ok(first)
}

/// One line from stdin (used for recovery keys and tokens).
pub fn stdin_line() -> Result<String> {
    Ok(first_line(&read_stdin_all()?))
}

/// All of stdin, trimmed (used for tokens).
pub fn stdin_token() -> Result<String> {
    Ok(read_stdin_all()?.trim().to_owned())
}

/// A secret value from stdin, a file, or a hidden prompt.
pub fn secret_value(app: &App, source: &ValueSource, prompt: &str) -> Result<String> {
    if source.value_stdin {
        return Ok(read_stdin_all()?.trim_end_matches(['\n', '\r']).to_owned());
    }
    if let Some(path) = &source.from_file {
        return Ok(fs::read_to_string(path)?
            .trim_end_matches(['\n', '\r'])
            .to_owned());
    }
    if !app.interactive {
        bail!("no terminal to ask for the value: pass --value-stdin or --from-file");
    }
    Ok(rpassword::prompt_password(prompt)?)
}
