# rip-settings

Type-safe settings management for Rust with declarative schema generation.

[![Crates.io](https://img.shields.io/crates/v/rip-settings)](https://crates.io/crates/rip-settings)
[![docs.rs](https://img.shields.io/docsrs/rip-settings)](https://docs.rs/rip-settings)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

> Rust port of [kmp-settings](https://github.com/mlm-games/kmp-settings).

## Features

- `#[derive(Settings)]` generates schema, validation, and `SettingsSchema` impl
- Pluggable backends (`JsonFileBackend`, `MemoryBackend`)
- Validation, backup/restore (checksum + versioning), migration, undo/redo, reset, PIN lock
- `settings-repose` - `AutoSettingsScreen` + dialogs ported from `kmp-settings/settings-ui-compose` via `../repose`

## Installation

```toml
[dependencies]
rip-settings = "0.1.1"
rip-settings-derive = "0.1.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["sync", "rt"] }
```

Optional UI:

```toml
rip-settings-repose = { version = "0.1.1", path = "settings-repose" } # requires ../repose checkout
```

Requires Rust 1.85+ (edition 2024).

## Quick Start

```rust
use rip_settings::prelude::*;
use rip_settings_derive::{Category, Settings};
use serde::{Deserialize, Serialize};

#[derive(Category)]
#[category(order = 0)]
pub struct General;

#[derive(Settings, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[settings(version = 1, app_id = "my_app")]
pub struct AppSettings {
    #[setting(title = "Dark Mode", category = "General", kind = "toggle")]
    pub dark_mode: bool,
    #[setting(title = "Username", category = "General", kind = "text_input")]
    #[validate(length(min = 3, max = 30), required)]
    pub username: String,
    #[setting(persisted_only)]
    pub last_sync: Option<i64>,
}

impl Default for AppSettings {
    fn default() -> Self { Self { dark_mode: false, username: String::new(), last_sync: None } }
}

fn main() -> Result<(), SettingsError> {
    let repo = SettingsRepository::<AppSettings>::new(Box::new(MemoryBackend::new()));
    repo.update(|s| s.dark_mode = true)?;
    repo.set_field("username", serde_json::json!("Alice"))?;
    let json = SettingsBackupManager::new("my_app", 1).export(&repo.get())?;
    println!("{json}");
    Ok(())
}
```

Repose UI:

```rust
use rip_settings_repose::AutoSettingsScreen;
use repose_ui::overlay::OverlayHandle;
use std::rc::Rc;

let snap = repo.get();
let overlay = OverlayHandle::new();
let view = AutoSettingsScreen(&snap, Rc::new(|n, v| { let _ = repo.set_field(n, v); }), overlay, vec![], vec![]);
```

See `settings-example/` and `settings-repose/src/dialogs/` for all dialogs (Slider, Dropdown, Input, Reset, Export/Import, Lock, TimePicker).

## License

MIT OR Apache-2.0 - [LICENSE](LICENSE)
