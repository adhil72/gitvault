use std::time::{SystemTime, UNIX_EPOCH};

use gitvault_core::ports::Clock;

/// [`Clock`] backed by the operating system clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix_secs(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|elapsed| i64::try_from(elapsed.as_secs()).ok())
            .unwrap_or_default()
    }
}
