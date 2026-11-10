use anyhow::Result;
use gitvault_core::dto::{GenerateInput, GenerateKind};

use crate::{app::App, cli::GenerateCmd};

fn base(kind: GenerateKind) -> GenerateInput {
    GenerateInput {
        kind,
        length: 20,
        uppercase: true,
        lowercase: true,
        digits: true,
        symbols: true,
        avoid_ambiguous: false,
        words: 5,
        separator: "-".into(),
        capitalize: false,
    }
}

pub fn run(app: &App, cmd: &GenerateCmd) -> Result<()> {
    let input = match cmd {
        GenerateCmd::Password {
            length,
            no_uppercase,
            no_lowercase,
            no_digits,
            no_symbols,
            avoid_ambiguous,
        } => GenerateInput {
            length: *length,
            uppercase: !no_uppercase,
            lowercase: !no_lowercase,
            digits: !no_digits,
            symbols: !no_symbols,
            avoid_ambiguous: *avoid_ambiguous,
            ..base(GenerateKind::Password)
        },
        GenerateCmd::Passphrase {
            words,
            separator,
            capitalize,
        } => GenerateInput {
            words: *words,
            separator: separator.clone(),
            capitalize: *capitalize,
            ..base(GenerateKind::Passphrase)
        },
        GenerateCmd::Pin { length } => GenerateInput {
            length: *length,
            ..base(GenerateKind::Pin)
        },
    };
    let generated = app.services.tools.generate(&input)?;
    app.emit(&generated, || generated.value.clone())
}
