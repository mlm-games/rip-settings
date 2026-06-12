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

#[derive(Settings, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[settings(version = 1, app_id = "test_app")]
struct TestSettings {
    #[setting(title = "Enabled", category = "General", kind = "toggle")]
    enabled: bool,

    #[setting(
        title = "Volume",
        category = "Audio",
        category_order = 1,
        kind = "slider",
        min = 0.0,
        max = 100.0,
        step = 1.0
    )]
    #[validate(range(min = 0.0, max = 100.0))]
    volume: f32,

    #[setting(title = "Name", category = "General", kind = "text_input")]
    #[validate(length(min = 1, max = 50), required)]
    name: String,

    #[setting(persisted_only)]
    internal_counter: i32,

    #[setting(
        title = "Advanced",
        category = "General",
        kind = "toggle",
        depends_on = "enabled"
    )]
    advanced: bool,

    #[setting(title = "Critical", category = "General", kind = "toggle", no_reset)]
    critical_flag: bool,
}

#[test]
fn test_schema_fields() {
    let settings = TestSettings::default();
    let fields = settings.fields();

    assert_eq!(fields.len(), 6);

    let enabled_field = settings.field_by_name("enabled").unwrap();
    assert_eq!(enabled_field.title, "Enabled");
    assert_eq!(enabled_field.category, "General");
    assert!(!enabled_field.is_persisted_only);

    let internal_field = settings.field_by_name("internal_counter").unwrap();
    assert!(internal_field.is_persisted_only);
}

#[test]
fn test_ui_fields() {
    let settings = TestSettings::default();
    let ui = settings.ui_fields();

    assert_eq!(ui.len(), 5);
    assert!(ui.iter().all(|f| f.name != "internal_counter"));
}

#[test]
fn test_get_set_field_value() {
    let mut settings = TestSettings::default();

    assert_eq!(
        settings.get_field_value("enabled").unwrap(),
        serde_json::json!(false)
    );

    settings
        .set_field_value("enabled", serde_json::json!(true))
        .unwrap();
    assert!(settings.enabled);

    settings
        .set_field_value("volume", serde_json::json!(75.0))
        .unwrap();
    assert_eq!(settings.volume, 75.0);

    settings
        .set_field_value("name", serde_json::json!("Hello"))
        .unwrap();
    assert_eq!(settings.name, "Hello");
}

#[test]
fn test_unknown_field() {
    let mut settings = TestSettings::default();
    assert!(settings.get_field_value("nonexistent").is_err());
    assert!(settings
        .set_field_value("nonexistent", serde_json::json!(1))
        .is_err());
}

#[test]
fn test_repository_basic() {
    let backend = MemoryBackend::new();
    let repo = SettingsRepository::<TestSettings>::new(Box::new(backend));

    let settings = repo.get();
    assert_eq!(settings, TestSettings::default());

    repo.update(|s| {
        s.enabled = true;
        s.volume = 50.0;
        s.name = "test".into();
    })
    .unwrap();

    let updated = repo.get();
    assert!(updated.enabled);
    assert_eq!(updated.volume, 50.0);
}

#[test]
fn test_repository_set_field() {
    let backend = MemoryBackend::new();
    let repo = SettingsRepository::<TestSettings>::new(Box::new(backend));

    repo.set_field("volume", serde_json::json!(80.0)).unwrap();
    assert_eq!(repo.get().volume, 80.0);
}

#[test]
fn test_repository_validation() {
    let backend = MemoryBackend::new();
    let repo = SettingsRepository::<TestSettings>::new(Box::new(backend));

    assert!(repo.set_field("volume", serde_json::json!(50.0)).is_ok());

    let result = repo.set_field("volume", serde_json::json!(150.0));
    assert!(result.is_err());
    assert_eq!(repo.get().volume, 50.0);
}

#[test]
fn test_repository_reset_field() {
    let backend = MemoryBackend::new();
    let repo = SettingsRepository::<TestSettings>::new(Box::new(backend));

    repo.set_field("volume", serde_json::json!(80.0)).unwrap();
    assert_eq!(repo.get().volume, 80.0);

    repo.reset_field("volume").unwrap();
    assert_eq!(repo.get().volume, 0.0);
}

#[test]
fn test_repository_reset_all() {
    let backend = MemoryBackend::new();
    let repo = SettingsRepository::<TestSettings>::new(Box::new(backend));

    repo.update(|s| {
        s.enabled = true;
        s.volume = 99.0;
        s.name = "Custom".into();
    })
    .unwrap();

    repo.reset_all().unwrap();
    assert_eq!(repo.get(), TestSettings::default());
}

#[test]
fn test_dependency_check() {
    let mut settings = TestSettings::default();
    settings.enabled = false;

    let advanced_field = settings.field_by_name("advanced").unwrap();
    assert!(!settings.is_field_enabled(&advanced_field));

    settings.enabled = true;
    assert!(settings.is_field_enabled(&advanced_field));
}

#[test]
fn test_grouped_by_category() {
    let settings = TestSettings::default();
    let groups = settings.grouped_by_category();

    assert!(!groups.is_empty());

    let general = groups.iter().find(|(cat, _)| cat == "General");
    assert!(general.is_some());
}

