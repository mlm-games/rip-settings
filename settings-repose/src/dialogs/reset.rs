#![allow(non_snake_case)]
use repose_core::*;
use repose_material::material3 as m3;
use repose_ui::overlay::OverlayHandle;
use repose_ui::{Box, Column, ViewExt};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub enum ResetOption {
    UiOnly,
    Category(String),
    All,
}

/// Reset dialog - mirrors KMP's `ResetSettingsDialog`.
pub fn ResetDialog(
    state: Rc<m3::DialogState>,
    overlay: OverlayHandle,
    categories: Vec<String>,
    on_reset: Rc<dyn Fn(ResetOption)>,
) -> View {
    let selected = remember_mutable_with_key("reset_selected".to_string(), || ResetOption::UiOnly);
    let sel_cat = remember_mutable_with_key("reset_sel_cat".to_string(), || String::new());
    let mut col = Column(Modifier::new()).child(repose_ui::Text("Choose what to reset:"));
    let opts = vec![
        (ResetOption::UiOnly, "UI Settings"),
        (ResetOption::All, "All Settings"),
    ];
    for (opt, title) in opts {
        let is_sel = *selected.get() == opt;
        let opt_c = opt.clone();
        let marker = if is_sel { "* " } else { "o " };
        col = col.child(
            Box(Modifier::new()
                .fill_max_width()
                .padding(4.0)
                .clickable()
                .on_click({
                    let selected = selected.clone();
                    move || selected.set(opt_c.clone())
                }))
            .child(repose_ui::Text(format!("{marker}{title}"))),
        );
    }
    for cat in categories {
        let cat_c = cat.clone();
        let is_sel = *sel_cat.get() == cat;
        let marker = if is_sel { "* " } else { "o " };
        col = col.child(
            Box(Modifier::new()
                .fill_max_width()
                .padding(4.0)
                .clickable()
                .on_click({
                    let sel_cat = sel_cat.clone();
                    let cat_c = cat_c.clone();
                    let selected = selected.clone();
                    move || {
                        sel_cat.set(cat_c.clone());
                        selected.set(ResetOption::Category(cat_c.clone()));
                    }
                }))
            .child(repose_ui::Text(format!("{marker}{cat_c}"))),
        );
    }
    let content = col;
    let confirm = Box(Modifier::new().clickable().on_click({
        let selected = selected.clone();
        let sel_cat = sel_cat.clone();
        let state = state.clone();
        let on_reset = on_reset.clone();
        move || {
            let opt = selected.get().clone();
            let opt = match &opt {
                ResetOption::Category(_) if sel_cat.get().is_empty() => ResetOption::UiOnly,
                _ => opt,
            };
            on_reset(opt);
            state.dismiss();
        }
    }))
    .child(repose_ui::Text("Reset"));
    let dismiss = Box(Modifier::new().clickable().on_click({
        let state = state.clone();
        move || state.dismiss()
    }))
    .child(repose_ui::Text("Cancel"));
    m3::AlertDialog(
        state,
        overlay,
        View::from(repose_ui::Text("Reset Settings")),
        content.into(),
        confirm.into(),
        Some(dismiss.into()),
        m3::AlertDialogConfig::default(),
    )
}
