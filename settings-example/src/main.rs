use multiplatform_settings_core::prelude::*;
use multiplatform_settings_derive::{Category, Settings};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Category)]
#[category(order = 0)]
pub struct General;

#[derive(Category)]
#[category(order = 1, title = "Look & Feel")]
pub struct Appearance;

#[derive(Category)]
#[category(order = 2)]
pub struct System;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
    Amoled,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub enum SwipeAction {
    #[default]
    None,
    Archive,
    Delete,
    Star,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct SwipeConfig {
    pub left: SwipeAction,
    pub right: SwipeAction,
}

#[derive(Settings, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[settings(version = 2, app_id = "example_app")]
pub struct AppSettings {
    #[setting(
        title = "Dark Mode",
        description = "Enable dark color scheme",
        category = "Appearance",
        category_order = 1,
        kind = "toggle"
    )]
    pub dark_mode: bool,

    #[setting(
        title = "Font Size",
        description = "Adjust the display font size",
        category = "Appearance",
        category_order = 1,
        kind = "slider",
        min = 8.0,
        max = 72.0,
        step = 1.0
    )]
    #[validate(
        range(min = 8.0, max = 72.0),
        error = "Font size must be between 8 and 72"
    )]
    pub font_size: f32,

    #[setting(
        title = "Theme",
        description = "Choose a color theme",
        category = "Appearance",
        category_order = 1,
        kind = "dropdown",
        options = "System,Light,Dark,AMOLED"
    )]
    pub theme: Theme,

    #[setting(
        title = "Username",
        description = "Your display name",
        category = "General",
        category_order = 0,
        kind = "text_input"
    )]
    #[validate(
        length(min = 3, max = 30),
        required,
        error = "Username must be 3-30 characters"
    )]
    pub username: String,

    #[setting(
        title = "Notifications",
        description = "Enable push notifications",
        category = "General",
        category_order = 0,
        kind = "toggle"
    )]
    pub notifications_enabled: bool,

    #[setting(
        title = "Notification Sound",
        description = "Play sound for notifications",
        category = "General",
        category_order = 0,
        kind = "toggle",
        depends_on = "notifications_enabled"
    )]
    pub notification_sound: bool,

    #[setting(
        title = "Clear Cache",
        description = "Remove all cached data",
        category = "System",
        category_order = 2,
        kind = "button",
        action = "clear_cache",
        confirm_title = "Clear Cache",
        confirm_message = "This will remove all cached data. Continue?",
        is_dangerous
    )]
    pub clear_cache_trigger: bool,

    #[setting(
        title = "Swipe Actions",
        description = "Configure swipe gestures",
        category = "General",
        category_order = 0,
        kind = "custom"
    )]
    pub swipe_config: SwipeConfig,

    #[setting(persisted_only)]
    pub last_sync_timestamp: Option<i64>,

    #[setting(persisted_only)]
    pub cached_username: Option<String>,

    #[setting(persisted_only)]
    pub widget_positions: HashMap<String, (i32, i32)>,

    #[setting(persisted_only, no_reset)]
    pub install_id: String,

    #[setting(persisted_only)]
    pub favorite_ids: Vec<i64>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            dark_mode: false,
            font_size: 16.0,
            theme: Theme::System,
            username: String::new(),
            notifications_enabled: true,
            notification_sound: true,
            clear_cache_trigger: false,
            swipe_config: SwipeConfig::default(),
            last_sync_timestamp: None,
            cached_username: None,
            widget_positions: HashMap::new(),
            install_id: String::new(),
            favorite_ids: Vec::new(),
        }
    }
}

