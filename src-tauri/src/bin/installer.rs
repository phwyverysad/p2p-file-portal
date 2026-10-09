#![windows_subsystem = "windows"]

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const EXE_NAME: &str = "P2PFilePortal.exe";
const DOWNLOAD_URL: &str = "https://github.com/phwyverysad/p2p-file-portal/releases/latest/download/P2P-File-Portal-Portable.exe";
const VERSION_URL: &str = "https://github.com/phwyverysad/p2p-file-portal/releases/latest/download/version.txt";

fn get_install_paths() -> (PathBuf, PathBuf, PathBuf) {
    let local_data = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
        .join("Programs")
        .join("P2PFilePortal");

    let exe_path = local_data.join(EXE_NAME);
    let version_path = local_data.join("version.txt");

    (local_data, exe_path, version_path)
}

fn check_remote_version() -> Option<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .danger_accept_invalid_certs(true)
        .build()
        .ok()?;

    let resp = client.get(VERSION_URL).send().ok()?;
    if resp.status().is_success() {
        let text = resp.text().ok()?;
        return Some(text.trim().to_string());
    }
    None
}

fn download_latest_binary(dest_path: &Path) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())?;

    let mut resp = client
        .get(DOWNLOAD_URL)
        .header("User-Agent", "P2P-File-Portal-Installer")
        .send()
        .map_err(|e| format!("Failed to download: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Server returned HTTP {}", resp.status()));
    }

    let temp_dest = dest_path.with_extension("tmp");
    let mut file = File::create(&temp_dest).map_err(|e| format!("Cannot create file: {}", e))?;
    std::io::copy(&mut resp, &mut file).map_err(|e| format!("Write failed: {}", e))?;
    drop(file);

    // Replace target binary
    if dest_path.exists() {
        let _ = fs::remove_file(dest_path);
    }
    fs::rename(&temp_dest, dest_path).map_err(|e| format!("Rename failed: {}", e))?;

    Ok(())
}

fn kill_running_instances() {
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "P2PFilePortal.exe"])
        .output();
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "p2p-file-portal.exe"])
        .output();
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "local-file-sharer.exe"])
        .output();
    std::thread::sleep(Duration::from_millis(300));
}

fn create_shortcuts(target_exe: &Path) {
    let script = format!(
        "$ws = New-Object -ComObject WScript.Shell; \
         $desk = [System.Environment]::GetFolderPath('Desktop'); \
         $s1 = $ws.CreateShortcut(\"$desk\\P2P File Portal.lnk\"); \
         $s1.TargetPath = \"{}\"; \
         $s1.Save(); \
         $start = [System.Environment]::GetFolderPath('StartMenu'); \
         $s2 = $ws.CreateShortcut(\"$start\\Programs\\P2P File Portal.lnk\"); \
         $s2.TargetPath = \"{}\"; \
         $s2.Save();",
        target_exe.display(),
        target_exe.display()
    );

    let _ = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
        .output();
}

fn launch_app(target_exe: &Path) {
    let _ = Command::new(target_exe).spawn();
}

fn show_native_message(title: &str, message: &str) {
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::iter::once;
        use std::os::windows::ffi::OsStrExt;

        extern "system" {
            fn MessageBoxW(hwnd: isize, text: *const u16, caption: *const u16, utype: u32) -> i32;
        }

        let wide_title: Vec<u16> = OsStr::new(title).encode_wide().chain(once(0)).collect();
        let wide_msg: Vec<u16> = OsStr::new(message).encode_wide().chain(once(0)).collect();

        unsafe {
            MessageBoxW(0, wide_msg.as_ptr(), wide_title.as_ptr(), 0x00000040); // MB_OK | MB_ICONINFORMATION
        }
    }
}

fn main() {
    let (install_dir, exe_path, version_path) = get_install_paths();

    // Check if already installed
    let is_installed = exe_path.exists();
    let local_version = if version_path.exists() {
        fs::read_to_string(&version_path).ok().map(|s| s.trim().to_string())
    } else {
        None
    };

    // If installed, check if update is needed
    if is_installed {
        // Fast-path: check if a newer version is available
        if let Some(remote_ver) = check_remote_version() {
            if let Some(ref local_ver) = local_version {
                if local_ver == &remote_ver {
                    // Up to date! Launch immediately with zero clicks!
                    launch_app(&exe_path);
                    return;
                }
            }
        } else {
            // Cannot reach GitHub or offline: launch existing installed copy immediately
            launch_app(&exe_path);
            return;
        }
    }

    // Need to install or update
    let _ = fs::create_dir_all(&install_dir);
    kill_running_instances();

    match download_latest_binary(&exe_path) {
        Ok(_) => {
            // Save version
            if let Some(remote_ver) = check_remote_version() {
                let _ = fs::write(&version_path, remote_ver);
            } else {
                let _ = fs::write(&version_path, env!("CARGO_PKG_VERSION"));
            }

            create_shortcuts(&exe_path);
            launch_app(&exe_path);
        }
        Err(err) => {
            if is_installed {
                // If download failed but we already have an installed copy, launch it!
                launch_app(&exe_path);
            } else {
                show_native_message(
                    "P2P File Portal Web Installer",
                    &format!("ไม่สามารถดาวน์โหลด P2P File Portal ได้\nโปรดตรวจสอบการเชื่อมต่ออินเทอร์เน็ตแล้วลองใหม่อีกครั้ง\n\nรายละเอียด: {}", err)
                );
            }
        }
    }
}
