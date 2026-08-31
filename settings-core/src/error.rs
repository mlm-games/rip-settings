use thiserror::Error;

#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Unknown field: {0}")]
    UnknownField(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("App ID mismatch: expected {expected}, got {actual}")]
    AppMismatch { expected: String, actual: String },

    #[error("Schema version {actual} is newer than supported {supported}")]
    VersionTooNew { actual: u32, supported: u32 },

    #[error("Checksum mismatch - file may be corrupted")]
    ChecksumMismatch,

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Lock error: {0}")]
    LockError(String),

    #[error("PIN must be at least {min_length} characters")]
    PinTooShort { min_length: usize },

    #[error("Invalid PIN")]
    InvalidPin,

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),
}
