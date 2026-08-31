#![allow(non_snake_case)]
use repose_core::*;
use repose_material::material3 as m3;
use repose_ui::overlay::OverlayHandle;
use repose_ui::{Box, ViewExt};
use std::rc::Rc;

/// Setting confirmation dialog - mirrors KMP SettingConfirmationDialog.
pub fn SettingConfirmationDialog(
    state: Rc<m3::DialogState>,
    overlay: OverlayHandle,
    title: String,
    message: String,
    _is_dangerous: bool,
    on_confirm: Rc<dyn Fn()>,
    on_dismiss: Rc<dyn Fn()>,
) -> View {
    let confirm = Box(Modifier::new().clickable().on_click({
        let state = state.clone();
        let on_confirm = on_confirm.clone();
        move || {
            on_confirm();
            state.dismiss();
        }
    }))
    .child(repose_ui::Text("Confirm"));
    let dismiss = Box(Modifier::new().clickable().on_click({
        let state = state.clone();
        let on_dismiss = on_dismiss.clone();
        move || {
            on_dismiss();
            state.dismiss();
        }
    }))
    .child(repose_ui::Text("Cancel"));
    m3::AlertDialog(
        state,
        overlay,
        View::from(repose_ui::Text(title)),
        View::from(repose_ui::Text(message)),
        confirm.into(),
        Some(dismiss.into()),
        m3::AlertDialogConfig::default(),
    )
}
