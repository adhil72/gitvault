use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{
    crypto::{random_bytes, Kek},
    error::AppError,
};

const RECOVERY_INFO: &[u8] = b"gitvault:v2:recovery";
const RECOVERY_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
const RECOVERY_BYTES: usize = 20;

/// 160-bit recovery key shown once during initialization (8 groups of 4 base32 chars).
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct RecoveryKey([u8; RECOVERY_BYTES]);

impl RecoveryKey {
    #[must_use]
    pub fn generate() -> Self {
        Self(random_bytes::<RECOVERY_BYTES>())
    }

    #[must_use]
    pub fn to_display(&self) -> String {
        let chars = encode_base32(&self.0);
        chars
            .as_bytes()
            .chunks(4)
            .filter_map(|chunk| std::str::from_utf8(chunk).ok())
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn parse(text: &str) -> Result<Self, AppError> {
        let cleaned: String = text
            .chars()
            .filter(|c| !matches!(c, '-' | ' ' | '\t' | '\n' | '\r'))
            .map(|c| c.to_ascii_uppercase())
            .collect();
        let bytes = decode_base32(&cleaned)?;
        let arr: [u8; RECOVERY_BYTES] = bytes
            .try_into()
            .map_err(|_| AppError::Validation("recovery key has invalid length".into()))?;
        Ok(Self(arr))
    }

    pub fn derive_kek(&self, salt: &[u8]) -> Result<Kek, AppError> {
        let hkdf = Hkdf::<Sha256>::new(Some(salt), &self.0);
        let mut out = [0_u8; 32];
        hkdf.expand(RECOVERY_INFO, &mut out)
            .map_err(|e| AppError::Crypto(format!("recovery HKDF failed: {e}")))?;
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
            out.push(char::from(
                RECOVERY_ALPHABET[((buffer >> bits) & 31) as usize],
            ));
        }
    }
    out
}

fn decode_base32(text: &str) -> Result<Vec<u8>, AppError> {
    let mut out = Vec::new();
    let (mut buffer, mut bits) = (0_u32, 0_u32);
    for ch in text.bytes() {
        let value = RECOVERY_ALPHABET
            .iter()
            .position(|&c| c == ch)
            .ok_or_else(|| AppError::Validation("invalid character in recovery key".into()))?;
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
    use crate::crypto::{unwrap_key, wrap_key, VaultKey};

    #[test]
    fn recovery_key_parse_and_unwrap_round_trip() {
        let key = VaultKey::generate();
        let rec = RecoveryKey::generate();
        let salt = random_bytes::<16>();
        let wrapped = wrap_key(&rec.derive_kek(&salt).unwrap(), &key).unwrap();

        let sloppy = rec.to_display().to_lowercase().replace('-', " ");
        let parsed = RecoveryKey::parse(&sloppy).unwrap();
        let unwrapped = unwrap_key(&parsed.derive_kek(&salt).unwrap(), &wrapped).unwrap();
        assert_eq!(unwrapped.as_bytes(), key.as_bytes());
    }
}
