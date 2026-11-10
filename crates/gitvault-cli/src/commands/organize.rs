//! `env list`, `tag list`: counts derived from the secrets.

use anyhow::Result;
use gitvault_core::dto::FacetCount;

use crate::{app::App, output::table};

#[derive(Clone, Copy)]
pub enum Facet {
    Environment,
    Tag,
}

pub fn list(app: &App, facet: Facet) -> Result<()> {
    let facets = app.services.secrets.facets(&app.vault_id()?)?;
    let (title, counts): (&str, Vec<FacetCount>) = match facet {
        Facet::Environment => ("ENVIRONMENT", facets.environments),
        Facet::Tag => ("TAG", facets.tags),
    };
    app.emit(&counts, || {
        let rows: Vec<Vec<String>> = counts
            .iter()
            .map(|c| vec![c.name.clone(), c.count.to_string()])
            .collect();
        table(&[title, "SECRETS"], &rows)
    })
}
