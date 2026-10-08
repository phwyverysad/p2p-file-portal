#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

pub fn register_context_menu(exe_path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let shell_path = r"Software\Classes\*\shell";
        let (shell_key, _) = hkcu
            .create_subkey(shell_path)
            .map_err(|e| format!("Failed to open shell key: {}", e))?;

        let (menu_key, _) = shell_key
            .create_subkey("FileSharer")
            .map_err(|e| format!("Failed to create FileSharer key: {}", e))?;

        menu_key
            .set_value("", &"แชร์ลิ้ง (Share Link)")
            .map_err(|e| format!("Failed to set menu label: {}", e))?;

        menu_key
            .set_value("Icon", &exe_path)
            .map_err(|e| format!("Failed to set menu icon: {}", e))?;

        let (cmd_key, _) = menu_key
            .create_subkey("command")
            .map_err(|e| format!("Failed to create command key: {}", e))?;

        let cmd_string = format!("\"{}\" \"%1\"", exe_path);
        cmd_key
            .set_value("", &cmd_string)
            .map_err(|e| format!("Failed to set command value: {}", e))?;

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = exe_path;
        Ok(())
    }
}

pub fn unregister_context_menu() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let shell_path = r"Software\Classes\*\shell";
        if let Ok(shell_key) = hkcu.open_subkey_with_flags(shell_path, KEY_WRITE) {
            if let Ok(menu_key) = shell_key.open_subkey_with_flags("FileSharer", KEY_WRITE) {
                let _ = menu_key.delete_subkey("command");
            }
            let _ = shell_key.delete_subkey("FileSharer");
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

pub fn is_context_menu_registered() -> bool {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let cmd_path = r"Software\Classes\*\shell\FileSharer\command";
        hkcu.open_subkey(cmd_path).is_ok()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}
