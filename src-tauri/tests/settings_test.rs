use local_file_sharer_lib::settings::AppSettings;
use local_file_sharer_lib::windows::{is_context_menu_registered, register_context_menu, unregister_context_menu};
use tempfile::tempdir;

#[test]
fn test_settings_load_save() {
    let dir = tempdir().unwrap();
    let settings_file = dir.path().join("settings.json");

    let default_settings = AppSettings::load(&settings_file);
    assert_eq!(default_settings.language, "th");
    assert!(default_settings.minimize_to_tray);

    let mut custom = default_settings;
    custom.language = "en".to_string();
    custom.auto_start = true;
    custom.save(&settings_file).expect("should save settings");

    let reloaded = AppSettings::load(&settings_file);
    assert_eq!(reloaded.language, "en");
    assert!(reloaded.auto_start);
}

#[test]
fn test_windows_context_menu_registration() {
    let dummy_exe = r"C:\Test\FileSharer.exe";
    let reg_result = register_context_menu(dummy_exe);
    assert!(reg_result.is_ok());

    #[cfg(target_os = "windows")]
    assert!(is_context_menu_registered());

    let unreg_result = unregister_context_menu();
    assert!(unreg_result.is_ok());

    #[cfg(target_os = "windows")]
    assert!(!is_context_menu_registered());
}
