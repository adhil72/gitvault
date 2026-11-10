/// Source of unique identifiers. Injected so tests can use predictable ids.
pub trait IdGenerator: Send + Sync {
    /// A new unique id (UUID string).
    fn new_id(&self) -> String;
}
