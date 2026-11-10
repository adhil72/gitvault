use std::{fs, io::Write};

use anyhow::Result;
use gitvault_core::dto::{EnvFilter, ImportEnvInput};

use crate::{
    app::App,
    cli::{ExportCmd, FilterArgs, ImportArgs, ImportCmd},
    output::table,
};

pub fn import(app: &App, cmd: &ImportCmd) -> Result<()> {
    let ImportCmd::Env(args) = cmd;
    let input = ImportEnvInput {
        vault_id: app.vault_id()?,
        content: fs::read_to_string(&args.file)?,
        environment: args.environment.clone(),
        overwrite: args.overwrite,
    };
    if args.preview {
        return preview(app, &input);
    }
    run_import(app, args, &input)
}

fn preview(app: &App, input: &ImportEnvInput) -> Result<()> {
    let entries = app.services.secrets.preview_env(input)?;
    app.emit(&entries, || {
        let rows: Vec<Vec<String>> = entries
            .iter()
            .map(|e| {
                vec![
                    e.name.clone(),
                    e.masked_value.clone(),
                    format!("{:?}", e.status),
                ]
            })
            .collect();
        table(&["NAME", "VALUE", "STATUS"], &rows)
    })
}

fn run_import(app: &App, args: &ImportArgs, input: &ImportEnvInput) -> Result<()> {
    let report = app.services.secrets.import_env(input)?;
    app.emit(&report, || {
        format!(
            "Imported {}: {} created, {} updated, {} skipped.",
            args.file.display(),
            report.created,
            report.updated,
            report.skipped
        )
    })
}

pub fn export(app: &App, cmd: &ExportCmd) -> Result<()> {
    let ExportCmd::Env { filter, output } = cmd;
    let text = app
        .services
        .secrets
        .export_env(&app.vault_id()?, &to_filter(filter))?;
    match output {
        Some(path) => {
            write_private(path, &text)?;
            app.emit(&serde_json::json!({ "written": path }), || {
                format!("Wrote {}.", path.display())
            })
        }
        None => {
            print!("{text}");
            Ok(())
        }
    }
}

pub fn to_filter(filter: &FilterArgs) -> EnvFilter {
    EnvFilter {
        environment: filter.environment.clone(),
        tag: filter.tag.clone(),
    }
}

/// Writes a file readable only by the current user.
pub fn write_private(path: &std::path::Path, text: &str) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options.open(path)?.write_all(text.as_bytes())?;
    Ok(())
}
