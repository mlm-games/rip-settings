use crate::error::SettingsError;
use crate::schema::SettingsSchema;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::Digest;

/// Serialized backup bundle.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SettingsBundle<T> {
    pub format_version: u32,
    pub schema_version: u32,
    pub app_id: String,
    pub exported_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_info: Option<DeviceInfo>,
    pub settings: T,
    pub settings_raw: String,
    pub checksum: String,
}

/// Optional device/platform information included in backups.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DeviceInfo {
    pub platform: String,
    pub os_version: String,
    pub app_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

/// Options for importing settings.
#[derive(Clone, Debug)]
pub struct ImportOptions {
    pub validate_app_id: bool,
    pub validate_checksum: bool,
    pub merge_mode: MergeMode,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            validate_app_id: true,
            validate_checksum: true,
            merge_mode: MergeMode::Overwrite,
        }
    }
}

/// How to merge imported settings.
#[derive(Clone, Debug, PartialEq)]
pub enum MergeMode {
    Overwrite,
    KeepExisting,
    UpdateOnly,
}

/// Typed issues that can occur during backup validation.
#[derive(Clone, Debug, PartialEq)]
pub enum BackupIssue {
    ParseError(String),
    AppIdMismatch { expected: String, actual: String },
    VersionTooNew { actual: u32, supported: u32 },
}

/// Result of validating a backup file.
#[derive(Clone, Debug)]
pub struct BackupValidationResult {
    pub is_valid: bool,
    pub settings_count: usize,
    pub schema_version: u32,
    pub exported_at: i64,
    pub issues: Vec<BackupIssue>,
    pub device_info: Option<DeviceInfo>,
}

/// Manages backup/restore operations.
pub struct SettingsBackupManager {
    app_id: String,
    schema_version: u32,
    device_info_provider: Option<Box<dyn Fn() -> DeviceInfo + Send + Sync>>,
}

impl SettingsBackupManager {
    pub fn new(app_id: impl Into<String>, schema_version: u32) -> Self {
        Self {
            app_id: app_id.into(),
            schema_version,
            device_info_provider: None,
        }
    }

    /// Set a provider for device info included in backups.
    pub fn with_device_info(
        mut self,
        provider: impl Fn() -> DeviceInfo + Send + Sync + 'static,
    ) -> Self {
        self.device_info_provider = Some(Box::new(provider));
        self
    }

    /// Export settings to a JSON string.
    pub fn export<T: SettingsSchema>(&self, settings: &T) -> Result<String, SettingsError> {
        let settings_raw = serde_json::to_string(settings)?;
        let checksum = hex::encode(sha2::Sha256::digest(settings_raw.as_bytes()));

        let bundle = SettingsBundle {
            format_version: 2,
            schema_version: self.schema_version,
            app_id: self.app_id.clone(),
            exported_at: Utc::now().timestamp_millis(),
            device_info: self.device_info_provider.as_ref().map(|p| p()),
            settings: settings.clone(),
            settings_raw,
            checksum,
        };

        serde_json::to_string_pretty(&bundle).map_err(SettingsError::Serialization)
    }

    /// Import settings from a JSON string.
    pub fn import<T: SettingsSchema>(
        &self,
        json_str: &str,
        options: &ImportOptions,
    ) -> Result<T, SettingsError> {
        let bundle: SettingsBundle<T> =
            serde_json::from_str(json_str).map_err(|e| SettingsError::ParseError(e.to_string()))?;

        if options.validate_app_id && bundle.app_id != self.app_id {
            return Err(SettingsError::AppMismatch {
                expected: self.app_id.clone(),
                actual: bundle.app_id,
            });
        }

        if bundle.schema_version > self.schema_version {
            return Err(SettingsError::VersionTooNew {
                actual: bundle.schema_version,
                supported: self.schema_version,
            });
        }

        if options.validate_checksum {
            let expected = hex::encode(sha2::Sha256::digest(bundle.settings_raw.as_bytes()));
            if bundle.checksum != expected {
                return Err(SettingsError::ChecksumMismatch);
            }
        }

        Ok(bundle.settings)
    }

