use anyhow::{anyhow, Result};
use gitvault_core::{
    domain::secret::SecretKind,
    dto::{
        AddSecretInput, ListSecretsQuery, SecretDetail, SecretSort, SecretSummary,
        UpdateSecretInput,
    },
    CoreError,
};

use crate::{
    app::App,
    cli::{AddArgs, FilterArgs, GetArgs, ListArgs, SecretCmd, SortArg, UpdateArgs},
    input,
    output::table,
    time,
};

pub fn run(app: &App, cmd: &SecretCmd) -> Result<()> {
    match cmd {
        SecretCmd::Add(args) => add(app, args),
        SecretCmd::Get(args) => get(app, args),
        SecretCmd::List(args) => list(app, args),
        SecretCmd::Update(args) => update(app, args),
        SecretCmd::Rotate { secret, value } => rotate(app, secret, value),
        SecretCmd::Delete { secret } => delete(app, secret),
        SecretCmd::History { secret } => history(app, secret),
        SecretCmd::Restore { secret, commit } => restore(app, secret, commit),
        SecretCmd::Search { text } => search(app, text),
    }
}

fn parse_kind(text: &str) -> Result<SecretKind> {
    SecretKind::parse(text).ok_or_else(|| {
        anyhow!(CoreError::Validation(
            "kind must be one of: api_key, password, certificate, note, env_file".into()
        ))
    })
}

fn parse_expiry(text: &str) -> Result<Option<i64>> {
    if text.trim().is_empty() {
        return Ok(None);
    }
    time::parse_date(text).map(Some)
}

fn add(app: &App, args: &AddArgs) -> Result<()> {
    let value = input::secret_value(app, &args.value, &format!("Value for {}: ", args.name))?;
    let detail = app.services.secrets.add(&AddSecretInput {
        vault_id: app.vault_id()?,
        name: args.name.clone(),
        kind: parse_kind(&args.kind)?,
        value,
        environment: args.filter.environment.clone(),
        tags: args.filter.tag.iter().cloned().collect(),
        notes: args.notes.clone(),
        expires_at: args
            .expires
            .as_deref()
            .map(parse_expiry)
            .transpose()?
            .flatten(),
        remind_before_days: args.remind_days,
    })?;
    app.emit(&detail, || {
        format!("Added {} ({}).", detail.summary.name, detail.summary.id)
    })
}

fn get(app: &App, args: &GetArgs) -> Result<()> {
    let found = app.resolve_secret(&args.secret, args.environment.as_deref())?;
    let vault = app.vault_id()?;
    let reveal = args.reveal || args.raw;
    let value = if reveal {
        Some(app.services.secrets.reveal(&vault, &found.id)?.value)
    } else {
        None
    };
    if args.raw {
        println!("{}", value.unwrap_or_default());
        return Ok(());
    }
    let detail = app.services.secrets.get(&vault, &found.id)?;
    let view = serde_json::json!({ "secret": detail, "value": value });
    app.emit(&view, || describe(&detail, value.as_deref()))
}

fn describe(detail: &SecretDetail, value: Option<&str>) -> String {
    let s = &detail.summary;
    let mut lines = vec![
        format!("Name:         {}", s.name),
        format!("Id:           {}", s.id),
        format!("Kind:         {:?}", s.kind),
        format!("Environment:  {}", s.environment.as_deref().unwrap_or("-")),
        format!(
            "Tags:         {}",
            if s.tags.is_empty() {
                "-".into()
            } else {
                s.tags.join(", ")
            }
        ),
        format!("Version:      {}", s.version),
        format!("Updated:      {}", time::format_time(s.updated_at)),
        format!(
            "Expires:      {}",
            s.expires_at.map_or("-".into(), time::format_date)
        ),
    ];
    if !detail.notes.is_empty() {
        lines.push(format!("Notes:        {}", detail.notes));
    }
    lines.push(format!(
        "Value:        {}",
        value.unwrap_or("•••••••• (use --reveal)")
    ));
    lines.join("\n")
}

fn filter_query(vault_id: String, filter: &FilterArgs) -> ListSecretsQuery {
    ListSecretsQuery {
        vault_id,
        search: None,
        environment: filter.environment.clone(),
        tag: filter.tag.clone(),
        kind: None,
        favorites_only: false,
        expiring_within_days: None,
        sort: None,
    }
}

fn list(app: &App, args: &ListArgs) -> Result<()> {
    let mut query = filter_query(app.vault_id()?, &args.filter);
    query.search = args.search.clone();
    query.kind = args.kind.as_deref().map(parse_kind).transpose()?;
    query.favorites_only = args.favorites;
    query.expiring_within_days = args.expiring_days;
    query.sort = args.sort.map(|s| match s {
        SortArg::Updated => SecretSort::Updated,
        SortArg::Name => SecretSort::Name,
        SortArg::Expiry => SecretSort::Expiry,
    });
    show_list(app, &app.services.secrets.list(&query)?)
}

