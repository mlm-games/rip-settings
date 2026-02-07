//! Central repository for reading/writing settings with change notification.

use crate::backend::SettingsBackend;
use crate::error::SettingsError;
use crate::schema::SettingsSchema;
use crate::validation;
use std::sync::{Arc, RwLock};
use tokio::sync::watch;

/// Callback type for setting change listeners.
pub type ChangeListener<T> = Box<dyn Fn(&str, &T, &T) + Send + Sync>;

/// Repository for reading/writing settings via a pluggable backend.
pub struct SettingsRepository<T: SettingsSchema> {
    backend: Box<dyn SettingsBackend>,
    current: Arc<RwLock<T>>,
    sender: watch::Sender<T>,
    receiver: watch::Receiver<T>,
    change_listeners: Arc<RwLock<Vec<ChangeListener<T>>>>,
}

impl<T: SettingsSchema + PartialEq> SettingsRepository<T> {
    /// Create a new repository, loading from the backend or using defaults.
    pub fn new(backend: Box<dyn SettingsBackend>) -> Self {
        let initial = Self::load_from_backend(&*backend);
        let (sender, receiver) = watch::channel(initial.clone());

        Self {
            backend,
            current: Arc::new(RwLock::new(initial)),
            sender,
            receiver,
            change_listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get the current settings value.
    pub fn get(&self) -> T {
        self.current.read().unwrap().clone()
    }

    /// Update settings with a mutation function.
    pub fn update(&self, f: impl FnOnce(&mut T)) -> Result<(), SettingsError> {
        let old = self.get();
        let mut new = old.clone();
        f(&mut new);

        if old == new {
            return Ok(());
        }

        self.persist(&new)?;

        *self.current.write().unwrap() = new.clone();
        let _ = self.sender.send(new.clone());

        // Notify listeners — find which fields changed
        let fields = new.fields();
        for field in &fields {
            if old.get_field_value(field.name).ok() != new.get_field_value(field.name).ok() {
                self.notify_change(field.name, &old, &new);
            }
        }

        Ok(())
    }

    /// Set a single field by name from a JSON value.
    pub fn set_field(&self, name: &str, value: serde_json::Value) -> Result<(), SettingsError> {
        let mut current = self.get();

        // Validate before setting
        if let Some(field_meta) = current.field_by_name(name) {
            if let Some(ref rules) = field_meta.validation {
                let result = validation::validate_value(&value, rules);
                if let crate::validation::ValidationResult::Invalid(msg) = result {
                    return Err(SettingsError::ValidationFailed(msg));
                }
            }
        }

        let old = current.clone();
        current.set_field_value(name, value)?;

        if old == current {
            return Ok(());
        }

        self.persist(&current)?;

        *self.current.write().unwrap() = current.clone();
        let _ = self.sender.send(current.clone());
        self.notify_change(name, &old, &current);

        Ok(())
    }

    /// Get a single field value by name as JSON.
    pub fn get_field(&self, name: &str) -> Result<serde_json::Value, SettingsError> {
        self.get().get_field_value(name)
    }

    /// Subscribe to changes (returns a watch receiver).
    pub fn subscribe(&self) -> watch::Receiver<T> {
        self.receiver.clone()
    }

    /// Add a change listener.
    pub fn add_change_listener(&self, listener: ChangeListener<T>) {
        self.change_listeners.write().unwrap().push(listener);
    }

    /// Reload settings from the backend.
    pub fn reload(&self) -> Result<(), SettingsError> {
        let loaded = Self::load_from_backend(&*self.backend);
        *self.current.write().unwrap() = loaded.clone();
        let _ = self.sender.send(loaded);
        Ok(())
    }

    /// Reset all settings to defaults and persist.
    pub fn reset_all(&self) -> Result<(), SettingsError> {
        let defaults = T::default();
        self.persist(&defaults)?;
        *self.current.write().unwrap() = defaults.clone();
        let _ = self.sender.send(defaults);
        Ok(())
    }

    /// Reset a single field to its default.
    pub fn reset_field(&self, name: &str) -> Result<(), SettingsError> {
        let defaults = T::default();
        let default_value = defaults.get_field_value(name)?;
        self.set_field(name, default_value)
    }

    /// Get the backend reference (for backup manager, etc.).
    pub fn backend(&self) -> &dyn SettingsBackend {
        &*self.backend
    }

    fn persist(&self, value: &T) -> Result<(), SettingsError> {
        let data = serde_json::to_vec_pretty(value)?;
        self.backend.save(&data)
    }

    fn load_from_backend(backend: &dyn SettingsBackend) -> T {
        match backend.load() {
            Ok(data) if !data.is_empty() => serde_json::from_slice(&data).unwrap_or_default(),
            _ => T::default(),
        }
    }

    fn notify_change(&self, field_name: &str, old: &T, new: &T) {
        if let Ok(listeners) = self.change_listeners.read() {
            for listener in listeners.iter() {
                listener(field_name, old, new);
            }
        }
    }
}
