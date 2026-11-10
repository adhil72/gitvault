use anyhow::{anyhow, Result};
use gitvault_core::domain::settings::{Settings, Theme};

use crate::{app::App, cli::SettingsCmd};

pub fn run(app: &App, cmd: &SettingsCmd) -> Result<()> {
    match cmd {
        SettingsCmd::Get => show(app, &app.services.settings.get()),
        SettingsCmd::Set { key, value } => {
            let updated = apply(app.services.settings.get(), key, value)?;
            show(app, &app.services.settings.update(&updated)?)
        }
    }
}

fn show(app: &App, settings: &Settings) -> Result<()> {
    app.emit(settings, || {
        serde_json::to_string_pretty(settings).unwrap_or_default()
    })
}

fn apply(mut settings: Settings, key: &str, value: &str) -> Result<Settings> {
    match key {
        "auto_lock_minutes" => settings.auto_lock_minutes = value.parse()?,
        "clear_clipboard_seconds" => settings.clear_clipboard_seconds = value.parse()?,
        "hide_revealed_seconds" => settings.hide_revealed_seconds = value.parse()?,
        "auto_sync" => settings.auto_sync = value.parse()?,
        "theme" => settings.theme = parse_theme(value)?,
        "github_client_id" => {
            settings.github_client_id = (!value.is_empty()).then(|| value.to_owned())
        }
        other => return Err(anyhow!("unknown setting: {other}")),
    }
    Ok(settings)
}

fn parse_theme(value: &str) -> Result<Theme> {
    serde_json::from_value(serde_json::Value::String(value.to_lowercase()))
        .map_err(|_| anyhow!("theme must be dark, light or system"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_known_keys_and_rejects_unknown_ones() {
        let s = apply(Settings::default(), "auto_lock_minutes", "10").unwrap();
        assert_eq!(s.auto_lock_minutes, 10);
        assert_eq!(apply(s, "theme", "light").unwrap().theme, Theme::Light);
        assert!(apply(Settings::default(), "nope", "1").is_err());
    }
}
