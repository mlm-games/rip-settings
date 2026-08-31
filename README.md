# rip-settings

Type-safe settings management for Rust with declarative schema generation.

[![Crates.io](https://img.shields.io/crates/v/rip-settings)](https://crates.io/crates/rip-settings)
[![docs.rs](https://img.shields.io/docsrs/rip-settings)](https://docs.rs/rip-settings)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

> Rust port of [kmp-settings](https://github.com/mlm-games/kmp-settings) - adapted for Rust's type system and ecosystem.

## Features

- **Declarative settings**: Define settings with `#[derive(Settings)]` and `#[setting(...)]` on struct fields
- **Auto-generated schema**: Derive macro generates type-safe `SettingsSchema` with field metadata, categories, and validation
- **Pluggable backends**: `JsonFileBackend`, `MemoryBackend`, or implement `SettingsBackend` yourself
- **Built-in validation**: Range, length, pattern, required checks with custom error messages
- **Cross-platform paths**: `default_settings_path()` resolves platform-appropriate directories via `#[cfg]`
- **Backup/restore**: JSON export/import with SHA-256 checksums, schema versioning, and typed `BackupIssue`s
- **Migration**: `MigrationManager` with key renames, deletions, and custom transforms
- **Undo/redo & reset**: Change history with configurable depth, per-field/category/full reset
- **Observable**: `tokio::sync::watch` channels for reactive updates via `SettingsRepository::subscribe()`
- **Lock/PIN protection**: Optional Argon2-based PIN lock via `SettingsLockManager`

## Installation

```toml
[dependencies]
rip-settings = "0.1.1"
rip-settings-derive = "0.1.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["sync", "rt"] }
```

**Requirements:** Rust 1.85+ (edition 2024), `tokio` for async backends.

## Quick Start

### 1. Define your settings

```rust
use rip_settings_derive::{Category, Settings};
use serde::{Serialize, Deserialize};

// Define categories
#[derive(Category)]
#[category(order = 0)]
pub struct General;

#[derive(Category)]
#[category(order = 1, title = "Look & Feel")]
pub struct Appearance;

// Define settings
#[derive(Settings, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[settings(version = 1, app_id = "my_app")]
pub struct AppSettings {
    #[setting(title = "Dark Mode", category = "Appearance", kind = "toggle")]
    pub dark_mode: bool,

    #[setting(
        title = "Font Size",
        category = "Appearance",
        kind = "slider",
        min = 8.0, max = 72.0, step = 1.0,
    )]
    #[validate(range(min = 8.0, max = 72.0))]
    pub font_size: f32,

    #[setting(
        title = "Theme",
        category = "Appearance",
        kind = "dropdown",
        options = "System,Light,Dark,AMOLED"
    )]
    pub theme: String,

    #[setting(
        title = "Username",
        category = "General",
        kind = "text_input"
    )]
    #[validate(length(min = 3, max = 30), required)]
    pub username: String,

    // Persisted but not shown in UI
    #[setting(persisted_only)]
    pub last_sync: Option<i64>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            dark_mode: false,
            font_size: 16.0,
            theme: "System".into(),
            username: String::new(),
            last_sync: None,
        }
    }
}
```

### 2. Create repository

```rust
use rip_settings::prelude::*;

fn main() -> Result<(), SettingsError> {
    // Choose a backend
    let backend = JsonFileBackend::new(default_settings_path("my_app"));
    // let backend = MemoryBackend::new(); // for tests

    let repo = SettingsRepository::<AppSettings>::new(Box::new(backend));

    // Read
    let settings = repo.get();
    println!("Dark mode: {}", settings.dark_mode);

    // Update atomically with validation
    repo.update(|s| s.dark_mode = true)?;

    // Set single field (validates first)
    repo.set_field("font_size", serde_json::json!(20.0))?;

    // Subscribe for reactive updates
    let mut rx = repo.subscribe();
    tokio::spawn(async move {
        while rx.changed().await.is_ok() {
            println!("Settings changed: {:?}", *rx.borrow());
        }
    });

    // Export
    let backup_mgr = SettingsBackupManager::new("my_app", 1);
    let json = backup_mgr.export(&repo.get())?;
    println!("{json}");

    Ok(())
}
```

## Usage

### Supported Field Types

| Rust Type | Storage | `kind` values | Validation |
|-----------|---------|---------------|------------|
| `bool` | JSON | `toggle`, `button` | - |
| `i32`, `i64`, `f32`, `f64` | JSON | `slider`, `dropdown` | `range` |
| `String` | JSON | `text_input`, `dropdown` | `length`, `pattern`, `required` |
| `Option<T>` | JSON (nullable) | any | - |
| `Vec<T>`, `HashMap<K,V>` | JSON serialized | `custom` | - |
| `enum` (`Serialize`) | JSON string | `dropdown` (`options = "A,B,C"`) | - |
| Custom `Serialize` structs | JSON serialized | `custom` | - |

### Field Attributes

```rust
#[setting(
    title = "Font Size",              // Display title
    description = "Adjust font size", // Optional subtitle
    category = "Appearance",          // Category name
    category_order = 1,               // Sort order
    kind = "slider",                  // toggle | slider | dropdown | text_input | button | custom
    key = "font_size",                // Storage key (defaults to snake_case field name)
    min = 8.0, max = 72.0, step = 1.0,// For slider
    options = "A,B,C",               // For dropdown (comma-separated)
    depends_on = "dark_mode",        // Disabled when dependency is falsy
    action = "clear_cache",          // For button
    confirm_title = "Are you sure?", // Confirmation dialog
    confirm_message = "This cannot be undone.",
    is_dangerous,                    // Marks confirmation as destructive
    persisted_only,                  // Hidden from UI, persisted only
    no_reset,                        // Excluded from reset operations
    platforms = "windows,linux"      // Platform visibility filter
)]
```

### Validation

```rust
#[setting(title = "Server URL", category = "General", kind = "text_input")]
#[validate(pattern = "^https?://.*", error = "Must be a valid URL")]
pub server_url: String,

#[setting(title = "Username", category = "General", kind = "text_input")]
#[validate(length(min = 3, max = 30), required, error = "Username must be 3-30 chars")]
pub username: String,

#[setting(title = "Font Size", category = "Appearance", kind = "slider", min = 8.0, max = 72.0)]
#[validate(range(min = 8.0, max = 72.0))]
pub font_size: f32,
```

`set_field` and `update` run validation before persisting; on failure they return `SettingsError::ValidationFailed`.

### Platform-Specific Settings

```rust
#[setting(
    title = "Windows-only Feature",
    category = "General",
    kind = "toggle",
    platforms = "windows" // comma-separated: windows, macos, linux, ios, android
)]
pub win_feature: bool,
```

`field.platforms` lists allowed platforms; the consumer filters via `is_field_enabled`/`visible_ui_fields`.

`default_settings_path("my_app")` uses `#[cfg]` to pick the correct directory:

| Platform | Path |
|----------|------|
| Windows | `%LOCALAPPDATA%\my_app\settings.json` |
| macOS | `~/Library/Application Support/my_app/settings.json` |
| Linux | `$XDG_DATA_HOME/my_app/settings.json` |
| iOS | `~/Documents/my_app/settings.json` |
| Android | `/data/data/my_app/files/settings.json` |

### Field Dependencies

```rust
#[setting(title = "Enable Notifications", category = "General", kind = "toggle")]
pub notifications_enabled: bool,

#[setting(
    title = "Notification Sound",
    category = "General",
    kind = "toggle",
    depends_on = "notifications_enabled" // disabled when above is false
)]
pub notification_sound: bool,
```

Check at runtime:

```rust
let field = settings.field_by_name("notification_sound").unwrap();
let enabled = settings.is_field_enabled(&field);
```

### Confirmation Dialogs

```rust
#[setting(
    title = "Clear Cache",
    category = "System",
    kind = "button",
    action = "clear_cache",
    confirm_title = "Clear Cache",
    confirm_message = "This will remove all cached data. Continue?",
    is_dangerous
)]
pub clear_cache_trigger: bool,
```

Metadata is available via `field.confirmation: Option<ConfirmationConfig>` (title, message, confirm/cancel text, `is_dangerous`).

### Backup & Restore

```rust
use rip_settings::backup::{ImportOptions, MergeMode};

let backup_mgr = SettingsBackupManager::new("my_app", 1)
    .with_device_info(|| DeviceInfo {
        platform: "linux".into(),
        os_version: "6.8".into(),
        app_version: "1.0.0".into(),
        ..Default::default()
    });

// Export (pretty JSON with format_version, schema_version, app_id, checksum, timestamp)
let json = backup_mgr.export(&repo.get())?;

// Validate without importing
let result = backup_mgr.validate(&json);
assert!(result.is_valid);

// Import with options
let imported: AppSettings = backup_mgr.import(&json, &ImportOptions {
    validate_app_id: true,
    validate_checksum: true,
    merge_mode: MergeMode::Overwrite,
})?;
repo.update(|s| *s = imported)?;
```

Import validates `app_id`, `schema_version` (must not be newer), and SHA-256 `checksum`; errors are `SettingsError::AppMismatch`, `VersionTooNew`, `ChecksumMismatch`.

### Migration

```rust
use rip_settings::migration::MigrationManager;

let mut data = serde_json::json!({
    "old_theme_name": "dark",
    "deprecated_field": true
});

let mgr = MigrationManager::new(3)
    .add_key_rename(0, 1, "old_theme_name", "theme")
    .add_key_deletion(1, 2, vec!["deprecated_field".into()])
    .add_migration(2, 3, |json| {
        if let Some(obj) = json.as_object_mut() {
            obj.insert("new_field".into(), serde_json::json!(42));
        }
        Ok(())
    });

mgr.migrate(&mut data)?;
```

### Undo / Redo

```rust
use std::sync::Arc;

let repo = Arc::new(SettingsRepository::<AppSettings>::new(Box::new(MemoryBackend::new())));
let undo = UndoManager::new(repo.clone(), 50); // depth = 50

undo.set_field("font_size", serde_json::json!(24.0))?;
assert!(undo.can_undo());
undo.undo()?;
undo.redo()?;
```

### Reset

```rust
let reset = ResetManager::new(&*repo);
reset.reset_field("font_size")?;          // single field
reset.reset_category("Appearance")?;      // whole category
reset.reset_all()?;                       // all fields (respects `no_reset`)
```

### Lock / PIN Protection

```rust
let lock = SettingsLockManager::new();
lock.enable_lock("1234")?;               // Argon2 hash + random salt
assert!(lock.is_locked());
lock.unlock("1234")?;                    // verifies PIN
lock.set_timeout(300_000);               // auto-lock after 5 min
lock.disable_lock("1234")?;
```

### Observable

```rust
let mut rx = repo.subscribe();           // tokio::sync::watch::Receiver<AppSettings>
while rx.changed().await.is_ok() {
    let current = rx.borrow().clone();
    println!("updated: {current:?}");
}
```

## Repose UI

`settings-repose` (crate `rip-settings-repose`) ports `kmp-settings/settings-ui-compose` to [`repose`](https://github.com/mlm-games/repose) (`../repose` path dependency).

Mirrors the KMP dialogs and `AutoSettingsScreen`:

```
settings-repose/     -- repose UI (depends on ../repose via path)
  src/
    lib.rs
    auto_settings_screen.rs -- AutoSettingsScreen (grouped by category, toggles/sliders/dropdowns)
    components.rs           -- SettingsSection, SettingsToggle, SettingsItem, SettingsAction
    dialogs/
      slider.rs             -- SliderSettingDialog (like kmp SliderSettingDialog.kt)
      dropdown.rs           -- DropdownSettingDialog
      input.rs              -- InputDialog (TextInput)
      selection.rs          -- SelectionDialog
      confirmation.rs       -- SettingConfirmationDialog
      reset.rs              -- ResetDialog (UiOnly/Category/All)
      export.rs             -- ExportSettingsDialog / ImportSettingsDialog
      lock.rs               -- LockDialog (PIN via SettingsLockManager)
      time_picker.rs        -- TimePickerSettingDialog (minutesOfDay)
```

Example:

```rust
use rip_settings::prelude::*;
use rip_settings_repose::AutoSettingsScreen;
use repose_core::signal;
use repose_ui::overlay::OverlayHandle;

let repo = std::sync::Arc::new(SettingsRepository::<AppSettings>::new(Box::new(MemoryBackend::new())));
let snap = repo.get();
let overlay = OverlayHandle::new();
let on_set = std::rc::Rc::new({
    let repo = repo.clone();
    move |name: &str, val: serde_json::Value| { let _ = repo.set_field(name, val); }
});
let view = AutoSettingsScreen(&snap, on_set, overlay, vec![], vec![]);
// Mount `view` inside your repose app's `OverlayHost`.
```

Individual dialogs are also exported for custom layouts (`SliderSettingDialog`, `DropdownSettingDialog`, `InputDialog`, `SelectionDialog`, `ResetDialog`, `ExportSettingsDialog`, `ImportSettingsDialog`, `LockDialog`, `TimePickerSettingDialog`, `SettingConfirmationDialog`).

## Architecture

```
settings-derive/     - proc macro crate (#[derive(Settings)], #[derive(Category)])
settings-core/       - runtime library (repository, backends, validation, backup, migration, etc.)
  src/
    backend.rs       - SettingsBackend trait + JsonFileBackend, MemoryBackend
    field.rs         - FieldMeta, FieldKind, ValidationRules, ConfirmationConfig
    schema.rs        - SettingsSchema trait
    repository.rs    - SettingsRepository with watch channel
    validation.rs    - validation engine
    backup.rs        - SettingsBackupManager, SettingsBundle, checksum
    migration.rs     - MigrationManager, KeyRename, KeyDeletion
    undo.rs          - UndoManager
    reset.rs         - ResetManager
    lock.rs          - SettingsLockManager (Argon2)
    platform.rs      - default_settings_path() via #[cfg]
    error.rs         - SettingsError
settings-repose/     -- repose UI (AutoSettingsScreen + dialogs, path = ../repose)
settings-example/    - example application (cargo run -p settings-example)
```

## Development

```bash
git clone https://github.com/mlm-games/rip-settings.git
cd rip-settings

# Run all tests
cargo test --workspace

# Run the example
cargo run -p settings-example

# Shorthands (from .cargo/config.toml)
cargo test-all        # test --workspace
cargo run-example     # run --package settings-example

# Format and lint
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings

# Build docs
cargo doc --workspace --no-deps --open
```

Tests live alongside source (`#[cfg(test)]`) and in `settings-example/tests/integration_test.rs`.

To publish locally for testing:

```bash
cargo publish -p rip-settings-derive --dry-run
cargo publish -p rip-settings --dry-run
```

## CI / Publishing

CI and releases use reusable workflows from [`mlm-games/ci`](https://github.com/mlm-games/ci) like [`rlobkit`](https://github.com/mlm-games/rlobkit):

- `/.github/workflows/ci.yml` → `mlm-games/ci/.github/workflows/rust-ci.yml@main` (fmt/clippy/test, as in `rlobkit`/`repose`)
- `/.github/workflows/release.yml` → `mlm-games/ci/.github/workflows/crate-publish.yml@main` with `workspace-publish: true` and `workspace-exclude: "settings-example"` (tag `v*.*.*` or manual `workflow_dispatch` with `bump_type`/`mark_prerelease`)

Manual publish:

```bash
# Bump version in Cargo.toml [workspace.package]
cargo publish -p rip-settings-derive
cargo publish -p rip-settings  # derive must be published first
# rip-settings-repose is published together via workspace-publish (exclude example only)
```

Requires `CARGO_REGISTRY_TOKEN` in repository secrets.

## Support

- Issues: [GitHub Issues](https://github.com/mlm-games/rip-settings/issues)
- Discussions: [GitHub Discussions](https://github.com/mlm-games/rip-settings/discussions)

## License

MIT OR Apache-2.0 - See [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE) for details.
