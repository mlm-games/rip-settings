//! Schema migration support.

use crate::error::SettingsError;

/// A single migration step.
pub trait Migration: Send + Sync {
    /// Source version this migration applies from.
    fn from_version(&self) -> u32;
    /// Target version after migration.
    fn to_version(&self) -> u32;
    /// Apply the migration to a raw JSON value.
    fn migrate(&self, value: &mut serde_json::Value) -> Result<(), SettingsError>;
}

/// Rename a key from one name to another.
pub struct KeyRename {
    pub from_ver: u32,
    pub to_ver: u32,
    pub old_key: String,
    pub new_key: String,
}

impl KeyRename {
    pub fn new(
        from_ver: u32,
        to_ver: u32,
        old_key: impl Into<String>,
        new_key: impl Into<String>,
    ) -> Self {
        Self {
            from_ver: from_ver,
            to_ver: to_ver,
            old_key: old_key.into(),
            new_key: new_key.into(),
        }
    }
}

impl Migration for KeyRename {
    fn from_version(&self) -> u32 {
        self.from_ver
    }
    fn to_version(&self) -> u32 {
        self.to_ver
    }
    fn migrate(&self, value: &mut serde_json::Value) -> Result<(), SettingsError> {
        if let Some(obj) = value.as_object_mut() {
            if let Some(v) = obj.remove(&self.old_key) {
                obj.insert(self.new_key.clone(), v);
            }
        }
        Ok(())
    }
}

/// Delete one or more keys.
pub struct KeyDeletion {
    pub from_ver: u32,
    pub to_ver: u32,
    pub keys: Vec<String>,
}

impl KeyDeletion {
    pub fn new(from_ver: u32, to_ver: u32, keys: Vec<String>) -> Self {
        Self {
            from_ver: from_ver,
            to_ver: to_ver,
            keys,
        }
    }
}

impl Migration for KeyDeletion {
    fn from_version(&self) -> u32 {
        self.from_ver
    }
    fn to_version(&self) -> u32 {
        self.to_ver
    }
    fn migrate(&self, value: &mut serde_json::Value) -> Result<(), SettingsError> {
        if let Some(obj) = value.as_object_mut() {
            for key in &self.keys {
                obj.remove(key);
            }
        }
        Ok(())
    }
}

/// Custom migration with a closure.
pub struct CustomMigration {
    from_ver: u32,
    to_ver: u32,
    func: Box<dyn Fn(&mut serde_json::Value) -> Result<(), SettingsError> + Send + Sync>,
}

impl CustomMigration {
    pub fn new(
        from_ver: u32,
        to_ver: u32,
        func: impl Fn(&mut serde_json::Value) -> Result<(), SettingsError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            from_ver,
            to_ver,
            func: Box::new(func),
        }
    }
}

impl Migration for CustomMigration {
    fn from_version(&self) -> u32 {
        self.from_ver
    }
    fn to_version(&self) -> u32 {
        self.to_ver
    }
    fn migrate(&self, value: &mut serde_json::Value) -> Result<(), SettingsError> {
        (self.func)(value)
    }
}

/// Result of running migrations.
#[derive(Clone, Debug)]
pub enum MigrationResult {
    NoMigrationNeeded,
    Success {
        from_version: u32,
        to_version: u32,
        migrations_applied: usize,
    },
    PartialSuccess {
        from_version: u32,
        to_version: u32,
        migrations_applied: usize,
        errors: Vec<(u32, String)>,
    },
}

/// Manages and applies migrations to raw settings data.
pub struct MigrationManager {
    current_version: u32,
    migrations: Vec<Box<dyn Migration>>,
}

impl MigrationManager {
    pub fn new(current_version: u32) -> Self {
        Self {
            current_version,
            migrations: Vec::new(),
        }
    }

    /// Add a migration step.
    pub fn add(mut self, migration: impl Migration + 'static) -> Self {
        self.migrations.push(Box::new(migration));
        self
    }

