#![allow(non_snake_case)]
use repose_core::*;
use repose_material::material3 as m3;
use repose_ui::{Box, Column, Row, ViewExt};

/// A grouped section with a title, similar to `SettingsSection` in KMP.
pub fn SettingsSection(title: &str, content: View) -> View {
    let header: View = if title.is_empty() {
        Box(Modifier::new().height(0.0))
    } else {
        Box(Modifier::new().padding(8.0)).child(repose_ui::Text(title.to_string()))
    };
    Column(
        Modifier::new()
            .padding(8.0)
            .background(theme().surface_container_high)
            .clip_rounded(12.0),
    )
    .child((header, content))
}

/// A toggle row (Switch) - mirrors KMP's `SettingsToggle`.
pub fn SettingsToggle(
    title: &str,
    description: Option<&str>,
    checked: bool,
    enabled: bool,
    on_checked_change: impl Fn(bool) + 'static,
) -> View {
    let desc = description.unwrap_or("");
    let title_v = Column(Modifier::new().flex_grow(1.0)).child((
        repose_ui::Text(title.to_string()),
        if desc.is_empty() {
            Box(Modifier::new().height(0.0))
        } else {
            repose_ui::Text(desc.to_string())
        },
    ));
    Row(Modifier::new()
        .fill_max_width()
        .padding(12.0)
        .align_items(AlignItems::CENTER)
        .background(Color::TRANSPARENT))
    .child((
        title_v,
        m3::Switch(
            checked,
            on_checked_change,
            m3::SwitchConfig {
                enabled,
                ..Default::default()
            },
        ),
    ))
}

/// A generic clickable row - mirrors KMP's `SettingsItem`.
pub fn SettingsItem(
    title: &str,
    subtitle: &str,
    description: Option<&str>,
    enabled: bool,
    on_click: impl Fn() + 'static,
) -> View {
    let desc = description.unwrap_or("").to_string();
    let sub = if subtitle.is_empty() && desc.is_empty() {
        String::new()
    } else if desc.is_empty() {
        subtitle.to_string()
    } else if subtitle.is_empty() {
        desc
    } else {
        format!("{subtitle} - {desc}")
    };
    let row_content = Column(Modifier::new().flex_grow(1.0)).child((
        repose_ui::Text(title.to_string()),
        if sub.is_empty() {
            Box(Modifier::new().height(0.0))
        } else {
            repose_ui::Text(sub)
        },
    ));
    Box(Modifier::new()
        .fill_max_width()
        .padding(12.0)
        .clickable()
        .on_click(move || {
            if enabled {
                on_click();
            }
        })
        .enabled(enabled))
    .child(
        Row(Modifier::new().align_items(AlignItems::CENTER))
            .child((row_content, repose_ui::Text(">"))),
    )
}

/// An action button row - mirrors KMP's `SettingsAction`.
pub fn SettingsAction(
    title: &str,
    description: Option<&str>,
    enabled: bool,
    on_click: impl Fn() + 'static,
    trailing_content: Option<View>,
) -> View {
    let sub = description.unwrap_or("").to_string();
    let col = Column(Modifier::new().flex_grow(1.0)).child((
        repose_ui::Text(title.to_string()),
        if sub.is_empty() {
            Box(Modifier::new().height(0.0))
        } else {
            repose_ui::Text(sub)
        },
    ));
    let trailing = trailing_content.unwrap_or(Box(Modifier::new()));
    Box(Modifier::new()
        .fill_max_width()
        .padding(12.0)
        .clickable()
        .on_click(move || {
            if enabled {
                on_click();
            }
        })
        .enabled(enabled))
    .child(Row(Modifier::new().align_items(AlignItems::CENTER)).child((col, trailing)))
}
