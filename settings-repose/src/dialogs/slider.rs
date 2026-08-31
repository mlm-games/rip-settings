#![allow(non_snake_case)]
use repose_core::*;
use repose_material::material3 as m3;
use repose_ui::overlay::OverlayHandle;
use repose_ui::{Box, Column, ViewExt};
use std::rc::Rc;

/// Slider setting dialog - mirrors KMP's `SliderSettingDialog`.
pub fn SliderSettingDialog(
    state: Rc<m3::DialogState>,
    overlay: OverlayHandle,
    title: String,
    current_value: f32,
    min: f32,
    max: f32,
    step: f32,
    on_value_selected: Rc<dyn Fn(f32)>,
) -> View {
    let pending = remember_mutable_with_key(format!("slider_pending_{title}"), || current_value);
    let v = *pending.get();
    let content = Column(Modifier::new().padding(16.0)).child((
        repose_ui::Text(format!("{v:.2}")),
        m3::Slider(
            v,
            (min, max),
            if step > 0.0 { Some(step) } else { None },
            {
                let pending = pending.clone();
                move |nv| pending.set(nv)
            },
            m3::SliderConfig::default(),
        ),
    ));
    let confirm = Box(Modifier::new().clickable().on_click({
        let pending = pending.clone();
        let on_value_selected = on_value_selected.clone();
        let state = state.clone();
        move || {
            on_value_selected(*pending.get());
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
        content.into(),
        confirm.into(),
        Some(dismiss.into()),
        m3::AlertDialogConfig::default(),
    )
}
