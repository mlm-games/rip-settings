#![allow(non_snake_case)]
use repose_core::*;
use repose_material::material3 as m3;
use repose_ui::overlay::OverlayHandle;
use repose_ui::{Box, Column, ViewExt};
use std::rc::Rc;

fn format_minutes(total: i32) -> String {
    let clamped = total.clamp(0, 1439);
    let h = clamped / 60;
    let m = clamped % 60;
    format!("{h:02}:{m:02}")
}

/// Time picker dialog - mirrors KMP's `TimePickerSettingDialog`.
pub fn TimePickerSettingDialog(
    state: Rc<m3::DialogState>,
    overlay: OverlayHandle,
    current_minutes: i32,
    on_time_selected: Rc<dyn Fn(i32)>,
) -> View {
    let pending = remember_mutable_with_key("time_picker_pending".to_string(), || current_minutes);
    let hour = (*pending.get() / 60) as u32;
    let minute = (*pending.get() % 60) as u32;
    let picker_state = Rc::new(m3::TimePickerState::new(hour, minute));
    let picker = m3::TimePicker(
        picker_state.clone(),
        {
            let pending = pending.clone();
            Rc::new(move |h: u32, m: u32| pending.set((h * 60 + m) as i32))
        },
        {
            let pending = pending.clone();
            let on_time_selected = on_time_selected.clone();
            let state = state.clone();
            Rc::new(move || {
                on_time_selected(*pending.get());
                state.dismiss();
            })
        },
        m3::TimePickerConfig::default(),
    );
    let content = Column(Modifier::new())
        .child(repose_ui::Text(format_minutes(*pending.get())))
        .child(picker);
    let confirm = Box(Modifier::new().clickable().on_click({
        let pending = pending.clone();
        let state = state.clone();
        let on_time_selected = on_time_selected.clone();
        move || {
            on_time_selected(*pending.get());
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
        View::from(repose_ui::Text("Select Time")),
        content.into(),
        confirm.into(),
        Some(dismiss.into()),
        m3::AlertDialogConfig::default(),
    )
}
