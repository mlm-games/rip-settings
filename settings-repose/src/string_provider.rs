#![allow(non_snake_case)]

use std::sync::{Arc, OnceLock, RwLock};

/// String-resource provider - port of KMP `LocalStringResourceProvider` /
/// `StringResourceProvider`.
///
/// KMP uses `CompositionLocal` to inject Android string resources.
/// This crate provides a trait + global that can be overridden for localization.
pub trait StringResourceProvider: Send + Sync + 'static {
    fn get_string(&self, key: &str) -> String {
        key.to_string()
    }
}

pub struct NoOpStringResourceProvider;

impl StringResourceProvider for NoOpStringResourceProvider {}

static GLOBAL_PROVIDER: OnceLock<RwLock<Arc<dyn StringResourceProvider>>> = OnceLock::new();

fn global() -> &'static RwLock<Arc<dyn StringResourceProvider>> {
    GLOBAL_PROVIDER.get_or_init(|| RwLock::new(Arc::new(NoOpStringResourceProvider)))
}

/// Set global string provider (mirrors `ProvideStringResources`).
pub fn set_string_provider<P: StringResourceProvider>(provider: P) {
    *global().write().unwrap() = Arc::new(provider);
}

/// Resolve a string key via the current provider (falls back to key).
pub fn get_string(key: &str) -> String {
    global().read().unwrap().get_string(key)
}

/// Provide a custom string resource provider for the duration of `f`.
/// Mirrors KMP `ProvideStringResources { content }`.
pub fn provide_string_resources<P: StringResourceProvider + Clone>(
    provider: P,
    f: impl FnOnce() -> repose_core::View,
) -> repose_core::View {
    let prev = global().read().unwrap().clone();
    *global().write().unwrap() = Arc::new(provider);
    let v = f();
    *global().write().unwrap() = prev;
    v
}
