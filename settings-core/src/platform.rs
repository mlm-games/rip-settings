//! Platform-specific default storage paths.

use std::path::PathBuf;

/// Get the default settings file path for the current platform.
///
/// - **Windows**: `%LOCALAPPDATA%\<app_name>\settings.json`
/// - **macOS**: `~/Library/Application Support/<app_name>/settings.json`
/// - **Linux**: `$XDG_DATA_HOME/<app_name>/settings.json` or `~/.local/share/<app_name>/settings.json`
/// - **iOS**: `<HOME>/Documents/<app_name>/settings.json`
/// - **Android**: `/data/data/<app_name>/files/settings.json`
///
/// **Android note:** This uses the hard-coded path `/data/data/<app_name>/files/`.
/// In a real Android application, obtain the data directory from the Android context
/// at runtime (e.g. via JNI) using `context.getFilesDir()` instead.
pub fn default_settings_path(app_name: &str) -> PathBuf {
    platform_settings_path(app_name)
}

/// Get the default path for a named settings file.
pub fn named_settings_path(app_name: &str, file_name: &str) -> PathBuf {
    let mut path = platform_settings_dir(app_name);
    path.push(file_name);
    path
}

#[cfg(target_os = "windows")]
fn platform_settings_path(app_name: &str) -> PathBuf {
    let mut path = platform_settings_dir(app_name);
    path.push("settings.json");
    path
}

#[cfg(target_os = "windows")]
fn platform_settings_dir(app_name: &str) -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("APPDATA"))
        .unwrap_or_else(|_| {
            let home = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into());
            format!("{}\\AppData\\Local", home)
        });
    PathBuf::from(base).join(app_name)
}

#[cfg(target_os = "macos")]
fn platform_settings_path(app_name: &str) -> PathBuf {
    let mut path = platform_settings_dir(app_name);
    path.push("settings.json");
    path
}

#[cfg(target_os = "macos")]
fn platform_settings_dir(app_name: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join(app_name)
}

#[cfg(target_os = "linux")]
fn platform_settings_path(app_name: &str) -> PathBuf {
    let mut path = platform_settings_dir(app_name);
    path.push("settings.json");
    path
}

#[cfg(target_os = "linux")]
fn platform_settings_dir(app_name: &str) -> PathBuf {
    let data_home = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        format!("{}/.local/share", home)
    });
    PathBuf::from(data_home).join(app_name)
}

#[cfg(target_os = "ios")]
fn platform_settings_path(app_name: &str) -> PathBuf {
    let mut path = platform_settings_dir(app_name);
    path.push("settings.json");
    path
}

#[cfg(target_os = "ios")]
fn platform_settings_dir(app_name: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join("Documents").join(app_name)
}

#[cfg(target_os = "android")]
fn platform_settings_path(app_name: &str) -> PathBuf {
    let mut path = platform_settings_dir(app_name);
    path.push("settings.json");
    path
}

#[cfg(target_os = "android")]
fn platform_settings_dir(app_name: &str) -> PathBuf {
    PathBuf::from(format!("/data/data/{}/files", app_name))
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "ios",
    target_os = "android"
)))]
fn platform_settings_path(app_name: &str) -> PathBuf {
    let mut path = platform_settings_dir(app_name);
    path.push("settings.json");
    path
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "ios",
    target_os = "android"
)))]
fn platform_settings_dir(app_name: &str) -> PathBuf {
    PathBuf::from(".").join(app_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_path_not_empty() {
        let path = default_settings_path("test_app");
        assert!(!path.as_os_str().is_empty());
        assert!(path.to_string_lossy().contains("test_app"));
    }

    #[test]
    fn test_named_path() {
        let path = named_settings_path("test_app", "custom.json");
        assert!(path.to_string_lossy().contains("custom.json"));
    }
}
