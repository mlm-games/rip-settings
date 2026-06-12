use crate::error::SettingsError;
use crate::repository::SettingsRepository;
use crate::schema::SettingsSchema;
use chrono::Utc;
use std::collections::VecDeque;
use std::sync::RwLock;

/// Records a single field change.
#[derive(Clone, Debug)]
pub struct SettingChange {
    pub field_name: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub timestamp: i64,
}

/// Manages undo/redo history for setting changes.
///
/// Wraps a [`SettingsRepository`] and automatically records changes
/// when using [`UndoManager::set_field`] instead of [`SettingsRepository::set_field`].
pub struct UndoManager<T: SettingsSchema + PartialEq> {
    repository: std::sync::Arc<SettingsRepository<T>>,
    undo_stack: RwLock<VecDeque<SettingChange>>,
    redo_stack: RwLock<VecDeque<SettingChange>>,
    max_history: usize,
}

impl<T: SettingsSchema + PartialEq> UndoManager<T> {
    pub fn new(repository: std::sync::Arc<SettingsRepository<T>>, max_history: usize) -> Self {
        Self {
            repository,
            undo_stack: RwLock::new(VecDeque::new()),
            redo_stack: RwLock::new(VecDeque::new()),
            max_history,
        }
    }

    /// Set a field and automatically record the change for undo.
    pub fn set_field(&self, name: &str, value: serde_json::Value) -> Result<(), SettingsError> {
        let old_val = self.repository.get_field(name)?;
        self.repository.set_field(name, value.clone())?;
        self.record_change_inner(name, old_val, value);
        Ok(())
    }

    /// Record a change for undo support (separate call required when using
    /// [`SettingsRepository::set_field`] directly).
    pub fn record_change(
        &self,
        field_name: impl Into<String>,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    ) {
        self.record_change_inner(field_name, old_value, new_value);
    }

    fn record_change_inner(
        &self,
        field_name: impl Into<String>,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    ) {
        let change = SettingChange {
            field_name: field_name.into(),
            old_value,
            new_value,
            timestamp: Utc::now().timestamp_millis(),
        };

        let mut undo = self.undo_stack.write().unwrap_or_else(|e| e.into_inner());
        undo.push_back(change);
        while undo.len() > self.max_history {
            undo.pop_front();
        }

        self.redo_stack.write().unwrap_or_else(|e| e.into_inner()).clear();
    }

    /// Undo the last change.
    pub fn undo(&self) -> Result<bool, SettingsError> {
        let change = {
            let mut undo = self.undo_stack.write().unwrap_or_else(|e| e.into_inner());
            undo.pop_back()
        };

        let Some(change) = change else {
            return Ok(false);
        };

        self.repository
            .set_field(&change.field_name, change.old_value.clone())?;

        self.redo_stack
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .push_back(change);
        Ok(true)
    }

    /// Redo the last undone change.
    pub fn redo(&self) -> Result<bool, SettingsError> {
        let change = {
            let mut redo = self.redo_stack.write().unwrap_or_else(|e| e.into_inner());
            redo.pop_back()
        };

        let Some(change) = change else {
            return Ok(false);
        };

        self.repository
            .set_field(&change.field_name, change.new_value.clone())?;

        self.undo_stack
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .push_back(change);
        Ok(true)
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.read().unwrap_or_else(|e| e.into_inner()).is_empty()
    }

    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.read().unwrap_or_else(|e| e.into_inner()).is_empty()
    }

    pub fn undo_description(&self) -> Option<String> {
        self.undo_stack
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .back()
            .map(|c| format!("Undo: {}", c.field_name))
    }

    pub fn redo_description(&self) -> Option<String> {
        self.redo_stack
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .back()
            .map(|c| format!("Redo: {}", c.field_name))
    }

    /// Clear all undo/redo history.
    pub fn clear_history(&self) {
        self.undo_stack
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        self.redo_stack
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }
}