    /// Validate a backup without importing.
    pub fn validate(&self, json_str: &str) -> BackupValidationResult {
        let parse_result: Result<SettingsBundle<serde_json::Value>, _> =
            serde_json::from_str(json_str);

        match parse_result {
            Err(e) => BackupValidationResult {
                is_valid: false,
                settings_count: 0,
                schema_version: 0,
                exported_at: 0,
                issues: vec![BackupIssue::ParseError(e.to_string())],
                device_info: None,
            },
            Ok(bundle) => {
                let mut issues = Vec::new();

                if bundle.app_id != self.app_id {
                    issues.push(BackupIssue::AppIdMismatch {
                        expected: self.app_id.clone(),
                        actual: bundle.app_id,
                    });
                }

                if bundle.schema_version > self.schema_version {
                    issues.push(BackupIssue::VersionTooNew {
                        actual: bundle.schema_version,
                        supported: self.schema_version,
                    });
                }

                let settings_count = bundle.settings.as_object().map(|o| o.len()).unwrap_or(0);

                BackupValidationResult {
                    is_valid: issues.is_empty(),
                    settings_count,
                    schema_version: bundle.schema_version,
                    exported_at: bundle.exported_at,
                    issues,
                    device_info: bundle.device_info,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
    struct TestSettings {
        dark_mode: bool,
        font_size: f32,
    }

    impl SettingsSchema for TestSettings {
        fn schema_version(&self) -> u32 {
            1
        }
        fn app_id(&self) -> &'static str {
            "test_app"
        }
        fn fields(&self) -> Vec<crate::field::FieldMeta> {
            vec![]
        }
        fn get_field_value(&self, name: &str) -> Result<serde_json::Value, SettingsError> {
            match name {
                "dark_mode" => Ok(serde_json::json!(self.dark_mode)),
                "font_size" => Ok(serde_json::json!(self.font_size)),
                _ => Err(SettingsError::UnknownField(name.into())),
            }
        }
        fn set_field_value(
            &mut self,
            name: &str,
            value: serde_json::Value,
        ) -> Result<(), SettingsError> {
            match name {
                "dark_mode" => {
                    self.dark_mode = serde_json::from_value(value)?;
                    Ok(())
                }
                "font_size" => {
                    self.font_size = serde_json::from_value(value)?;
                    Ok(())
                }
                _ => Err(SettingsError::UnknownField(name.into())),
            }
        }
    }

    #[test]
    fn test_export_import_roundtrip() {
        let manager = SettingsBackupManager::new("test_app", 1);
        let settings = TestSettings {
            dark_mode: true,
            font_size: 24.0,
        };
        let exported = manager.export(&settings).unwrap();
        let imported: TestSettings = manager
            .import(&exported, &ImportOptions::default())
            .unwrap();
        assert_eq!(settings, imported);
    }

    #[test]
    fn test_app_id_mismatch() {
        let manager = SettingsBackupManager::new("test_app", 1);
        let other_manager = SettingsBackupManager::new("other_app", 1);
        let settings = TestSettings::default();
        let exported = other_manager.export(&settings).unwrap();
        let result: Result<TestSettings, _> = manager.import(&exported, &ImportOptions::default());
        assert!(matches!(result, Err(SettingsError::AppMismatch { .. })));
    }

    #[test]
    fn test_validate_typed_issues() {
        let manager = SettingsBackupManager::new("test_app", 1);
        let exported = manager.export(&TestSettings::default()).unwrap();
        let result = manager.validate(&exported);
        assert!(result.is_valid);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_validate_mismatch() {
        let other = SettingsBackupManager::new("other_app", 1);
        let exported = other.export(&TestSettings::default()).unwrap();
        let manager = SettingsBackupManager::new("test_app", 1);
        let result = manager.validate(&exported);
        assert!(!result.is_valid);
        assert!(matches!(
            &result.issues[0],
            BackupIssue::AppIdMismatch { .. }
        ));
    }
}