fn show_list(app: &App, items: &[SecretSummary]) -> Result<()> {
    app.emit(&items, || {
        let rows: Vec<Vec<String>> = items
            .iter()
            .map(|s| {
                vec![
                    s.id.clone(),
                    s.name.clone(),
                    s.environment.clone().unwrap_or_else(|| "-".into()),
                    time::format_date(s.updated_at),
                    s.expires_in_days.map_or("-".into(), |d| format!("{d}d")),
                ]
            })
            .collect();
        table(&["ID", "NAME", "ENVIRONMENT", "UPDATED", "EXPIRES"], &rows)
    })
}

fn search(app: &App, text: &str) -> Result<()> {
    let mut query = filter_query(
        app.vault_id()?,
        &FilterArgs {
            environment: None,
            tag: None,
        },
    );
    query.search = Some(text.to_owned());
    show_list(app, &app.services.secrets.list(&query)?)
}

#[allow(clippy::too_many_lines)] // maps every optional flag onto the current values
fn update(app: &App, args: &UpdateArgs) -> Result<()> {
    let found = app.resolve_secret(&args.secret, None)?;
    let vault = app.vault_id()?;
    let current = app.services.secrets.get(&vault, &found.id)?;
    let s = &current.summary;
    let wants_value = args.value.value_stdin || args.value.from_file.is_some();
    let value = if wants_value {
        Some(input::secret_value(app, &args.value, "New value: ")?)
    } else {
        None
    };
    let updated = app.services.secrets.update(&UpdateSecretInput {
        vault_id: vault,
        secret_id: s.id.clone(),
        name: args.name.clone().unwrap_or_else(|| s.name.clone()),
        kind: args
            .kind
            .as_deref()
            .map(parse_kind)
            .transpose()?
            .unwrap_or(s.kind),
        value,
        environment: args.environment.clone().or_else(|| s.environment.clone()),
        tags: if args.tags.is_empty() {
            s.tags.clone()
        } else {
            args.tags.clone()
        },
        notes: args.notes.clone().or_else(|| Some(current.notes.clone())),
        expires_at: match &args.expires {
            Some(text) => parse_expiry(text)?,
            None => s.expires_at,
        },
        remind_before_days: current.remind_before_days,
        favorite: args.favorite.unwrap_or(s.favorite),
    })?;
    app.emit(&updated, || {
        format!(
            "Updated {} (version {}).",
            updated.summary.name, updated.summary.version
        )
    })
}

fn rotate(app: &App, secret: &str, value: &crate::cli::ValueSource) -> Result<()> {
    let found = app.resolve_secret(secret, None)?;
    let new_value = input::secret_value(app, value, "New value: ")?;
    let detail = app
        .services
        .secrets
        .rotate(&app.vault_id()?, &found.id, &new_value)?;
    app.emit(&detail, || {
        format!(
            "Rotated {} (version {}).",
            detail.summary.name, detail.summary.version
        )
    })
}

fn delete(app: &App, secret: &str) -> Result<()> {
    let found = app.resolve_secret(secret, None)?;
    app.services.secrets.delete(&app.vault_id()?, &found.id)?;
    app.emit(&serde_json::json!({ "deleted": found.id }), || {
        format!(
            "Deleted {}. Restore it with `gitvault secret history {}`.",
            found.name, found.id
        )
    })
}

fn history(app: &App, secret: &str) -> Result<()> {
    let id = history_target(app, secret)?;
    let entries = app.services.secrets.history(&app.vault_id()?, &id)?;
    app.emit(&entries, || {
        let rows: Vec<Vec<String>> = entries
            .iter()
            .map(|e| {
                vec![
                    format!("v{}", e.version),
                    e.commit.chars().take(10).collect(),
                    time::format_time(e.at),
                    e.author.clone(),
                    e.message.clone(),
                ]
            })
            .collect();
        table(&["VERSION", "COMMIT", "WHEN", "AUTHOR", "CHANGE"], &rows)
    })
}

/// Deleted secrets cannot be found by name, so ids are accepted as they are.
fn history_target(app: &App, secret: &str) -> Result<String> {
    app.resolve_secret(secret, None)
        .map(|s| s.id)
        .or_else(|_| Ok(secret.to_owned()))
}

fn restore(app: &App, secret: &str, commit: &str) -> Result<()> {
    let id = history_target(app, secret)?;
    let detail = app
        .services
        .secrets
        .restore(&app.vault_id()?, &id, commit)?;
    app.emit(&detail, || {
        format!(
            "Restored {} as version {}.",
            detail.summary.name, detail.summary.version
        )
    })
}
