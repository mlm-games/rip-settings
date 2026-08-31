//! # rip-settings-repose
//!
//! Repose UI for `rip-settings` - ported from `kmp-settings/settings-ui-compose`.
//!
//! Provides `AutoSettingsScreen` and dialogs (`SliderSettingDialog`, `DropdownSettingDialog`,
//! `InputDialog`, `SelectionDialog`, `ResetDialog`, `ExportSettingsDialog`, `LockDialog`,
//! `SettingConfirmationDialog`, `TimePickerSettingDialog`) backed by `repose-material`.
//!
//! The API mirrors `kmp-settings`'s `AutoSettingsScreen` but uses `repose`'s
//! `View`/`Signal` reactive model instead of Compose `State`.

#![allow(non_snake_case)]

pub mod auto_settings_screen;
pub mod components;
pub mod dialogs;
pub mod observer;
pub mod string_provider;

pub use auto_settings_screen::{AutoSettingsScreen, CategoryConfig, CustomTypeHandler};
pub use components::*;
pub use dialogs::*;
pub use observer::{observe_field, on_setting_changed};
pub use string_provider::{
    StringResourceProvider, get_string, provide_string_resources, set_string_provider,
};