#[test]
fn test_resettable_fields() {
    let settings = TestSettings::default();
    let resettable = settings.resettable_fields();

    assert!(resettable.iter().all(|f| f.name != "critical_flag"));
    assert!(resettable.iter().any(|f| f.name == "volume"));
}

#[test]
fn test_backup_roundtrip() {
    let backend = MemoryBackend::new();
    let repo = SettingsRepository::<TestSettings>::new(Box::new(backend));

    repo.update(|s| {
        s.enabled = true;
        s.volume = 42.0;
        s.name = "backup_test".into();
    })
    .unwrap();

    let backup_mgr = SettingsBackupManager::new("test_app", 1);
    let exported = backup_mgr.export(&repo.get()).unwrap();

    let imported: TestSettings = backup_mgr
        .import(
            &exported,
            &multiplatform_settings_core::backup::ImportOptions::default(),
        )
        .unwrap();

    assert_eq!(imported, repo.get());
}

#[test]
fn test_migration() {
    let mut data = serde_json::json!({
        "old_key": "value",
        "remove_me": true,
        "keep": 42
    });

    let mgr = MigrationManager::new(3)
        .add_key_rename(0, 1, "old_key", "new_key")
        .add_key_deletion(1, 2, vec!["remove_me".into()]);

    let result = mgr.migrate(&mut data);
    assert!(matches!(
        result,
        multiplatform_settings_core::migration::MigrationResult::Success { .. }
    ));

    assert_eq!(data["new_key"], "value");
    assert!(data.get("old_key").is_none());
    assert!(data.get("remove_me").is_none());
    assert_eq!(data["keep"], 42);
}

#[test]
fn test_undo_redo() {
    let repo = Arc::new(SettingsRepository::<TestSettings>::new(Box::new(
        MemoryBackend::new(),
    )));
    let undo = UndoManager::new(repo.clone(), 10);

    undo.set_field("volume", serde_json::json!(50.0)).unwrap();

    assert!(undo.can_undo());
    assert!(!undo.can_redo());

    undo.undo().unwrap();
    assert_eq!(repo.get().volume, 0.0);
    assert!(undo.can_redo());

    undo.redo().unwrap();
    assert_eq!(repo.get().volume, 50.0);
}

#[test]
fn test_subscribe() {
    let repo = SettingsRepository::<TestSettings>::new(Box::new(MemoryBackend::new()));
    let mut rx = repo.subscribe();

    assert_eq!(*rx.borrow(), TestSettings::default());

    repo.set_field("volume", serde_json::json!(77.0)).unwrap();

    assert_eq!(rx.borrow_and_update().volume, 77.0);
}

#[test]
fn test_change_listener() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let repo = SettingsRepository::<TestSettings>::new(Box::new(MemoryBackend::new()));
    let change_count = Arc::new(AtomicUsize::new(0));
    let count_clone = change_count.clone();

    repo.add_change_listener(Box::new(move |_field_name, _old, _new| {
        count_clone.fetch_add(1, Ordering::SeqCst);
    }));

    repo.set_field("volume", serde_json::json!(10.0)).unwrap();
    assert!(change_count.load(Ordering::SeqCst) >= 1);

    repo.set_field("enabled", serde_json::json!(true)).unwrap();
    assert!(change_count.load(Ordering::SeqCst) >= 2);
}

#[test]
fn test_json_file_backend() {
    let dir = std::env::temp_dir().join("settings_test");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("test_settings.json");

    let _ = std::fs::remove_file(&path);

    let backend = JsonFileBackend::new(path.clone());
    let repo = SettingsRepository::<TestSettings>::new(Box::new(backend));

    repo.update(|s| {
        s.volume = 88.0;
        s.name = "file_test".into();
    })
    .unwrap();

    let backend2 = JsonFileBackend::new(path.clone());
    let repo2 = SettingsRepository::<TestSettings>::new(Box::new(backend2));
    assert_eq!(repo2.get().volume, 88.0);
    assert_eq!(repo2.get().name, "file_test");

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn test_schema_version_and_app_id() {
    let settings = TestSettings::default();
    assert_eq!(settings.schema_version(), 1);
    assert_eq!(settings.app_id(), "test_app");
}

#[test]
fn test_field_kind_slider() {
    let settings = TestSettings::default();
    let volume_field = settings.field_by_name("volume").unwrap();

    match &volume_field.kind {
        multiplatform_settings_core::field::FieldKind::Slider { min, max, step } => {
            assert_eq!(*min, 0.0);
            assert_eq!(*max, 100.0);
            assert_eq!(*step, 1.0);
        }
        other => panic!("Expected Slider, got {:?}", other),
    }
}

#[test]
fn test_complex_types() {
    let mut settings = AppSettings::default();

    settings.theme = Theme::Dark;
    assert_eq!(settings.theme, Theme::Dark);

    settings.swipe_config = SwipeConfig {
        left: SwipeAction::Archive,
        right: SwipeAction::Delete,
    };
    assert_eq!(settings.swipe_config.left, SwipeAction::Archive);
}
