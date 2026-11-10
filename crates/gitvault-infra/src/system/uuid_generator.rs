use gitvault_core::ports::IdGenerator;

/// [`IdGenerator`] producing random (v4) UUIDs.
pub struct UuidGenerator;

impl IdGenerator for UuidGenerator {
    fn new_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
