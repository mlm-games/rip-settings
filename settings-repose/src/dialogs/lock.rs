#![allow(non_snake_case)]
use repose_core::*;
use repose_material::material3 as m3;
use repose_ui::overlay::OverlayHandle;
use repose_ui::{Box, Column, ViewExt};
use rip_settings::lock::SettingsLockManager;
use std::rc::Rc;

/// Simple unlock-only dialog - thin wrapper over SettingsLockDialog.
pub fn LockDialog(
    state: Rc<m3::DialogState>,
    overlay: OverlayHandle,
    lock_manager: Rc<SettingsLockManager>,
    on_unlocked: Rc<dyn Fn()>,
) -> View {
    SettingsLockDialog(state, overlay, lock_manager, false, on_unlocked, Rc::new(|| {}))
}

/// Full lock dialog - port of KMP SettingsLockDialog.
pub fn SettingsLockDialog(
    state: Rc<m3::DialogState>,
    overlay: OverlayHandle,
    lock_manager: Rc<SettingsLockManager>,
    is_setting_pin: bool,
    on_success: Rc<dyn Fn()>,
    on_dismiss: Rc<dyn Fn()>,
) -> View {
    let pin = remember_mutable_with_key("lock_pin".to_string(), || String::new());
    let confirm_pin = remember_mutable_with_key("lock_confirm".to_string(), || String::new());
    let error = remember_mutable_with_key("lock_err_full".to_string(), || Option::<String>::None);
    let is_processing = remember_mutable_with_key("lock_proc".to_string(), || false);
    let pin_field = m3::OutlinedTextField(
        Modifier::new().fill_max_width(),
        pin.get().to_string(),
        {
            let pin = pin.clone();
            let error = error.clone();
            move |s: String| {
                if s.len() <= 6 {
                    pin.set(s);
                    error.set(None);
                }
            }
        },
        m3::OutlinedTextFieldConfig {
            label: Some("PIN".into()),
            placeholder: Some("****".into()),
            ..Default::default()
        },
    );
    let confirm_field = if is_setting_pin {
        m3::OutlinedTextField(
            Modifier::new().fill_max_width(),
            confirm_pin.get().to_string(),
            {
                let confirm_pin = confirm_pin.clone();
                let error = error.clone();
                move |s: String| {
                    if s.len() <= 6 {
                        confirm_pin.set(s);
                        error.set(None);
                    }
                }
            },
            m3::OutlinedTextFieldConfig {
                label: Some("Confirm PIN".into()),
                placeholder: Some("****".into()),
                ..Default::default()
            },
        )
    } else {
        Box(Modifier::new()).into()
    };
    let mut content_col = Column(Modifier::new().padding(8.0))
        .child(pin_field)
        .child(confirm_field);
    if let Some(err) = error.get().clone() {
        content_col = content_col.child(repose_ui::Text(err));
    }
    let pin_c = pin.clone();
    let confirm_pin_c = confirm_pin.clone();
    let error_c = error.clone();
    let is_processing_c = is_processing.clone();
    let lock_manager_c = lock_manager.clone();
    let on_success_c = on_success.clone();
    let state_c = state.clone();
    let confirm_action = {
        let pin = pin_c.clone();
        let confirm_pin = confirm_pin_c.clone();
        let error = error_c.clone();
        let is_processing = is_processing_c.clone();
        let lock_manager = lock_manager_c.clone();
        let on_success = on_success_c.clone();
        let state = state_c.clone();
        move || {
            if *is_processing.get() {
                return;
            }
            is_processing.set(true);
            let p = pin.get().to_string();
            if is_setting_pin {
                let cp = confirm_pin.get().to_string();
                if p.len() < 4 {
                    error.set(Some("PIN must be at least 4 digits".into()));
                } else if p != cp {
                    error.set(Some("PINs don't match".into()));
                } else {
                    match lock_manager.enable_lock(&p) {
                        Ok(()) => {
                            error.set(None);
                            on_success();
                            state.dismiss();
                        }
                        Err(e) => error.set(Some(e.to_string())),
                    }
                }
            } else {
                match lock_manager.unlock(&p) {
                    Ok(()) => {
                        error.set(None);
                        on_success();
                        state.dismiss();
                    }
                    Err(_e) => error.set(Some("Invalid PIN".into())),
                }
            }
            is_processing.set(false);
        }
    };
    let confirm_enabled = !*is_processing.get() && !pin.get().is_empty();
    let confirm = Box(Modifier::new()
        .clickable()
        .enabled(confirm_enabled)
        .on_click(confirm_action))
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
        View::from(repose_ui::Text(if is_setting_pin { "Set PIN" } else { "Enter PIN" })),
        content_col.into(),
        confirm.into(),
        Some(dismiss.into()),
        m3::AlertDialogConfig::default(),
    )
}