fn main() {
    println!("=== Multiplatform Settings Example ===\n");

    let backend = MemoryBackend::new();
    let repo = SettingsRepository::<AppSettings>::new(Box::new(backend));

    let settings = repo.get();
    println!("Initial settings:");
    println!("  dark_mode: {}", settings.dark_mode);
    println!("  font_size: {}", settings.font_size);
    println!("  theme: {:?}", settings.theme);
    println!("  notifications: {}", settings.notifications_enabled);
    println!();

    repo.update(|s| {
        s.dark_mode = true;
        s.font_size = 20.0;
        s.theme = Theme::Dark;
        s.username = "Alice".into();
        s.notifications_enabled = true;
    })
    .unwrap();

    let settings = repo.get();
    println!("After update:");
    println!("  dark_mode: {}", settings.dark_mode);
    println!("  font_size: {}", settings.font_size);
    println!("  theme: {:?}", settings.theme);
    println!("  username: {}", settings.username);
    println!();

    repo.set_field("font_size", serde_json::json!(24.0))
        .unwrap();
    println!("After set_field(font_size=24): {}", repo.get().font_size);
    println!();

    let result = repo.set_field("font_size", serde_json::json!(200.0));
    println!(
        "Validation (font_size=200): {:?}",
        result.err().map(|e| e.to_string())
    );
    println!("  font_size unchanged: {}", repo.get().font_size);
    println!();

    let settings = repo.get();
    println!("Schema info:");
    println!("  version: {}", settings.schema_version());
    println!("  app_id: {}", settings.app_id());
    println!("  total fields: {}", settings.fields().len());
    println!("  UI fields: {}", settings.ui_fields().len());
    println!(
        "  visible UI fields: {}",
        settings.visible_ui_fields().len()
    );
    println!();

    println!("Categories:");
    for (category, fields) in settings.grouped_by_category() {
        println!("  [{}]", category);
        for field in &fields {
            let enabled = settings.is_field_enabled(field);
            let value = settings.get_field_value(field.name).unwrap();
            println!(
                "    {} ({}): {} {}",
                field.title,
                field.name,
                value,
                if !enabled { "[DISABLED]" } else { "" }
            );
        }
    }
    println!();

    println!("Dependency demo:");
    let snd_field = settings.field_by_name("notification_sound").unwrap();
    println!(
        "  notifications_enabled=true → notification_sound enabled: {}",
        settings.is_field_enabled(&snd_field)
    );

    repo.update(|s| s.notifications_enabled = false).unwrap();
    let settings = repo.get();
    let snd_field = settings.field_by_name("notification_sound").unwrap();
    println!(
        "  notifications_enabled=false → notification_sound enabled: {}",
        settings.is_field_enabled(&snd_field)
    );
    println!();

    let backup_mgr = SettingsBackupManager::new("example_app", 2);
    let exported = backup_mgr.export(&repo.get()).unwrap();
    println!("Exported backup ({} bytes):", exported.len());
    println!("  {}", &exported[..exported.len().min(200)]);
    println!();

    repo.update(|s| s.dark_mode = false).unwrap();
    println!("Before import: dark_mode={}", repo.get().dark_mode);

    let imported: AppSettings = backup_mgr
        .import(
            &exported,
            &multiplatform_settings_core::backup::ImportOptions::default(),
        )
        .unwrap();
    repo.update(|s| *s = imported).unwrap();
    println!("After import: dark_mode={}", repo.get().dark_mode);
    println!();

    let mut data = serde_json::json!({
        "old_theme_name": "dark",
        "font_size": 14,
        "deprecated_field": true
    });

    let migration_mgr = MigrationManager::new(3)
        .add_key_rename(0, 1, "old_theme_name", "theme")
        .add_key_deletion(1, 2, vec!["deprecated_field".into()]);

    let result = migration_mgr.migrate(&mut data);
    println!("Migration result: {:?}", result);
    println!("  Migrated data: {}", data);
    println!();

    let repo = Arc::new(SettingsRepository::<AppSettings>::new(Box::new(
        MemoryBackend::new(),
    )));
    let undo_mgr = UndoManager::new(repo.clone(), 20);

    undo_mgr.set_field("font_size", serde_json::json!(30.0)).unwrap();

    println!("Undo/Redo:");
    println!("  font_size after change: {}", repo.get().font_size);
    println!("  can_undo: {}", undo_mgr.can_undo());

    undo_mgr.undo().unwrap();
    println!("  font_size after undo: {}", repo.get().font_size);
    println!("  can_redo: {}", undo_mgr.can_redo());

    undo_mgr.redo().unwrap();
    println!("  font_size after redo: {}", repo.get().font_size);
    println!();

    let reset_mgr = ResetManager::new(&*repo);
    reset_mgr.reset_field("font_size").unwrap();
    println!("After reset font_size: {}", repo.get().font_size);
    println!();

    let lock_mgr = SettingsLockManager::new();
    lock_mgr.enable_lock("1234").unwrap();
    println!("Lock enabled: {}", lock_mgr.is_lock_enabled());
    println!("Is locked: {}", lock_mgr.is_locked());

    lock_mgr.set_timeout(999_999_999);
    lock_mgr.unlock("1234").unwrap();
    println!("After unlock: is_locked={}", lock_mgr.is_locked());

    let wrong_pin = lock_mgr.unlock("0000");
    println!("Wrong PIN result: {:?}", wrong_pin.err());
    println!();

    let path = default_settings_path("example_app");
    println!("Default settings path: {}", path.display());

    println!("\n=== Done ===");
}
