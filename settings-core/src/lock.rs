use crate::error::SettingsError;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

/// Stored lock state.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LockState {
    pub enabled: bool,
    pub pin_hash: Option<String>,
    pub timeout_ms: u64,
    pub last_unlock_ms: i64,
}

/// PIN hasher trait for customizable hashing.
pub trait PinHasher: Send + Sync {
    fn hash(&self, pin: &str) -> String;
    fn verify(&self, pin: &str, hash: &str) -> bool;
}

/// Default hasher using Argon2id.
pub struct DefaultPinHasher;

impl PinHasher for DefaultPinHasher {
    fn hash(&self, pin: &str) -> String {
        use argon2::password_hash::SaltString;
        use argon2::password_hash::rand_core::OsRng;
        use argon2::{Argon2, PasswordHasher};

        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(pin.as_bytes(), &salt)
            .expect("Argon2 hashing failed")
            .to_string()
    }

    fn verify(&self, pin: &str, hash: &str) -> bool {
        use argon2::password_hash::PasswordHash;
        use argon2::{Argon2, PasswordVerifier};

        let Ok(parsed) = PasswordHash::new(hash) else {
            return false;
        };
        Argon2::default()
            .verify_password(pin.as_bytes(), &parsed)
            .is_ok()
    }
}

/// Manages PIN-based settings locking.
pub struct SettingsLockManager {
    state: RwLock<LockState>,
    hasher: Box<dyn PinHasher>,
    min_pin_length: usize,
}

impl SettingsLockManager {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(LockState::default()),
            hasher: Box::new(DefaultPinHasher),
            min_pin_length: 4,
        }
    }

    pub fn with_hasher(mut self, hasher: impl PinHasher + 'static) -> Self {
        self.hasher = Box::new(hasher);
        self
    }

    pub fn with_min_pin_length(mut self, length: usize) -> Self {
        self.min_pin_length = length;
        self
    }

    pub fn load_state(&self, data: &[u8]) -> Result<(), SettingsError> {
        if data.is_empty() {
            return Ok(());
        }
        let state: LockState =
            serde_json::from_slice(data).map_err(|e| SettingsError::ParseError(e.to_string()))?;
        *self
            .state
            .write()
            .map_err(|e| SettingsError::LockPoisoned(e.to_string()))? = state;
        Ok(())
    }

    pub fn save_state(&self) -> Result<Vec<u8>, SettingsError> {
        let state = self
            .state
            .read()
            .map_err(|e| SettingsError::LockPoisoned(e.to_string()))?
            .clone();
        serde_json::to_vec(&state).map_err(SettingsError::Serialization)
    }

    #[must_use]
    pub fn is_lock_enabled(&self) -> bool {
        self.state.read().unwrap_or_else(|e| e.into_inner()).enabled
    }

    #[must_use]
    pub fn is_locked(&self) -> bool {
        let state = self.state.read().unwrap_or_else(|e| e.into_inner());
        if !state.enabled {
            return false;
        }
        if state.timeout_ms == 0 {
            return true;
        }
        let now = chrono::Utc::now().timestamp_millis();
        (now - state.last_unlock_ms) > state.timeout_ms as i64
    }

    pub fn enable_lock(&self, pin: &str) -> Result<(), SettingsError> {
        if pin.len() < self.min_pin_length {
            return Err(SettingsError::PinTooShort {
                min_length: self.min_pin_length,
            });
        }

        let mut state = self
            .state
            .write()
            .map_err(|e| SettingsError::LockPoisoned(e.to_string()))?;
        state.enabled = true;
        state.pin_hash = Some(self.hasher.hash(pin));
        Ok(())
    }

    pub fn disable_lock(&self, pin: &str) -> Result<(), SettingsError> {
        if !self.validate_pin(pin)? {
            return Err(SettingsError::InvalidPin);
        }

        let mut state = self
            .state
            .write()
            .map_err(|e| SettingsError::LockPoisoned(e.to_string()))?;
        state.enabled = false;
        state.pin_hash = None;
        state.last_unlock_ms = 0;
        Ok(())
    }

    pub fn validate_pin(&self, pin: &str) -> Result<bool, SettingsError> {
        let state = self
            .state
            .read()
            .map_err(|e| SettingsError::LockPoisoned(e.to_string()))?;
        match &state.pin_hash {
            Some(hash) => Ok(self.hasher.verify(pin, hash)),
            None => Ok(false),
        }
    }

    pub fn unlock(&self, pin: &str) -> Result<(), SettingsError> {
        if !self.validate_pin(pin)? {
            return Err(SettingsError::InvalidPin);
        }

        let mut state = self
            .state
            .write()
            .map_err(|e| SettingsError::LockPoisoned(e.to_string()))?;
        state.last_unlock_ms = chrono::Utc::now().timestamp_millis();
        Ok(())
    }

    pub fn lock(&self) {
        let mut state = self.state.write().unwrap_or_else(|e| e.into_inner());
        state.last_unlock_ms = 0;
    }

    pub fn set_timeout(&self, timeout_ms: u64) {
        let mut state = self.state.write().unwrap_or_else(|e| e.into_inner());
        state.timeout_ms = timeout_ms;
    }

    pub fn change_pin(&self, current_pin: &str, new_pin: &str) -> Result<(), SettingsError> {
        if !self.validate_pin(current_pin)? {
            return Err(SettingsError::InvalidPin);
        }
        if new_pin.len() < self.min_pin_length {
            return Err(SettingsError::PinTooShort {
                min_length: self.min_pin_length,
            });
        }

        let mut state = self
            .state
            .write()
            .map_err(|e| SettingsError::LockPoisoned(e.to_string()))?;
        state.pin_hash = Some(self.hasher.hash(new_pin));
        Ok(())
    }

    #[must_use]
    pub fn has_pin_set(&self) -> bool {
        self.state
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .pin_hash
            .is_some()
    }
}

impl Default for SettingsLockManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_lifecycle() {
        let manager = SettingsLockManager::new();

        assert!(!manager.is_lock_enabled());
        assert!(!manager.is_locked());

        manager.enable_lock("1234").unwrap();
        assert!(manager.is_lock_enabled());
        assert!(manager.is_locked());

        manager.unlock("1234").unwrap();
        assert!(manager.is_locked());

        manager.set_timeout(999_999_999);
        manager.unlock("1234").unwrap();
        assert!(!manager.is_locked());

        assert!(manager.unlock("0000").is_err());
    }

    #[test]
    fn test_pin_too_short() {
        let manager = SettingsLockManager::new();
        assert!(matches!(
            manager.enable_lock("12"),
            Err(SettingsError::PinTooShort { .. })
        ));
    }

    #[test]
    fn test_change_pin() {
        let manager = SettingsLockManager::new();
        manager.enable_lock("1234").unwrap();

        assert!(manager.change_pin("1234", "5678").is_ok());
        assert!(manager.validate_pin("5678").unwrap());
        assert!(!manager.validate_pin("1234").unwrap());
    }
}
