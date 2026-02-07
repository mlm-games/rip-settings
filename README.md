# multiplatform-settings-rs

A settings framework for Rust (adapted from my kotlin version) with:

- **Derive macro** — `#[derive(Settings)]` generates schema, field metadata, and type-safe accessors
- **Pluggable backends** — JSON file, in-memory, or implement your own
- **Validation** — range, length, pattern, required, with custom validators
- **Backup/restore** — export/import settings with checksums and version checks
- **Migration** — schema versioning with key renames, deletions, and custom transforms
- **Undo/redo** — change history with configurable depth
- **Reset** — per-field, per-category, or full reset to defaults
- **Platform-aware storage** — automatic platform-appropriate paths via `#[cfg]`
- **Observable** — `tokio::sync::watch` channels for reactive updates
- **Lock/PIN protection** — optional PIN-based settings lock

## Quick Start

```rust
use multiplatform_settings::prelude::*;
use serde::{Serialize, Deserialize};

// Define categories
#[derive(Category)]
#[category(order = 0)]
pub struct General;

#[derive(Category)]
#[category(order = 1)]
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

    #[setting(title = "Theme", category = "Appearance", kind = "dropdown",
              options = "System,Light,Dark,AMOLED")]
    pub theme: String,

    #[setting(persisted_only)]
    pub last_sync: Option<i64>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            dark_mode: false,
            font_size: 16.0,
            theme: "System".into(),
            last_sync: None,
        }
    }
}

fn main() {
    let backend = JsonFileBackend::new(default_settings_path("my_app"));
    let repo = SettingsRepository::<AppSettings>::new(Box::new(backend));

    // Read
    let settings = repo.get();
    println!("Dark mode: {}", settings.dark_mode);

    // Update
    repo.update(|s| s.dark_mode = true).unwrap();

    // Set single field
    repo.set_field("font_size", serde_json::json!(20.0)).unwrap();

    // Export
    let backup = repo.export("my_app", 1).unwrap();
    println!("{}", backup);
}
```

## Architecture

```
settings-derive/     — proc macro crate (#[derive(Settings)], #[derive(Category)])
settings-core/       — runtime library (repository, backends, validation, backup, migration, etc.)
settings-example/    — example application
```
