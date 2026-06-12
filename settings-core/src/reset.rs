use crate::error::SettingsError;
use crate::repository::SettingsRepository;
use crate::schema::SettingsSchema;

/// Manager for reset operations with snapshot support.
pub struct ResetManager<'a, T: SettingsSchema + PartialEq> {
    repository: &'a SettingsRepository<T>,
}

/// A snapshot of all settings values for later restoration.
#[derive(Clone, Debug)]
pub struct SettingsSnapshot {
    pub timestamp: i64,
    pub data: serde_json::Value,
}

impl<'a, T: SettingsSchema + PartialEq> ResetManager<'a, T> {
    pub fn new(repository: &'a SettingsRepository<T>) -> Self {
        Self { repository }
    }

    /// Reset a single field to its default value.
    pub fn reset_field(&self, name: &str) -> Result<(), SettingsError> {
        self.repository.reset_field(name)
    }

    /// Reset multiple fields to their defaults atomically (single persist).
    pub fn reset_fields(&self, names: &[&str]) -> Result<usize, SettingsError> {
        let defaults = T::default();
        let count = names.len();
        self.repository.update(|s| {
            for &name in names {
                if let Ok(val) = defaults.get_field_value(name) {
                    let _ = s.set_field_value(name, val);
                }
            }
        })?;
        Ok(count)
    }

    /// Reset all fields in a category.
    pub fn reset_category(&self, category: &str) -> Result<usize, SettingsError> {
        let current = self.repository.get();
        let fields = current.resettable_fields_in_category(category);
        let names: Vec<&str> = fields.iter().map(|f| f.name).collect();
        self.reset_fields(&names)
    }

    /// Reset all UI (non-persisted-only) settings.
    pub fn reset_ui_settings(&self) -> Result<usize, SettingsError> {
        let current = self.repository.get();
        let fields = current.ui_fields();
        let names: Vec<&str> = fields
            .iter()
            .filter(|f| !f.no_reset)
            .map(|f| f.name)
            .collect();
        self.reset_fields(&names)
    }

    /// Reset everything (including persisted-only fields).
    pub fn reset_all(&self) -> Result<(), SettingsError> {
        self.repository.reset_all()
    }

    /// Create a snapshot of current settings for later restoration.
    pub fn create_snapshot(&self) -> Result<SettingsSnapshot, SettingsError> {
        let current = self.repository.get();
        let data = serde_json::to_value(&current)?;
        Ok(SettingsSnapshot {
            timestamp: chrono::Utc::now().timestamp_millis(),
            data,
        })
    }

    /// Restore settings from a snapshot.
    pub fn restore_snapshot(&self, snapshot: &SettingsSnapshot) -> Result<(), SettingsError> {
        let restored: T = serde_json::from_value(snapshot.data.clone())?;
        self.repository.update(|s| *s = restored)
    }
}
