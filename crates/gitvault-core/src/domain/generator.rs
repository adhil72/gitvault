//! Password, passphrase and PIN generation.

use gitvault_crypto::random;

use crate::{domain::wordlist::WORDS, error::CoreError};

const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()-_=+[]{};:,.?";
const AMBIGUOUS: &str = "0O1lI|";

/// Options for [`generate_password`].
#[derive(Debug, Clone)]
pub struct PasswordOptions {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub digits: bool,
    pub symbols: bool,
    pub avoid_ambiguous: bool,
}

impl Default for PasswordOptions {
    fn default() -> Self {
        Self {
            length: 20,
            uppercase: true,
            lowercase: true,
            digits: true,
            symbols: true,
            avoid_ambiguous: false,
        }
    }
}

/// A generated value with an entropy estimate.
#[derive(Debug, Clone)]
pub struct Generated {
    pub value: String,
    pub entropy_bits: f64,
}

/// Generates a random password.
///
/// # Errors
/// Fails when no character class is enabled or the length is out of range (8..=128).
pub fn generate_password(options: &PasswordOptions) -> Result<Generated, CoreError> {
    if !(8..=128).contains(&options.length) {
        return Err(CoreError::Validation(
            "password length must be between 8 and 128".into(),
        ));
    }
    let groups = enabled_groups(options);
    if groups.is_empty() {
        return Err(CoreError::Validation(
            "enable at least one character type".into(),
        ));
    }
    let all: Vec<char> = groups.iter().flatten().copied().collect();
    let mut chars: Vec<char> = groups.iter().map(|group| pick(group)).collect();
    while chars.len() < options.length {
        chars.push(pick(&all));
    }
    shuffle(&mut chars);
    #[allow(clippy::cast_precision_loss)]
    let entropy_bits = options.length as f64 * (all.len() as f64).log2();
    Ok(Generated {
        value: chars.into_iter().collect(),
        entropy_bits,
    })
}

fn enabled_groups(options: &PasswordOptions) -> Vec<Vec<char>> {
    let wanted = [
        (options.lowercase, LOWER),
        (options.uppercase, UPPER),
        (options.digits, DIGITS),
        (options.symbols, SYMBOLS),
    ];
    wanted
        .iter()
        .filter(|(enabled, _)| *enabled)
        .map(|(_, set)| {
            set.chars()
                .filter(|c| !(options.avoid_ambiguous && AMBIGUOUS.contains(*c)))
                .collect()
        })
        .collect()
}

fn pick(set: &[char]) -> char {
    let upper = u32::try_from(set.len()).unwrap_or(1);
    set.get(random::below(upper) as usize)
        .copied()
        .unwrap_or('a')
}

fn shuffle(chars: &mut [char]) {
    for i in (1..chars.len()).rev() {
        let bound = u32::try_from(i + 1).unwrap_or(1);
        chars.swap(i, random::below(bound) as usize);
    }
}

/// Generates a passphrase of `words` random words (3..=12).
///
/// # Errors
/// Fails when the word count is out of range.
pub fn generate_passphrase(
    words: usize,
    separator: &str,
    capitalize: bool,
) -> Result<Generated, CoreError> {
    if !(3..=12).contains(&words) {
        return Err(CoreError::Validation(
            "passphrase must have between 3 and 12 words".into(),
        ));
    }
    let count = u32::try_from(WORDS.len()).unwrap_or(1);
    let chosen: Vec<String> = (0..words)
        .map(|_| {
            let word = WORDS
                .get(random::below(count) as usize)
                .copied()
                .unwrap_or("word");
            if capitalize {
                capitalize_first(word)
            } else {
                word.to_owned()
            }
        })
        .collect();
    #[allow(clippy::cast_precision_loss)]
    let entropy_bits = words as f64 * (WORDS.len() as f64).log2();
    Ok(Generated {
        value: chosen.join(separator),
        entropy_bits,
    })
}

fn capitalize_first(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// Generates a numeric PIN (4..=12 digits).
///
/// # Errors
/// Fails when the length is out of range.
pub fn generate_pin(length: usize) -> Result<Generated, CoreError> {
    if !(4..=12).contains(&length) {
        return Err(CoreError::Validation(
            "PIN length must be between 4 and 12".into(),
        ));
    }
    let digits: Vec<char> = DIGITS.chars().collect();
    let value: String = (0..length).map(|_| pick(&digits)).collect();
    #[allow(clippy::cast_precision_loss)]
    let entropy_bits = length as f64 * 10_f64.log2();
    Ok(Generated {
        value,
        entropy_bits,
    })
}

/// Human label for an entropy estimate.
#[must_use]
pub fn strength_label(entropy_bits: f64) -> &'static str {
    match entropy_bits {
        b if b < 40.0 => "weak",
        b if b < 60.0 => "fair",
        b if b < 80.0 => "strong",
        _ => "very strong",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_has_requested_length_and_every_enabled_class() {
        let out = generate_password(&PasswordOptions::default())
            .unwrap()
            .value;
        assert_eq!(out.chars().count(), 20);
        assert!(out.chars().any(|c| c.is_ascii_lowercase()));
        assert!(out.chars().any(|c| c.is_ascii_uppercase()));
        assert!(out.chars().any(|c| c.is_ascii_digit()));
        assert!(out.chars().any(|c| SYMBOLS.contains(c)));
    }

    #[test]
    fn password_can_avoid_ambiguous_characters() {
        let options = PasswordOptions {
            avoid_ambiguous: true,
            length: 128,
            ..PasswordOptions::default()
        };
        let out = generate_password(&options).unwrap().value;
        assert!(!out.chars().any(|c| AMBIGUOUS.contains(c)));
    }

    #[test]
    fn rejects_empty_charset_and_bad_length() {
        let none = PasswordOptions {
            uppercase: false,
            lowercase: false,
            digits: false,
            symbols: false,
            ..PasswordOptions::default()
        };
        assert!(generate_password(&none).is_err());
        assert!(generate_password(&PasswordOptions {
            length: 4,
            ..PasswordOptions::default()
        })
        .is_err());
    }

    #[test]
    fn passphrase_and_pin_shapes() {
        let phrase = generate_passphrase(5, "-", true).unwrap().value;
        assert_eq!(phrase.split('-').count(), 5);
        assert!(phrase.chars().next().is_some_and(char::is_uppercase));
        let pin = generate_pin(6).unwrap().value;
        assert!(pin.len() == 6 && pin.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn strength_labels_increase_with_entropy() {
        assert_eq!(strength_label(20.0), "weak");
        assert_eq!(strength_label(100.0), "very strong");
    }
}
