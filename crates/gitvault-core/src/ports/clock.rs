/// Source of the current time. Injected so tests can control it.
pub trait Clock: Send + Sync {
    /// Seconds since the Unix epoch.
    fn now_unix_secs(&self) -> i64;
}
