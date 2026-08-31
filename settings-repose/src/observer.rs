#![allow(non_snake_case)]

use std::sync::Arc;

use repose_core::Signal;
use rip_settings::repository::SettingsRepository;
use rip_settings::schema::SettingsSchema;

/// Settings observer utilities - port of KMP `SettingsObserver.kt`.

/// Observe a single field as a reactive `Signal`.
///
/// Mirrors `SettingsRepository<T>.observeFieldAsState` / `observeField` in KMP.
/// Returns a `Signal` seeded with the current field value. In a real repose app,
/// wire this to `repo.subscribe()` via an `effect` to get live updates:
///
/// ```ignore
/// let sig = observe_field::<AppSettings, bool>(repo.clone(), "dark_mode");
/// scoped_effect(move || {
///     let mut rx = repo.subscribe();
///     // poll rx and update sig
/// });
/// ```
pub fn observe_field<T, V>(repo: Arc<SettingsRepository<T>>, field_name: &str) -> Signal<V>
where
    T: SettingsSchema + PartialEq + 'static,
    V: Clone + PartialEq + Send + Sync + serde::de::DeserializeOwned + 'static,
{
    let field = field_name.to_string();
    let initial: V = repo
        .get()
        .get_field_value(&field)
        .and_then(|v| {
            serde_json::from_value(v)
                .map_err(|e| rip_settings::error::SettingsError::Serialization(e))
        })
        .unwrap_or_else(|_| panic!("observe_field: unknown field {field}"));
    // NOTE: live updates require wiring `repo.subscribe()` to this signal via an effect.
    // We intentionally avoid capturing `Signal` in `add_change_listener` (which requires
    // Send+Sync) since `Signal` is `Rc<RefCell<_>>` (!Send). Use the `on_setting_changed`
    // helper or a manual `watch` subscription for live updates.
    Signal::new(initial)
}

/// React to setting changes with a side effect.
///
/// Mirrors `OnSettingChanged` / `FieldChangeListener` in KMP.
/// Returns a guard (currently no-op `Drop`; listeners are stored in the repo).
pub struct SettingChangedGuard<T: SettingsSchema> {
    repo: Arc<SettingsRepository<T>>,
    field_name: String,
}

impl<T: SettingsSchema> Drop for SettingChangedGuard<T> {
    fn drop(&mut self) {}
}

pub fn on_setting_changed<T, F>(
    repo: Arc<SettingsRepository<T>>,
    field_name: &str,
    on_change: F,
) -> SettingChangedGuard<T>
where
    T: SettingsSchema + PartialEq + 'static,
    F: Fn(Option<serde_json::Value>, Option<serde_json::Value>) + Send + Sync + 'static,
{
    let field = field_name.to_string();
    let field2 = field.clone();
    repo.add_change_listener(Box::new(move |changed_field: &str, old: &T, new: &T| {
        if changed_field == field2 {
            let old_v = old.get_field_value(&field2).ok();
            let new_v = new.get_field_value(&field2).ok();
            on_change(old_v, new_v);
        }
    }));
    SettingChangedGuard {
        repo,
        field_name: field_name.to_string(),
    }
}
