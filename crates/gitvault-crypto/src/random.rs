//! Cryptographically secure random bytes from the operating system.

use rand::{rngs::OsRng, RngCore};

/// Fills `buffer` with random bytes.
pub fn fill(buffer: &mut [u8]) {
    OsRng.fill_bytes(buffer);
}

/// Returns `N` random bytes.
#[must_use]
pub fn bytes<const N: usize>() -> [u8; N] {
    let mut out = [0_u8; N];
    fill(&mut out);
    out
}

/// A uniformly random integer in `0..upper` (rejection sampling, no modulo bias).
///
/// # Panics
/// Never panics; returns 0 when `upper` is 0.
#[must_use]
pub fn below(upper: u32) -> u32 {
    if upper == 0 {
        return 0;
    }
    let zone = u32::MAX - (u32::MAX % upper);
    loop {
        let candidate = u32::from_le_bytes(bytes::<4>());
        if candidate < zone {
            return candidate % upper;
        }
    }
}
