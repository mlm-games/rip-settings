use crate::backend::SettingsBackend;
use crate::error::SettingsError;
use crate::schema::SettingsSchema;
use crate::validation;
use crate::validation::ValidationResult;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::watch;

pub type ChangeListener<T> = Box<dyn Fn(&str, &T, &T) + Send + Sync>;

/// Repository for reading/writing settings via a pluggable backend.
pub struct SettingsRepository<T: SettingsSchema> {
    backend: Box<dyn SettingsBackend>,
    current: Mutex<T>,
    sender: watch::Sender<T>,
    receiver: watch::Receiver<T>,
    change_listeners: Arc<RwLock<Vec<ChangeListener<T>>>>,
}

impl<T: SettingsSchema + PartialEq> SettingsRepository<T> {
    pub fn new(backend: Box<dyn SettingsBackend>) -> Self {
        let initial = Self::load_from_backend(&*backend);
        let (sender, receiver) = watch::channel(initial.clone());
        Self {
            backend,
            current: Mutex::new(initial),
            sender,
            receiver,
            change_listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get the current settings value.
    pub fn get(&self) -> T {
        self.current
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Update settings with a mutation function.
    /// Validates all fields after mutation — if validation fails, the mutation is not persisted.
    pub fn update(&self, f: impl FnOnce(&mut T)) -> Result<(), SettingsError> {
        let mut guard = self
            .current
            .lock()
            .map_err(|e| SettingsError::LockPoisoned(e.to_string()))?;
        let old = guard.clone();
        let mut candidate = old.clone();
        f(&mut candidate);

        if old == candidate {
            return Ok(());
        }

        // Validate all fields after mutation
        for field in candidate.fields() {
            if let Some(ref rules) = field.validation {
                if let Ok(val) = candidate.get_field_value(field.name) {
                    if let ValidationResult::Invalid(msg) = validation::validate_value(&val, rules) {
                        return Err(SettingsError::ValidationFailed(msg));
                    }
                }
            }
        }

        *guard = candidate;
        self.persist(&guard)?;
        let _ = self.sender.send(guard.clone());

        let fields = guard.fields();
        for field in &fields {
            if old.get_field_value(field.name).ok() != guard.get_field_value(field.name).ok() {
                self.notify_change(field.name, &old, &guard);
            }
        }

        Ok(())
    }

    /// Set a single field by name from a JSON value.
    pub fn set_field(&self, name: &str, value: serde_json::Value) -> Result<(), SettingsError> {
        let mut guard = self
            .current
            .lock()
            .map_err(|e| SettingsError::LockPoisoned(e.to_string()))?;

        if let Some(field_meta) = guard.field_by_name(name) {
            if let Some(ref rules) = field_meta.validation {
                let result = validation::validate_value(&value, rules);
                if let ValidationResult::Invalid(msg) = result {
                    return Err(SettingsError::ValidationFailed(msg));
                }
            }
        }

        let old = guard.clone();
        guard.set_field_value(name, value)?;

        if old == *guard {
            return Ok(());
        }

        self.persist(&guard)?;
        let _ = self.sender.send(guard.clone());
        self.notify_change(name, &old, &guard);

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
        self.change_listeners
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .push(listener);
    }

    /// Reload settings from the backend.
    pub fn reload(&self) -> Result<(), SettingsError> {
        let loaded = Self::load_from_backend(&*self.backend);
        let mut guard = self
            .current
            .lock()
            .map_err(|e| SettingsError::LockPoisoned(e.to_string()))?;
        *guard = loaded.clone();
        let _ = self.sender.send(loaded);
        Ok(())
    }

    /// Reset all settings to defaults and persist.
    pub fn reset_all(&self) -> Result<(), SettingsError> {
        let defaults = T::default();
        let mut guard = self
            .current
            .lock()
            .map_err(|e| SettingsError::LockPoisoned(e.to_string()))?;
        *guard = defaults;
        self.persist(&guard)?;
        let _ = self.sender.send(guard.clone());
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
