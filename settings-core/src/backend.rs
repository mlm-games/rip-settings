//! Pluggable persistence backends.

use crate::error::SettingsError;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Trait for settings storage backends.
pub trait SettingsBackend: Send + Sync {
    /// Load raw bytes from storage. Returns empty vec if not found.
    fn load(&self) -> Result<Vec<u8>, SettingsError>;

    /// Save raw bytes to storage.
    fn save(&self, data: &[u8]) -> Result<(), SettingsError>;

    /// Check if storage exists.
    fn exists(&self) -> bool;

    /// Delete storage.
    fn delete(&self) -> Result<(), SettingsError>;
}

/// JSON file backend — stores settings as a pretty-printed JSON file.
pub struct JsonFileBackend {
    path: PathBuf,
}

impl JsonFileBackend {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl SettingsBackend for JsonFileBackend {
    fn load(&self) -> Result<Vec<u8>, SettingsError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        std::fs::read(&self.path).map_err(SettingsError::Io)
    }

    fn save(&self, data: &[u8]) -> Result<(), SettingsError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write to temp file then rename for atomicity
        let temp_path = self.path.with_extension("tmp");
        std::fs::write(&temp_path, data)?;
        std::fs::rename(&temp_path, &self.path)?;

        Ok(())
    }

    fn exists(&self) -> bool {
        self.path.exists()
    }

    fn delete(&self) -> Result<(), SettingsError> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

/// In-memory backend — useful for testing.
pub struct MemoryBackend {
    data: Mutex<Vec<u8>>,
}

impl MemoryBackend {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(Vec::new()),
        }
    }

    pub fn with_data(data: Vec<u8>) -> Self {
        Self {
            data: Mutex::new(data),
        }
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsBackend for MemoryBackend {
    fn load(&self) -> Result<Vec<u8>, SettingsError> {
        let guard = self
            .data
            .lock()
            .map_err(|e| SettingsError::Backend(e.to_string()))?;
        Ok(guard.clone())
    }

    fn save(&self, data: &[u8]) -> Result<(), SettingsError> {
        let mut guard = self
            .data
            .lock()
            .map_err(|e| SettingsError::Backend(e.to_string()))?;
        *guard = data.to_vec();
        Ok(())
    }

    fn exists(&self) -> bool {
        let guard = self.data.lock().unwrap_or_else(|e| e.into_inner());
        !guard.is_empty()
    }

    fn delete(&self) -> Result<(), SettingsError> {
        let mut guard = self
            .data
            .lock()
            .map_err(|e| SettingsError::Backend(e.to_string()))?;
        guard.clear();
        Ok(())
    }
}
