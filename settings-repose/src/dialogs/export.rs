#![allow(non_snake_case)]
use std::rc::Rc;
use repose_core::*;
use repose_material::material3 as m3;
use repose_ui::overlay::OverlayHandle;
use repose_ui::{Box, Column, ViewExt};
use rip_settings::backup::{BackupValidationResult, SettingsBackupManager};
use rip_settings::schema::SettingsSchema;

/// Export settings dialog - mirrors KMP ExportSettingsDialog.
pub fn ExportSettingsDialog<T: SettingsSchema>(
    state: Rc<m3::DialogState>,
    overlay: OverlayHandle,
    backup_manager: Rc<SettingsBackupManager>,
    settings: T,
    on_export: Rc<dyn Fn(String)>,
) -> View {
    let exported = backup_manager.export(&settings).unwrap_or_else(|e| format!("export failed: {e}"));
    let size = exported.len();
    let content = Column(Modifier::new())
        .child(repose_ui::Text("Settings exported successfully!"))
        .child(repose_ui::Text(format!("Size: {size} bytes")));
    let confirm = Box(Modifier::new().clickable().on_click({
        let state = state.clone();
        let on_export = on_export.clone();
        let exported = exported.clone();
        move || { on_export(exported.clone()); state.dismiss(); }
    })).child(repose_ui::Text("Share"));
    let dismiss = Box(Modifier::new().clickable().on_click({
        let state = state.clone();
        move || state.dismiss()
    })).child(repose_ui::Text("Cancel"));
    m3::AlertDialog(
        state,
        overlay,
        View::from(repose_ui::Text("Export Settings")),
        content.into(),
        confirm.into(),
        Some(dismiss.into()),
        m3::AlertDialogConfig::default(),
    )
}

/// Import settings dialog - mirrors KMP ImportSettingsDialog.
pub fn ImportSettingsDialog(
    state: Rc<m3::DialogState>,
    overlay: OverlayHandle,
    backup_manager: Rc<SettingsBackupManager>,
    json_content: String,
    on_import_complete: Rc<dyn Fn(BackupValidationResult)>,
) -> View {
    let validation = backup_manager.validate(&json_content);
    let issues_text = if validation.is_valid {
        format!("Ready to import {} settings", validation.settings_count)
    } else {
        let mut s = String::from("Validation issues:\n");
        for issue in &validation.issues {
            s.push_str(&format!("* {issue:?}\n"));
        }
        s
    };
    let content = Column(Modifier::new()).child(repose_ui::Text(issues_text));
    let can_import = validation.is_valid;
    let confirm: View = if can_import {
        Box(Modifier::new().clickable().on_click({
            let state = state.clone();
            let on_import_complete = on_import_complete.clone();
            let validation = validation.clone();
            move || { on_import_complete(validation.clone()); state.dismiss(); }
        })).child(repose_ui::Text("Import")).into()
    } else {
        Box(Modifier::new().clickable().on_click({
            let state = state.clone();
            move || state.dismiss()
        })).child(repose_ui::Text("Done")).into()
    };
    let dismiss: Option<View> = if can_import {
        Some(Box(Modifier::new().clickable().on_click({
            let state = state.clone();
            move || state.dismiss()
        })).child(repose_ui::Text("Cancel")).into())
    } else { None };
    m3::AlertDialog(
        state,
        overlay,
        View::from(repose_ui::Text("Import Settings")),
        content.into(),
        confirm,
        dismiss,
        m3::AlertDialogConfig::default(),
    )
}
