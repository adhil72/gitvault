//! Password, passphrase and PIN generator.

use crate::{
    domain::generator::{
        generate_passphrase, generate_password, generate_pin, strength_label, Generated,
        PasswordOptions,
    },
    dto::{GenerateInput, GenerateKind, GeneratedDto},
    error::CoreError,
};

#[derive(Default)]
pub struct ToolsService;

impl ToolsService {
    /// Generates a password, passphrase or PIN.
    ///
    /// # Errors
    /// Validation error for out-of-range options.
    pub fn generate(&self, input: &GenerateInput) -> Result<GeneratedDto, CoreError> {
        let generated = match input.kind {
            GenerateKind::Password => generate_password(&PasswordOptions {
                length: input.length as usize,
                uppercase: input.uppercase,
                lowercase: input.lowercase,
                digits: input.digits,
                symbols: input.symbols,
                avoid_ambiguous: input.avoid_ambiguous,
            })?,
            GenerateKind::Passphrase => {
                generate_passphrase(input.words as usize, &input.separator, input.capitalize)?
            }
            GenerateKind::Pin => generate_pin(input.length as usize)?,
        };
        Ok(to_dto(generated))
    }
}

fn to_dto(generated: Generated) -> GeneratedDto {
    GeneratedDto {
        strength: strength_label(generated.entropy_bits).to_owned(),
        entropy_bits: generated.entropy_bits,
        value: generated.value,
    }
}
