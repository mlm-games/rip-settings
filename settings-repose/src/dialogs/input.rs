#![allow(non_snake_case)]
use repose_core::*;
use repose_material::material3 as m3;
use repose_ui::overlay::OverlayHandle;
use repose_ui::{Box, ViewExt};
use std::rc::Rc;

/// Input dialog - mirrors KMP's `InputDialog`.
pub fn InputDialog(
    state: Rc<m3::DialogState>,
    overlay: OverlayHandle,
    title: String,
    initial_value: String,
    on_confirm: Rc<dyn Fn(String)>,
) -> View {
    let text = remember_mutable_with_key(format!("input_{title}"), || initial_value.clone());
    let field = m3::OutlinedTextField(
        Modifier::new().fill_max_width(),
        text.get().to_string(),
        {
            let text = text.clone();
            move |s: String| text.set(s)
        },
        m3::OutlinedTextFieldConfig {
            label: Some("Value".into()),
            placeholder: Some("Enter value".into()),
            ..Default::default()
        },
    );
    let confirm = Box(Modifier::new().clickable().on_click({
        let text = text.clone();
        let state = state.clone();
        let on_confirm = on_confirm.clone();
        move || {
            on_confirm(text.get().to_string());
            state.dismiss();
        }
    }))
    .child(repose_ui::Text("OK"));
    let dismiss = Box(Modifier::new().clickable().on_click({
        let state = state.clone();
        move || state.dismiss()
    }))
    .child(repose_ui::Text("Cancel"));
    m3::AlertDialog(
        state,
        overlay,
        View::from(repose_ui::Text(title)),
        field.into(),
        confirm.into(),
        Some(dismiss.into()),
        m3::AlertDialogConfig::default(),
    )
}
