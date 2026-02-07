//! The core trait that the derive macro implements.

use crate::error::SettingsError;
use crate::field::FieldMeta;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Trait implemented by `#[derive(Settings)]`.
///
/// Provides schema introspection, per-field get/set via `serde_json::Value`,
/// and utility methods for UI grouping, dependency checking, etc.
pub trait SettingsSchema:
    Serialize + for<'de> Deserialize<'de> + Default + Clone + Send + Sync + 'static
{
    /// Schema version number.
    fn schema_version(&self) -> u32 {
        1
    }

    /// Application identifier.
    fn app_id(&self) -> &'static str {
        ""
    }

    /// All field metadata.
    fn fields(&self) -> Vec<FieldMeta>;

    /// Get a field's current value as JSON.
    fn get_field_value(&self, name: &str) -> Result<serde_json::Value, SettingsError>;

    /// Set a field's value from JSON.
    fn set_field_value(
        &mut self,
        name: &str,
        value: serde_json::Value,
    ) -> Result<(), SettingsError>;

    /// Get metadata for a specific field.
    fn field_by_name(&self, name: &str) -> Option<FieldMeta> {
        self.fields().into_iter().find(|f| f.name == name)
    }

    /// Get metadata for a specific persistence key.
    fn field_by_key(&self, key: &str) -> Option<FieldMeta> {
        self.fields().into_iter().find(|f| f.key == key)
    }

    /// Fields that have UI representation.
    fn ui_fields(&self) -> Vec<FieldMeta> {
        self.fields()
            .into_iter()
            .filter(|f| !f.is_persisted_only)
            .collect()
    }

    /// UI fields visible on the current platform.
    fn visible_ui_fields(&self) -> Vec<FieldMeta> {
        self.ui_fields()
            .into_iter()
            .filter(|f| f.is_visible_on_current_platform())
            .collect()
    }

    /// Group visible UI fields by category, sorted by category order.
    fn grouped_by_category(&self) -> Vec<(String, Vec<FieldMeta>)> {
        let fields = self.visible_ui_fields();
        let mut groups: BTreeMap<(i32, String), Vec<FieldMeta>> = BTreeMap::new();

        for field in fields {
            let key = (field.category_order, field.category.to_string());
            groups.entry(key).or_default().push(field);
        }

        groups
            .into_iter()
            .map(|((_, cat), fields)| (cat, fields))
            .collect()
    }

    /// Get ordered list of category names.
    fn ordered_categories(&self) -> Vec<String> {
        self.grouped_by_category()
            .into_iter()
            .map(|(cat, _)| cat)
            .collect()
    }

    /// Check if a field is enabled based on its dependency.
    fn is_field_enabled(&self, field: &FieldMeta) -> bool {
        let Some(dep_name) = field.depends_on else {
            return true;
        };

        match self.get_field_value(dep_name) {
            Ok(v) => match v {
                serde_json::Value::Bool(b) => b,
                serde_json::Value::Number(n) => n.as_f64().is_some_and(|n| n != 0.0),
                serde_json::Value::String(s) => !s.is_empty(),
                serde_json::Value::Null => false,
                _ => true,
            },
            Err(_) => true,
        }
    }

    /// Fields that can be reset (not marked no_reset).
    fn resettable_fields(&self) -> Vec<FieldMeta> {
        self.fields().into_iter().filter(|f| !f.no_reset).collect()
    }

    /// Resettable fields in a specific category.
    fn resettable_fields_in_category(&self, category: &str) -> Vec<FieldMeta> {
        self.ui_fields()
            .into_iter()
            .filter(|f| f.category == category && !f.no_reset)
            .collect()
    }
}
