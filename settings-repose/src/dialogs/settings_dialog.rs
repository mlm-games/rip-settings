#![allow(non_snake_case)]

use std::rc::Rc;

use repose_core::*;
use repose_material::material3 as m3;
use repose_ui::overlay::OverlayHandle;
use repose_ui::{Box, ViewExt};

/// Generic settings dialog - port of KMP SettingsDialog.
pub fn SettingsDialog(
    state: Rc<m3::DialogState>,
    overlay: OverlayHandle,
    title: Option<String>,
    _on_dismiss: Rc<dyn Fn()>,
    confirm_button: View,
    dismiss_button: Option<View>,
    content: View,
) -> View {
    let title_view: View = match title {
        Some(t) => repose_ui::Text(t),
        None => Box(Modifier::new()),
    };
    let body = Box(Modifier::new().fill_max_width()).child(content);
    let confirm_wrapper = Box(Modifier::new()).child(confirm_button);
    let dismiss_wrapper = dismiss_button.map(|v| Box(Modifier::new()).child(v));
    m3::AlertDialog(
        state,
        overlay,
        title_view,
        body,
        confirm_wrapper,
        dismiss_wrapper,
        m3::AlertDialogConfig::default(),
    )
}
