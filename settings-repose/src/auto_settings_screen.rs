#![allow(non_snake_case)]
use crate::components::{SettingsAction, SettingsItem, SettingsSection, SettingsToggle};
use crate::dialogs::*;
use repose_core::*;
use repose_material::material3 as m3;
use repose_ui::overlay::OverlayHandle;
use repose_ui::{Box, Column, ViewExt};
use rip_settings::field::{FieldKind, FieldMeta};
use rip_settings::schema::SettingsSchema;
use rip_settings::validation;
use std::rc::Rc;

/// Custom renderer for a field kind not handled by the built-in palette.
pub struct CustomTypeHandler<T> {
    pub type_name: String,
    pub render: Rc<dyn Fn(&FieldMeta, &T, bool, Rc<dyn Fn(&str, serde_json::Value)>) -> View>,
}

/// Category display configuration (mirrors KMP's `CategoryConfig`).
#[derive(Clone)]
pub struct CategoryConfig {
    pub category: String,
    pub title: String,
}

/// Auto-generated settings screen from a `SettingsSchema`.
pub fn AutoSettingsScreen<T: SettingsSchema>(
    value: &T,
    on_set: Rc<dyn Fn(&str, serde_json::Value)>,
    overlay: OverlayHandle,
    category_configs: Vec<CategoryConfig>,
    custom_handlers: Vec<CustomTypeHandler<T>>,
) -> View {
    let groups = value.grouped_by_category();
    let title_map: std::collections::HashMap<String, String> = category_configs
        .into_iter()
        .map(|c| (c.category, c.title))
        .collect();
    let mut category_views: Vec<View> = Vec::new();
    for (category, fields) in groups {
        let display_title = title_map
            .get(&category)
            .cloned()
            .unwrap_or_else(|| category.clone());
        let mut field_views: Vec<View> = Vec::new();
        for field in fields {
            let enabled = value.is_field_enabled(&field);
            let title = field.title;
            let desc = if field.description.is_empty() {
                None
            } else {
                Some(field.description)
            };
            let handler_match = custom_handlers.iter().find(|h| match &field.kind {
                FieldKind::Custom { type_name } => *type_name == h.type_name,
                _ => false,
            });
            if let Some(handler) = handler_match {
                let view = (handler.render)(&field, value, enabled, on_set.clone());
                field_views.push(view);
                continue;
            }
            let fv: View = match &field.kind {
                FieldKind::Toggle => {
                    let raw = value
                        .get_field_value(field.name)
                        .unwrap_or(serde_json::Value::Bool(false));
                    let checked = raw.as_bool().unwrap_or(false);
                    let name = field.name.to_string();
                    let on_set_c = on_set.clone();
                    let validation_rules = field.validation.clone();
                    let confirm = field.confirmation.clone();
                    let ov = overlay.clone();
                    SettingsToggle(title, desc, checked, enabled, move |new_val| {
                        let json = serde_json::Value::Bool(new_val);
                        if let Some(rules) = validation_rules.clone() {
                            if let validation::ValidationResult::Invalid(msg) =
                                validation::validate_value(&json, &rules)
                            {
                                log::warn!("validation failed: {msg}");
                                return;
                            }
                        }
                        if let Some(cfg) = confirm.clone() {
                            let state = Rc::new(m3::DialogState::new());
                            let _dlg = SettingConfirmationDialog(
                                state.clone(),
                                ov.clone(),
                                cfg.title.clone(),
                                cfg.message.clone(),
                                cfg.is_dangerous,
                                {
                                    let on_set_c = on_set_c.clone();
                                    let name = name.clone();
                                    let json = json.clone();
                                    Rc::new(move || on_set_c(&name, json.clone()))
                                },
                                Rc::new(|| {}),
                            );
                            state.show();
                        } else {
                            on_set_c(&name, json);
                        }
                    })
                }
                FieldKind::Dropdown { options } => {
                    let subtitle = options.join(", ");
                    let name = field.name.to_string();
                    let on_set_c = on_set.clone();
                    let opts = options.clone();
                    let ov = overlay.clone();
                    let f_title = field.title.to_string();
                    SettingsItem(title, &subtitle, desc, enabled, move || {
                        let state = Rc::new(m3::DialogState::new());
                        let _dlg = DropdownSettingDialog(
                            state.clone(),
                            ov.clone(),
                            f_title.clone(),
                            opts.clone(),
                            0,
                            {
                                let on_set_c = on_set_c.clone();
                                let name = name.clone();
                                let opts = opts.clone();
                                Rc::new(move |idx: usize| {
                                    if let Some(opt) = opts.get(idx) {
                                        on_set_c(&name, serde_json::Value::String(opt.clone()));
                                    }
                                })
                            },
                        );
                        state.show();
                    })
                }
                FieldKind::Slider { min, max, step } => {
                    let raw = value
                        .get_field_value(field.name)
                        .unwrap_or(serde_json::json!(min));
                    let cur = raw.as_f64().unwrap_or(*min as f64) as f32;
                    let subtitle = format!("{cur}");
                    let name = field.name.to_string();
                    let on_set_c = on_set.clone();
                    let ov = overlay.clone();
                    let f_title = field.title.to_string();
                    let min_c = *min as f32;
                    let max_c = *max as f32;
                    let step_c = *step as f32;
                    SettingsItem(title, &subtitle, desc, enabled, move || {
                        let state = Rc::new(m3::DialogState::new());
                        let _dlg = SliderSettingDialog(
                            state.clone(),
                            ov.clone(),
                            f_title.clone(),
                            cur,
                            min_c,
                            max_c,
                            step_c,
                            {
                                let on_set_c = on_set_c.clone();
                                let name = name.clone();
                                Rc::new(move |v: f32| on_set_c(&name, serde_json::json!(v)))
                            },
                        );
                        state.show();
                    })
                }
                FieldKind::Button { action } => {
                    let act = action.clone();
                    SettingsAction(
                        title,
                        desc,
                        enabled,
                        move || {
                            log::info!("action: {act}");
                        },
                        None,
                    )
                }
                FieldKind::TextInput => {
                    let raw = value
                        .get_field_value(field.name)
                        .unwrap_or(serde_json::Value::String(String::new()));
                    let cur = raw.as_str().unwrap_or("").to_string();
                    let subtitle = if cur.is_empty() {
                        "(empty)".to_string()
                    } else {
                        cur.clone()
                    };
                    let name = field.name.to_string();
                    let on_set_c = on_set.clone();
                    let ov = overlay.clone();
                    let f_title = field.title.to_string();
                    SettingsItem(title, &subtitle, desc, enabled, move || {
                        let state = Rc::new(m3::DialogState::new());
                        let _dlg = InputDialog(
                            state.clone(),
                            ov.clone(),
                            f_title.clone(),
                            cur.clone(),
                            Rc::new({
                                let on_set_c = on_set_c.clone();
                                let name = name.clone();
                                move |new_val: String| {
                                    on_set_c(&name, serde_json::Value::String(new_val))
                                }
                            }),
                        );
                        state.show();
                    })
                }
                FieldKind::Custom { type_name } => {
                    SettingsItem(title, type_name, desc, enabled, || {})
                }
            };
            field_views.push(fv);
        }
        let mut inner = Column(Modifier::new());
        for v in field_views {
            inner = inner.child(v);
        }
        let section_content = inner;
        let header = Box(Modifier::new().padding(8.0)).child(repose_ui::Text(display_title));
        category_views.push(header);
        category_views.push(SettingsSection("", section_content));
    }
    let mut outer = Column(Modifier::new().fill_max_size().padding(16.0));
    for v in category_views {
        outer = outer.child(v);
    }
    outer
}
