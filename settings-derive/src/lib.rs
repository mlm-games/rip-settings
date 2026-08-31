//! Proc macros for the rip-settings framework.
//!
//! Provides `#[derive(Settings)]` and `#[derive(Category)]`.

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod category;
mod settings;

/// Derive macro for settings structs.
///
/// Generates an implementation of `SettingsSchema` with:
/// - Field metadata from `#[setting(...)]` attributes
/// - Type-safe `get_field_value` / `set_field_value` via serde
/// - `schema_version()` and `app_id()` from `#[settings(...)]`
///
/// # Example
///
/// ```ignore
/// #[derive(Settings, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
/// #[settings(version = 1, app_id = "my_app")]
/// pub struct AppSettings {
///     #[setting(title = "Dark Mode", category = "Appearance", kind = "toggle")]
///     pub dark_mode: bool,
///
///     #[setting(persisted_only)]
///     pub last_sync: Option<i64>,
/// }
/// ```
#[proc_macro_derive(Settings, attributes(settings, setting, validate))]
pub fn derive_settings(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    settings::expand_settings(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Derive macro for category marker structs.
///
/// # Example
///
/// ```ignore
/// #[derive(Category)]
/// #[category(order = 0)]
/// pub struct General;
///
/// #[derive(Category)]
/// #[category(order = 1, title = "Look & Feel")]
/// pub struct Appearance;
/// ```
#[proc_macro_derive(Category, attributes(category))]
pub fn derive_category(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    category::expand_category(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
