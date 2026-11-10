//! Recovery key: 160 random bits shown once as 8 groups of 4 base32 characters.

use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{error::CryptoError, random, vault_key::Kek};

const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
const KEY_BYTES: usize = 20;
const GROUP_LEN: usize = 4;

/// A recovery key. It can unwrap the vault key if the master password is lost.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct RecoveryKey([u8; KEY_BYTES]);

impl RecoveryKey {
    #[must_use]
    pub fn generate() -> Self {
        Self(random::bytes::<KEY_BYTES>())
    }

    /// Human-friendly form, e.g. `7QK2-9XMA-4TRD-8WPE-J3NV-6HBC-AB23-CD45`.
    #[must_use]
    pub fn to_display(&self) -> String {
        let chars = encode_base32(&self.0);
        let groups: Vec<&str> = chars
            .as_bytes()
            .chunks(GROUP_LEN)
            .filter_map(|chunk| std::str::from_utf8(chunk).ok())
            .collect();
        groups.join("-")
    }

    /// Parses the display form (case-insensitive; dashes and spaces ignored).
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidFormat`] for wrong length or characters.
    pub fn parse(text: &str) -> Result<Self, CryptoError> {
        let cleaned: String = text
            .chars()
            .filter(|c| !matches!(c, '-' | ' '))
            .map(|c| c.to_ascii_uppercase())
            .collect();
        let bytes = decode_base32(&cleaned)?;
        let array: [u8; KEY_BYTES] = bytes
            .try_into()
            .map_err(|_| CryptoError::InvalidFormat("recovery key has the wrong length".into()))?;
        Ok(Self(array))
    }

    /// Derives the KEK for this recovery key (the key already has full entropy).
    ///
    /// # Errors
    /// Returns [`CryptoError::Kdf`] if HKDF expansion fails.
    pub fn derive_kek(&self, salt: &[u8]) -> Result<Kek, CryptoError> {
        let hkdf = Hkdf::<Sha256>::new(Some(salt), &self.0);
        let mut out = [0_u8; 32];
        hkdf.expand(b"gitvault:recovery:v1", &mut out)
            .map_err(|e| CryptoError::Kdf(e.to_string()))?;
        Ok(Kek::from_bytes(out))
    }
}

fn encode_base32(bytes: &[u8]) -> String {
    let mut out = String::new();
    let (mut buffer, mut bits) = (0_u32, 0_u32);
    for &byte in bytes {
        buffer = (buffer << 8) | u32::from(byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(char::from(ALPHABET[((buffer >> bits) & 31) as usize]));
        }
    }
    out
}

fn decode_base32(text: &str) -> Result<Vec<u8>, CryptoError> {
    let mut out = Vec::new();
    let (mut buffer, mut bits) = (0_u32, 0_u32);
    for ch in text.bytes() {
        let value = ALPHABET
            .iter()
            .position(|&c| c == ch)
            .ok_or_else(|| CryptoError::InvalidFormat("invalid recovery key character".into()))?;
        buffer = (buffer << 5) | u32::try_from(value).unwrap_or(0);
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(u8::try_from((buffer >> bits) & 0xFF).unwrap_or(0));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_has_eight_groups_of_four() {
        let shown = RecoveryKey::generate().to_display();
        let groups: Vec<&str> = shown.split('-').collect();
        assert_eq!(groups.len(), 8);
        assert!(groups.iter().all(|g| g.len() == 4));
    }

    #[test]
    fn parse_round_trips_and_is_forgiving() {
        let key = RecoveryKey::generate();
        let shown = key.to_display();
        let sloppy = shown.to_lowercase().replace('-', " ");
        assert_eq!(RecoveryKey::parse(&sloppy).unwrap().0, key.0);
    }

    #[test]
    fn parse_rejects_bad_input() {
        assert!(RecoveryKey::parse("ABCD-EFGH").is_err());
        assert!(RecoveryKey::parse("0000-0000-0000-0000-0000-0000-0000-0000").is_err());
    }

    #[test]
    fn same_key_and_salt_derive_same_kek() {
        let key = RecoveryKey::generate();
        let a = key.derive_kek(b"salt").unwrap();
        let b = RecoveryKey::parse(&key.to_display())
            .unwrap()
            .derive_kek(b"salt")
            .unwrap();
        assert_eq!(a.as_bytes(), b.as_bytes());
    }
}
