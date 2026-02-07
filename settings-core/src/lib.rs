//! # multiplatform-settings-core
//!
//! Runtime library for the multiplatform settings framework.
//! Use with `multiplatform-settings-derive` for the `#[derive(Settings)]` macro.

pub mod backend;
pub mod backup;
pub mod error;
pub mod field;
pub mod lock;
pub mod migration;
pub mod platform;
pub mod repository;
pub mod reset;
pub mod schema;
pub mod undo;
pub mod validation;

/// Prelude — import everything you commonly need.
pub mod prelude {
    pub use crate::backend::{JsonFileBackend, MemoryBackend, SettingsBackend};
    pub use crate::backup::SettingsBackupManager;
    pub use crate::error::SettingsError;
    pub use crate::field::{ConfirmationConfig, FieldKind, FieldMeta, ValidationRules};
    pub use crate::lock::SettingsLockManager;
    pub use crate::migration::{KeyDeletion, KeyRename, Migration, MigrationManager};
    pub use crate::platform::default_settings_path;
    pub use crate::repository::SettingsRepository;
    pub use crate::reset::ResetManager;
    pub use crate::schema::SettingsSchema;
    pub use crate::undo::UndoManager;
    pub use crate::validation::ValidationResult;
}
