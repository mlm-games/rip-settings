#![allow(non_snake_case)]
use repose_core::*;
use repose_material::material3 as m3;
use repose_ui::overlay::OverlayHandle;
use repose_ui::{Box, Column, ViewExt};
use std::rc::Rc;

/// Dropdown setting dialog - mirrors KMP DropdownSettingDialog.
pub fn DropdownSettingDialog(
    state: Rc<m3::DialogState>,
    overlay: OverlayHandle,
    title: String,
    options: Vec<String>,
    selected_index: usize,
    on_option_selected: Rc<dyn Fn(usize)>,
) -> View {
    let items = options
        .iter()
        .enumerate()
        .map(|(idx, opt)| {
            let label = opt.clone();
            let is_sel = idx == selected_index;
            Box(Modifier::new()
                .fill_max_width()
                .padding(8.0)
                .clickable()
                .on_click({
                    let state = state.clone();
                    let cb = on_option_selected.clone();
                    move || {
                        cb(idx);
                        state.dismiss();
                    }
                }))
            .child(Column(Modifier::new()).child((
                repose_ui::Text(if is_sel { "* " } else { "o " }),
                repose_ui::Text(label),
            )))
        })
        .collect::<Vec<View>>();
    let mut col = Column(Modifier::new());
    for v in items {
        col = col.child(v);
    }
    let content = col;
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
        Box(Modifier::new()).into(),
        Some(dismiss.into()),
        m3::AlertDialogConfig::default(),
    )
}
