//! Undo/redo support for setting changes.

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

    /// Record a change for undo support.
    pub fn record_change(
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

        let mut undo = self.undo_stack.write().unwrap();
        undo.push_back(change);
        while undo.len() > self.max_history {
            undo.pop_front();
        }

        // Clear redo stack on new change
        self.redo_stack.write().unwrap().clear();
    }

    /// Undo the last change.
    pub fn undo(&self) -> Result<bool, SettingsError> {
        let change = {
            let mut undo = self.undo_stack.write().unwrap();
            undo.pop_back()
        };

        let Some(change) = change else {
            return Ok(false);
        };

        self.repository
            .set_field(&change.field_name, change.old_value.clone())?;

        self.redo_stack.write().unwrap().push_back(change);
        Ok(true)
    }

    /// Redo the last undone change.
    pub fn redo(&self) -> Result<bool, SettingsError> {
        let change = {
            let mut redo = self.redo_stack.write().unwrap();
            redo.pop_back()
        };

        let Some(change) = change else {
            return Ok(false);
        };

        self.repository
            .set_field(&change.field_name, change.new_value.clone())?;

        self.undo_stack.write().unwrap().push_back(change);
        Ok(true)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.read().unwrap().is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.read().unwrap().is_empty()
    }

    pub fn undo_description(&self) -> Option<String> {
        self.undo_stack
            .read()
            .unwrap()
            .back()
            .map(|c| format!("Undo: {}", c.field_name))
    }

    pub fn redo_description(&self) -> Option<String> {
        self.redo_stack
            .read()
            .unwrap()
            .back()
            .map(|c| format!("Redo: {}", c.field_name))
    }

    /// Clear all undo/redo history.
    pub fn clear_history(&self) {
        self.undo_stack.write().unwrap().clear();
        self.redo_stack.write().unwrap().clear();
    }
}