    /// Add a key rename migration.
    pub fn add_key_rename(
        self,
        from_ver: u32,
        to_ver: u32,
        old_key: impl Into<String>,
        new_key: impl Into<String>,
    ) -> Self {
        self.add(KeyRename::new(from_ver, to_ver, old_key, new_key))
    }

    /// Add a key deletion migration.
    pub fn add_key_deletion(self, from_ver: u32, to_ver: u32, keys: Vec<String>) -> Self {
        self.add(KeyDeletion::new(from_ver, to_ver, keys))
    }

    /// Apply all applicable migrations to raw JSON data.
    pub fn migrate(&self, data: &mut serde_json::Value) -> MigrationResult {
        let stored_version = data
            .get("__schema_version")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        if stored_version >= self.current_version {
            return MigrationResult::NoMigrationNeeded;
        }

        let mut applicable: Vec<&dyn Migration> = self
            .migrations
            .iter()
            .filter(|m| {
                m.from_version() >= stored_version && m.to_version() <= self.current_version
            })
            .map(|m| m.as_ref())
            .collect();

        applicable.sort_by_key(|m| m.from_version());

        if applicable.is_empty() {
            // Just update version stamp
            if let Some(obj) = data.as_object_mut() {
                obj.insert(
                    "__schema_version".into(),
                    serde_json::json!(self.current_version),
                );
            }
            return MigrationResult::NoMigrationNeeded;
        }

        let mut applied = 0usize;
        let mut errors = Vec::new();

        for migration in &applicable {
            match migration.migrate(data) {
                Ok(()) => applied += 1,
                Err(e) => errors.push((migration.to_version(), e.to_string())),
            }
        }

        if let Some(obj) = data.as_object_mut() {
            obj.insert(
                "__schema_version".into(),
                serde_json::json!(self.current_version),
            );
        }

        if errors.is_empty() {
            MigrationResult::Success {
                from_version: stored_version,
                to_version: self.current_version,
                migrations_applied: applied,
            }
        } else {
            MigrationResult::PartialSuccess {
                from_version: stored_version,
                to_version: self.current_version,
                migrations_applied: applied,
                errors,
            }
        }
    }

    /// Get the stored schema version from raw JSON.
    pub fn stored_version(data: &serde_json::Value) -> u32 {
        data.get("__schema_version")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_rename() {
        let mut data = serde_json::json!({
            "old_name": "hello",
            "other": 42
        });

        let manager = MigrationManager::new(2).add_key_rename(0, 1, "old_name", "new_name");

        let result = manager.migrate(&mut data);
        assert!(matches!(result, MigrationResult::Success { .. }));
        assert_eq!(data["new_name"], "hello");
        assert!(data.get("old_name").is_none());
    }

    #[test]
    fn test_key_deletion() {
        let mut data = serde_json::json!({
            "keep_me": true,
            "delete_me": "gone",
            "also_delete": 123
        });

        let manager = MigrationManager::new(2).add_key_deletion(
            0,
            1,
            vec!["delete_me".into(), "also_delete".into()],
        );

        manager.migrate(&mut data);
        assert!(data.get("delete_me").is_none());
        assert!(data.get("also_delete").is_none());
        assert_eq!(data["keep_me"], true);
    }

    #[test]
    fn test_custom_migration() {
        let mut data = serde_json::json!({
            "font_size": 14
        });

        let migration = CustomMigration::new(0, 1, |value| {
            if let Some(obj) = value.as_object_mut() {
                if let Some(size) = obj.get("font_size").and_then(|v| v.as_i64()) {
                    obj.insert("font_size".into(), serde_json::json!(size * 2));
                }
            }
            Ok(())
        });

        let manager = MigrationManager::new(1).add(migration);
        manager.migrate(&mut data);
        assert_eq!(data["font_size"], 28);
    }
}
