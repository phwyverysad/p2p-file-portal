#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

pub fn register_context_menu(exe_path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let clean_path = exe_path.strip_prefix(r"\\?\").unwrap_or(exe_path);
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let shell_path = r"Software\Classes\*\shell";
        let (shell_key, _) = hkcu
            .create_subkey(shell_path)
            .map_err(|e| format!("Failed to open shell key: {}", e))?;

        let (menu_key, _) = shell_key
            .create_subkey("FileSharer")
            .map_err(|e| format!("Failed to create FileSharer key: {}", e))?;

        menu_key
            .set_value("", &"แชร์ลิ้ง")
            .map_err(|e| format!("Failed to set menu label: {}", e))?;

        menu_key
            .set_value("Icon", &clean_path)
            .map_err(|e| format!("Failed to set menu icon: {}", e))?;

        let _ = menu_key.set_value("Position", &"Top");

        let (cmd_key, _) = menu_key
            .create_subkey("command")
            .map_err(|e| format!("Failed to create command key: {}", e))?;

        let cmd_string = format!("\"{}\" \"%1\"", clean_path);
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

pub fn set_clipboard_text(text: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        extern "system" {
            fn OpenClipboard(hWndNewOwner: *mut std::ffi::c_void) -> i32;
            fn CloseClipboard() -> i32;
            fn EmptyClipboard() -> i32;
            fn SetClipboardData(uFormat: u32, hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
            fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> *mut std::ffi::c_void;
            fn GlobalLock(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
            fn GlobalUnlock(hMem: *mut std::ffi::c_void) -> i32;
        }

        const CF_UNICODETEXT: u32 = 13;
        const GMEM_MOVEABLE: u32 = 0x0002;

        let wide: Vec<u16> = OsStr::new(text).encode_wide().chain(std::iter::once(0)).collect();
        let bytes = wide.len() * 2;

        unsafe {
            if OpenClipboard(std::ptr::null_mut()) == 0 {
                return Err("Failed to open Windows clipboard".to_string());
            }
            EmptyClipboard();
            let h_mem = GlobalAlloc(GMEM_MOVEABLE, bytes);
            if h_mem.is_null() {
                CloseClipboard();
                return Err("Failed to allocate global memory".to_string());
            }
            let ptr = GlobalLock(h_mem) as *mut u16;
            if ptr.is_null() {
                CloseClipboard();
                return Err("Failed to lock global memory".to_string());
            }
            std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
            GlobalUnlock(h_mem);
            SetClipboardData(CF_UNICODETEXT, h_mem);
            CloseClipboard();
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = text;
        Ok(())
    }
}
