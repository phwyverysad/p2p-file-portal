#![windows_subsystem = "windows"]

use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use windows::core::{w, Interface, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreatePen, DeleteObject, EndPaint, GetStockObject, LineTo, MoveToEx,
    SelectObject, SetBkMode, SetTextColor, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS,
    DEFAULT_CHARSET, DEFAULT_PITCH, FW_BOLD, FW_NORMAL, HBRUSH, HDC, OUT_DEFAULT_PRECIS,
    PAINTSTRUCT, PS_SOLID, TRANSPARENT, WHITE_BRUSH,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
use windows::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_PROGRESS_CLASS, ICC_STANDARD_CLASSES, INITCOMMONCONTROLSEX,
    PBM_SETPOS, PBM_SETRANGE32, PBS_SMOOTH, PROGRESS_CLASS,
};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use windows::Win32::UI::WindowsAndMessaging::*;

const EXE_NAME: &str = "P2PFilePortal.exe";
const DOWNLOAD_URL: &str =
    "https://github.com/phwyverysad/p2p-file-portal/releases/latest/download/P2P-File-Portal-Portable.exe";
const VERSION_URL: &str =
    "https://github.com/phwyverysad/p2p-file-portal/releases/latest/download/version.txt";

const WM_APP_UPDATE_PROGRESS: u32 = WM_APP + 1;
const WM_APP_UPDATE_STATUS: u32 = WM_APP + 2;
const WM_APP_DONE: u32 = WM_APP + 3;
const WM_APP_FAILED: u32 = WM_APP + 4;
const WM_APP_ALREADY_INSTALLED: u32 = WM_APP + 5;

const IDC_CHK_DESKTOP: isize = 101;
const IDC_CHK_START: isize = 102;
const IDC_BTN_CANCEL: isize = 103;
const IDC_BTN_BROWSE: isize = 104;
const IDC_BTN_LANG: isize = 105;
const IDC_BTN_REINSTALL: isize = 106;

struct UIHandles {
    _main_hwnd: HWND,
    progress_hwnd: HWND,
    status_hwnd: HWND,
    pct_hwnd: HWND,
    detail_hwnd: HWND,
    edit_path: HWND,
    chk_desktop: HWND,
    chk_start: HWND,
    btn_cancel: HWND,
    btn_browse: HWND,
    btn_lang: HWND,
    btn_reinstall: HWND,
    lbl_sub: HWND,
    lbl_path: HWND,
}

unsafe impl Send for UIHandles {}
unsafe impl Sync for UIHandles {}

#[inline]
unsafe fn send_msg(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> LRESULT {
    SendMessageW(hwnd, msg, Some(WPARAM(wparam)), Some(LPARAM(lparam)))
}

// Global thread-safe state
static UI_HANDLES: OnceLock<UIHandles> = OnceLock::new();
static STATUS_THAI: Mutex<String> = Mutex::new(String::new());
static STATUS_ENG: Mutex<String> = Mutex::new(String::new());
static DETAIL_THAI: Mutex<String> = Mutex::new(String::new());
static DETAIL_ENG: Mutex<String> = Mutex::new(String::new());
static SHOULD_CANCEL: AtomicBool = AtomicBool::new(false);
static FORCE_REINSTALL: AtomicBool = AtomicBool::new(false);
static IS_ENGLISH: AtomicBool = AtomicBool::new(false);
static CANCEL_IS_EXIT: AtomicBool = AtomicBool::new(false);

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn get_default_install_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
        .join("Programs")
        .join("P2PFilePortal")
}

fn get_current_install_dir() -> PathBuf {
    if let Some(handles) = UI_HANDLES.get() {
        let mut buf = [0u16; 512];
        let len = unsafe { GetWindowTextW(handles.edit_path, &mut buf) };
        if len > 0 {
            let path_str = String::from_utf16_lossy(&buf[..len as usize]);
            let trimmed = path_str.trim();
            if !trimmed.is_empty() {
                return PathBuf::from(trimmed);
            }
        }
    }
    get_default_install_dir()
}

fn kill_running_instances() {
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return;
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let name = String::from_utf16_lossy(
                    &entry.szExeFile[..entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len())],
                );
                let name_lower = name.to_lowercase();
                if name_lower == "p2pfileportal.exe"
                    || name_lower == "p2p-file-portal.exe"
                    || name_lower == "local-file-sharer.exe"
                {
                    if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, entry.th32ProcessID) {
                        let _ = TerminateProcess(handle, 0);
                        let _ = CloseHandle(handle);
                    }
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }
    std::thread::sleep(Duration::from_millis(200));
}

fn create_shortcut(target_exe: &Path, lnk_path: &Path) -> Result<(), windows::core::Error> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

        let target_wide: Vec<u16> = target_exe.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        link.SetPath(PCWSTR(target_wide.as_ptr()))?;

        if let Some(parent) = target_exe.parent() {
            let dir_wide: Vec<u16> = parent.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
            link.SetWorkingDirectory(PCWSTR(dir_wide.as_ptr()))?;
        }

        let persist: IPersistFile = link.cast()?;
        let lnk_wide: Vec<u16> = lnk_path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        persist.Save(PCWSTR(lnk_wide.as_ptr()), true)?;
        Ok(())
    }
}

fn get_desktop_shortcut_path() -> Option<PathBuf> {
    if let Some(d) = dirs::desktop_dir() {
        return Some(d.join("P2P File Portal.lnk"));
    }
    if let Ok(user) = std::env::var("USERPROFILE") {
        return Some(PathBuf::from(user).join("Desktop").join("P2P File Portal.lnk"));
    }
    None
}

fn get_start_menu_shortcut_path() -> Option<PathBuf> {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let programs = PathBuf::from(appdata)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs");
        let _ = fs::create_dir_all(&programs);
        return Some(programs.join("P2P File Portal.lnk"));
    }
    None
}

fn launch_app(target_exe: &Path) {
    let _ = std::process::Command::new(target_exe).spawn();
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

fn post_status_update(hwnd: HWND, status_th: &str, status_en: &str, detail_th: &str, detail_en: &str) {
    if let Ok(mut lock) = STATUS_THAI.lock() {
        *lock = status_th.to_string();
    }
    if let Ok(mut lock) = STATUS_ENG.lock() {
        *lock = status_en.to_string();
    }
    if let Ok(mut lock) = DETAIL_THAI.lock() {
        *lock = detail_th.to_string();
    }
    if let Ok(mut lock) = DETAIL_ENG.lock() {
        *lock = detail_en.to_string();
    }
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_APP_UPDATE_STATUS, WPARAM(0), LPARAM(0));
    }
}

fn post_progress_update(hwnd: HWND, pct: usize) {
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_APP_UPDATE_PROGRESS, WPARAM(pct), LPARAM(0));
    }
}

fn apply_language(is_en: bool) {
    if let Some(h) = UI_HANDLES.get() {
        unsafe {
            if is_en {
                let _ = SetWindowTextW(h.lbl_sub, PCWSTR(to_wide("Fast & Private Direct File Sharing (LAN & Internet)").as_ptr()));
                let _ = SetWindowTextW(h.lbl_path, PCWSTR(to_wide("Installation Directory:").as_ptr()));
                let _ = SetWindowTextW(h.chk_desktop, PCWSTR(to_wide("Create Desktop Shortcut").as_ptr()));
                let _ = SetWindowTextW(h.chk_start, PCWSTR(to_wide("Create Start Menu Shortcut").as_ptr()));
                let _ = SetWindowTextW(h.btn_browse, PCWSTR(to_wide("Browse...").as_ptr()));
                let _ = SetWindowTextW(h.btn_lang, PCWSTR(to_wide("TH / ภาษาไทย").as_ptr()));
                let _ = SetWindowTextW(h.btn_reinstall, PCWSTR(to_wide("Reinstall").as_ptr()));
                if CANCEL_IS_EXIT.load(Ordering::SeqCst) {
                    let _ = SetWindowTextW(h.btn_cancel, PCWSTR(to_wide("Launch App").as_ptr()));
                } else {
                    let _ = SetWindowTextW(h.btn_cancel, PCWSTR(to_wide("Cancel").as_ptr()));
                }

                if let Ok(lock) = STATUS_ENG.lock() {
                    if !lock.is_empty() {
                        let _ = SetWindowTextW(h.status_hwnd, PCWSTR(to_wide(&lock).as_ptr()));
                    }
                }
                if let Ok(lock) = DETAIL_ENG.lock() {
                    let _ = SetWindowTextW(h.detail_hwnd, PCWSTR(to_wide(&lock).as_ptr()));
                }
            } else {
                let _ = SetWindowTextW(h.lbl_sub, PCWSTR(to_wide("ส่งไฟล์ความเร็วสูง ปลอดภัยทั้งในวงแลนและอินเทอร์เน็ต").as_ptr()));
                let _ = SetWindowTextW(h.lbl_path, PCWSTR(to_wide("โฟลเดอร์ติดตั้ง:").as_ptr()));
                let _ = SetWindowTextW(h.chk_desktop, PCWSTR(to_wide("สร้างไอคอนบนหน้า Desktop").as_ptr()));
                let _ = SetWindowTextW(h.chk_start, PCWSTR(to_wide("สร้างทางลัดในเมนูเริ่ม (Start Menu)").as_ptr()));
                let _ = SetWindowTextW(h.btn_browse, PCWSTR(to_wide("เลือก...").as_ptr()));
                let _ = SetWindowTextW(h.btn_lang, PCWSTR(to_wide("EN / English").as_ptr()));
                let _ = SetWindowTextW(h.btn_reinstall, PCWSTR(to_wide("ติดตั้งใหม่").as_ptr()));
                if CANCEL_IS_EXIT.load(Ordering::SeqCst) {
                    let _ = SetWindowTextW(h.btn_cancel, PCWSTR(to_wide("เปิดโปรแกรม").as_ptr()));
                } else {
                    let _ = SetWindowTextW(h.btn_cancel, PCWSTR(to_wide("ยกเลิก").as_ptr()));
                }

                if let Ok(lock) = STATUS_THAI.lock() {
                    if !lock.is_empty() {
                        let _ = SetWindowTextW(h.status_hwnd, PCWSTR(to_wide(&lock).as_ptr()));
                    }
                }
                if let Ok(lock) = DETAIL_THAI.lock() {
                    let _ = SetWindowTextW(h.detail_hwnd, PCWSTR(to_wide(&lock).as_ptr()));
                }
            }
        }
    }
}

// Background Installation Worker
fn run_installation_worker(hwnd: HWND) {
    let install_dir = get_current_install_dir();
    let exe_path = install_dir.join(EXE_NAME);
    let version_path = install_dir.join("version.txt");

    let is_installed = exe_path.exists();
    let local_ver = if version_path.exists() {
        fs::read_to_string(&version_path).ok().map(|s| s.trim().to_string())
    } else {
        None
    };

    post_status_update(
        hwnd,
        "กำลังตรวจสอบเวอร์ชันและไฟล์ติดตั้ง...",
        "Checking latest version and setup files...",
        "กำลังเชื่อมต่อ...",
        "Connecting...",
    );
    post_progress_update(hwnd, 5);

    let remote_ver = check_remote_version();

    // Check if current installed version matches latest remote version
    if is_installed && !FORCE_REINSTALL.load(Ordering::SeqCst) {
        if let (Some(ref r_ver), Some(ref l_ver)) = (&remote_ver, &local_ver) {
            if r_ver == l_ver {
                post_progress_update(hwnd, 100);
                post_status_update(
                    hwnd,
                    &format!("ตรวจพบเวอร์ชันล่าสุดแล้ว ({}) พร้อมเปิดใช้งาน", r_ver),
                    &format!("P2P File Portal is already up to date ({})", r_ver),
                    "คลิก 'เปิดโปรแกรม' หรือระบบจะเปิดอัตโนมัติใน 3 วินาที...",
                    "Click 'Launch App' or launching automatically in 3s...",
                );
                unsafe {
                    let _ = PostMessageW(Some(hwnd), WM_APP_ALREADY_INSTALLED, WPARAM(0), LPARAM(0));
                }

                // Countdown 3 seconds allowing user to click Reinstall if desired
                for _ in 0..30 {
                    if SHOULD_CANCEL.load(Ordering::SeqCst) || FORCE_REINSTALL.load(Ordering::SeqCst) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }

                if !FORCE_REINSTALL.load(Ordering::SeqCst) {
                    launch_app(&exe_path);
                    unsafe {
                        let _ = PostMessageW(Some(hwnd), WM_APP_DONE, WPARAM(0), LPARAM(0));
                    }
                    return;
                }
            }
        }
    }

    // Need installation, update, or forced reinstall
    post_progress_update(hwnd, 10);
    post_status_update(
        hwnd,
        "กำลังเตรียมพื้นที่ติดตั้ง...",
        "Preparing installation directory...",
        &install_dir.display().to_string(),
        &install_dir.display().to_string(),
    );

    let _ = fs::create_dir_all(&install_dir);
    kill_running_instances();

    post_progress_update(hwnd, 15);
    post_status_update(
        hwnd,
        "กำลังดาวน์โหลด P2P File Portal...",
        "Downloading P2P File Portal...",
        "กำลังเชื่อมต่อเซิร์ฟเวอร์ดาวน์โหลด...",
        "Connecting to download server...",
    );

    // Download stream
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(180))
        .danger_accept_invalid_certs(true)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            if is_installed {
                post_status_update(
                    hwnd,
                    "ไม่สามารถเชื่อมต่อได้ กำลังเปิดแอปเดิม...",
                    "Offline. Launching existing installation...",
                    "",
                    "",
                );
                std::thread::sleep(Duration::from_millis(800));
                launch_app(&exe_path);
                unsafe {
                    let _ = PostMessageW(Some(hwnd), WM_APP_DONE, WPARAM(0), LPARAM(0));
                }
                return;
            }
            post_status_update(hwnd, "เกิดข้อผิดพลาดในการเชื่อมต่อ", "Connection Error", &e.to_string(), &e.to_string());
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_APP_FAILED, WPARAM(0), LPARAM(0));
            }
            return;
        }
    };

    let mut resp = match client
        .get(DOWNLOAD_URL)
        .header("User-Agent", "P2P-File-Portal-Installer")
        .send()
    {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            if is_installed {
                launch_app(&exe_path);
                unsafe {
                    let _ = PostMessageW(Some(hwnd), WM_APP_DONE, WPARAM(0), LPARAM(0));
                }
                return;
            }
            let err = format!("Server returned HTTP {}", r.status());
            post_status_update(hwnd, "ดาวน์โหลดไม่สำเร็จ", "Download Failed", &err, &err);
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_APP_FAILED, WPARAM(0), LPARAM(0));
            }
            return;
        }
        Err(e) => {
            if is_installed {
                launch_app(&exe_path);
                unsafe {
                    let _ = PostMessageW(Some(hwnd), WM_APP_DONE, WPARAM(0), LPARAM(0));
                }
                return;
            }
            let err = e.to_string();
            post_status_update(hwnd, "การเชื่อมต่อขัดข้อง", "Network Connection Failed", &err, &err);
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_APP_FAILED, WPARAM(0), LPARAM(0));
            }
            return;
        }
    };

    let total_len = resp.content_length().unwrap_or(0);
    let temp_dest = exe_path.with_extension("tmp");
    let mut file = match File::create(&temp_dest) {
        Ok(f) => f,
        Err(e) => {
            let err = e.to_string();
            post_status_update(hwnd, "ไม่สามารถสร้างไฟล์ติดตั้ง", "Cannot create setup file", &err, &err);
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_APP_FAILED, WPARAM(0), LPARAM(0));
            }
            return;
        }
    };

    let mut downloaded: u64 = 0;
    let mut buffer = [0u8; 64 * 1024];

    loop {
        if SHOULD_CANCEL.load(Ordering::SeqCst) {
            drop(file);
            let _ = fs::remove_file(&temp_dest);
            return;
        }

        match resp.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                if file.write_all(&buffer[..n]).is_err() {
                    break;
                }
                downloaded += n as u64;

                let pct = if total_len > 0 {
                    15 + ((downloaded as f64 / total_len as f64) * 75.0) as usize
                } else {
                    50
                };
                post_progress_update(hwnd, pct.min(90));

                let dl_mb = downloaded as f64 / (1024.0 * 1024.0);
                let total_mb = total_len as f64 / (1024.0 * 1024.0);
                let detail = if total_len > 0 {
                    format!("{:.1} MB / {:.1} MB", dl_mb, total_mb)
                } else {
                    format!("{:.1} MB", dl_mb)
                };
                post_status_update(
                    hwnd,
                    "กำลังดาวน์โหลดไฟล์โปรแกรม...",
                    "Downloading application...",
                    &detail,
                    &detail,
                );
            }
            Err(_) => break,
        }
    }

    drop(file);

    if SHOULD_CANCEL.load(Ordering::SeqCst) {
        let _ = fs::remove_file(&temp_dest);
        return;
    }

    post_progress_update(hwnd, 92);
    post_status_update(
        hwnd,
        "กำลังติดตั้งและตรวจสอบความสมบูรณ์ของไฟล์...",
        "Verifying and installing executable...",
        "รอสักครู่...",
        "Please wait...",
    );

    if exe_path.exists() {
        let _ = fs::remove_file(&exe_path);
    }
    if let Err(e) = fs::rename(&temp_dest, &exe_path) {
        let err = e.to_string();
        post_status_update(hwnd, "เกิดข้อผิดพลาดในการบันทึกไฟล์", "Failed to save file", &err, &err);
        unsafe {
            let _ = PostMessageW(Some(hwnd), WM_APP_FAILED, WPARAM(0), LPARAM(0));
        }
        return;
    }

    // Save version
    if let Some(r_ver) = remote_ver {
        let _ = fs::write(&version_path, r_ver);
    } else {
        let _ = fs::write(&version_path, env!("CARGO_PKG_VERSION"));
    }

    post_progress_update(hwnd, 96);
    post_status_update(
        hwnd,
        "กำลังสร้างทางลัดโปรแกรม...",
        "Creating shortcuts...",
        "",
        "",
    );

    // Read checkboxes
    if let Some(handles) = UI_HANDLES.get() {
        unsafe {
            let create_desktop = send_msg(
                handles.chk_desktop,
                BM_GETCHECK,
                0,
                0,
            )
            .0 == 1; // BST_CHECKED = 1

            if create_desktop {
                if let Some(desktop_lnk) = get_desktop_shortcut_path() {
                    let _ = create_shortcut(&exe_path, &desktop_lnk);
                }
            }

            let create_start = send_msg(
                handles.chk_start,
                BM_GETCHECK,
                0,
                0,
            )
            .0 == 1;

            if create_start {
                if let Some(start_lnk) = get_start_menu_shortcut_path() {
                    let _ = create_shortcut(&exe_path, &start_lnk);
                }
            }
        }
    }

    post_progress_update(hwnd, 100);
    post_status_update(
        hwnd,
        "ติดตั้งเสร็จสมบูรณ์! กำลังเปิดโปรแกรม...",
        "Installation complete! Launching P2P File Portal...",
        "100%",
        "100%",
    );

    std::thread::sleep(Duration::from_millis(800));
    launch_app(&exe_path);

    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_APP_DONE, WPARAM(0), LPARAM(0));
    }
}

// Window Procedure
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CTLCOLORSTATIC => {
            let hdc = HDC(wparam.0 as _);
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, COLORREF(0x00334155)); // Slate-700
            LRESULT(GetStockObject(WHITE_BRUSH).0 as isize)
        }
        WM_CTLCOLORDLG | WM_CTLCOLORBTN => {
            let hdc = HDC(wparam.0 as _);
            SetBkMode(hdc, TRANSPARENT);
            LRESULT(GetStockObject(WHITE_BRUSH).0 as isize)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            let pen = CreatePen(PS_SOLID, 1, COLORREF(0x00E2E8F0));
            let old_pen = SelectObject(hdc, pen.into());
            let _ = MoveToEx(hdc, 24, 74, None);
            let _ = LineTo(hdc, 496, 74);
            let _ = MoveToEx(hdc, 24, 298, None);
            let _ = LineTo(hdc, 496, 298);
            let _ = SelectObject(hdc, old_pen);
            let _ = DeleteObject(pen.into());
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_APP_UPDATE_PROGRESS => {
            if let Some(handles) = UI_HANDLES.get() {
                let pct = wparam.0 as usize;
                let _ = send_msg(handles.progress_hwnd, PBM_SETPOS, pct, 0);
                let pct_text = to_wide(&format!("{}%", pct));
                let _ = SetWindowTextW(handles.pct_hwnd, PCWSTR(pct_text.as_ptr()));
            }
            LRESULT(0)
        }
        WM_APP_UPDATE_STATUS => {
            let is_en = IS_ENGLISH.load(Ordering::SeqCst);
            apply_language(is_en);
            LRESULT(0)
        }
        WM_APP_ALREADY_INSTALLED => {
            CANCEL_IS_EXIT.store(true, Ordering::SeqCst);
            if let Some(handles) = UI_HANDLES.get() {
                let _ = ShowWindow(handles.btn_reinstall, SW_SHOW);
                let is_en = IS_ENGLISH.load(Ordering::SeqCst);
                apply_language(is_en);
            }
            LRESULT(0)
        }
        WM_APP_DONE => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_APP_FAILED => {
            CANCEL_IS_EXIT.store(true, Ordering::SeqCst);
            if let Some(handles) = UI_HANDLES.get() {
                let is_en = IS_ENGLISH.load(Ordering::SeqCst);
                let btn_text = to_wide(if is_en { "Close" } else { "ปิด" });
                let _ = SetWindowTextW(handles.btn_cancel, PCWSTR(btn_text.as_ptr()));
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = (wparam.0 & 0xffff) as isize;
            match id {
                IDC_BTN_CANCEL => {
                    if CANCEL_IS_EXIT.load(Ordering::SeqCst) {
                        let install_dir = get_current_install_dir();
                        let exe_path = install_dir.join(EXE_NAME);
                        if exe_path.exists() {
                            launch_app(&exe_path);
                        }
                    } else {
                        SHOULD_CANCEL.store(true, Ordering::SeqCst);
                    }
                    PostQuitMessage(0);
                }
                IDC_BTN_BROWSE => {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        if let Some(handles) = UI_HANDLES.get() {
                            let wide = to_wide(&folder.to_string_lossy());
                            let _ = SetWindowTextW(handles.edit_path, PCWSTR(wide.as_ptr()));
                        }
                    }
                }
                IDC_BTN_LANG => {
                    let current = IS_ENGLISH.load(Ordering::SeqCst);
                    let new_val = !current;
                    IS_ENGLISH.store(new_val, Ordering::SeqCst);
                    apply_language(new_val);
                }
                IDC_BTN_REINSTALL => {
                    FORCE_REINSTALL.store(true, Ordering::SeqCst);
                    CANCEL_IS_EXIT.store(false, Ordering::SeqCst);
                    if let Some(handles) = UI_HANDLES.get() {
                        let _ = ShowWindow(handles.btn_reinstall, SW_HIDE);
                        let is_en = IS_ENGLISH.load(Ordering::SeqCst);
                        let cancel_text = to_wide(if is_en { "Cancel" } else { "ยกเลิก" });
                        let _ = SetWindowTextW(handles.btn_cancel, PCWSTR(cancel_text.as_ptr()));
                        let hwnd_raw = handles._main_hwnd.0 as usize;
                        std::thread::spawn(move || {
                            let thread_hwnd = HWND(hwnd_raw as *mut core::ffi::c_void);
                            run_installation_worker(thread_hwnd);
                        });
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            SHOULD_CANCEL.store(true, Ordering::SeqCst);
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn create_gui_and_run() {
    unsafe {
        let icc = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_STANDARD_CLASSES | ICC_PROGRESS_CLASS,
        };
        let _ = InitCommonControlsEx(&icc);

        let h_inst: HINSTANCE = GetModuleHandleW(None).unwrap().into();
        let class_name = w!("P2PFilePortalInstallerClass");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_inst,
            hIcon: LoadIconW(None, IDI_APPLICATION).unwrap_or_default(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: HICON::default(),
        };
        RegisterClassExW(&wc);

        let win_w = 540;
        let win_h = 390;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let pos_x = (screen_w - win_w) / 2;
        let pos_y = (screen_h - win_h) / 2;

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("P2P File Portal Setup | ติดตั้งโปรแกรม"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            pos_x,
            pos_y,
            win_w,
            win_h,
            None,
            None,
            Some(h_inst),
            None,
        )
        .expect("Failed to create main window");

        // Typography fonts
        let font_title = CreateFontW(
            22, 0, 0, 0,
            FW_BOLD.0 as i32,
            0, 0, 0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32,
            w!("Segoe UI"),
        );

        let font_bold = CreateFontW(
            15, 0, 0, 0,
            FW_BOLD.0 as i32,
            0, 0, 0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32,
            w!("Segoe UI"),
        );

        let font_regular = CreateFontW(
            15, 0, 0, 0,
            FW_NORMAL.0 as i32,
            0, 0, 0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32,
            w!("Segoe UI"),
        );

        let font_small = CreateFontW(
            13, 0, 0, 0,
            FW_NORMAL.0 as i32,
            0, 0, 0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32,
            w!("Segoe UI"),
        );

        // Header Title
        let h_title = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("P2P File Portal Setup"),
            WS_CHILD | WS_VISIBLE,
            24, 16, 320, 28,
            Some(hwnd),
            None,
            Some(h_inst),
            None,
        ).unwrap();
        let _ = send_msg(h_title, WM_SETFONT, font_title.0 as usize, 1);

        // Language Switch Button (Top Right)
        let btn_lang = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("EN / English"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            396, 18, 100, 26,
            Some(hwnd),
            Some(HMENU(IDC_BTN_LANG as _)),
            Some(h_inst),
            None,
        ).unwrap();
        let _ = send_msg(btn_lang, WM_SETFONT, font_small.0 as usize, 1);

        // Subtitle
        let lbl_sub = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("ส่งไฟล์ความเร็วสูง ปลอดภัยทั้งในวงแลนและอินเทอร์เน็ต"),
            WS_CHILD | WS_VISIBLE,
            24, 46, 470, 20,
            Some(hwnd),
            None,
            Some(h_inst),
            None,
        ).unwrap();
        let _ = send_msg(lbl_sub, WM_SETFONT, font_small.0 as usize, 1);

        // Path Label
        let lbl_path = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("โฟลเดอร์ติดตั้ง:"),
            WS_CHILD | WS_VISIBLE,
            24, 84, 470, 18,
            Some(hwnd),
            None,
            Some(h_inst),
            None,
        ).unwrap();
        let _ = send_msg(lbl_path, WM_SETFONT, font_bold.0 as usize, 1);

        // Path Edit Box (User can inspect or type custom path)
        let default_install_dir = get_default_install_dir();
        let path_text = to_wide(&default_install_dir.to_string_lossy());
        let edit_path = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("EDIT"),
            PCWSTR(path_text.as_ptr()),
            WS_CHILD | WS_VISIBLE | WS_BORDER | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
            24, 104, 386, 24,
            Some(hwnd),
            None,
            Some(h_inst),
            None,
        ).unwrap();
        let _ = send_msg(edit_path, WM_SETFONT, font_small.0 as usize, 1);

        // Browse Button
        let btn_browse = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("เลือก..."),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            418, 104, 78, 24,
            Some(hwnd),
            Some(HMENU(IDC_BTN_BROWSE as _)),
            Some(h_inst),
            None,
        ).unwrap();
        let _ = send_msg(btn_browse, WM_SETFONT, font_small.0 as usize, 1);

        // Checkbox: Desktop Shortcut
        let chk_desktop = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("สร้างไอคอนบนหน้า Desktop"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            24, 138, 470, 22,
            Some(hwnd),
            Some(HMENU(IDC_CHK_DESKTOP as _)),
            Some(h_inst),
            None,
        ).unwrap();
        let _ = send_msg(chk_desktop, WM_SETFONT, font_regular.0 as usize, 1);
        let _ = send_msg(chk_desktop, BM_SETCHECK, 1, 0); // Checked

        // Checkbox: Start Menu Shortcut
        let chk_start = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("สร้างทางลัดในเมนูเริ่ม (Start Menu)"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            24, 162, 470, 22,
            Some(hwnd),
            Some(HMENU(IDC_CHK_START as _)),
            Some(h_inst),
            None,
        ).unwrap();
        let _ = send_msg(chk_start, WM_SETFONT, font_regular.0 as usize, 1);
        let _ = send_msg(chk_start, BM_SETCHECK, 1, 0); // Checked

        // Status Label
        let status_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("กำลังเริ่มต้นการติดตั้ง..."),
            WS_CHILD | WS_VISIBLE,
            24, 196, 400, 20,
            Some(hwnd),
            None,
            Some(h_inst),
            None,
        ).unwrap();
        let _ = send_msg(status_hwnd, WM_SETFONT, font_bold.0 as usize, 1);

        // Percentage Label
        let pct_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("0%"),
            WS_CHILD | WS_VISIBLE,
            434, 196, 60, 20,
            Some(hwnd),
            None,
            Some(h_inst),
            None,
        ).unwrap();
        let _ = send_msg(pct_hwnd, WM_SETFONT, font_bold.0 as usize, 1);

        // Progress Bar
        let progress_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PROGRESS_CLASS,
            PCWSTR::null(),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(PBS_SMOOTH as u32),
            24, 218, 472, 22,
            Some(hwnd),
            None,
            Some(h_inst),
            None,
        ).unwrap();
        let _ = send_msg(progress_hwnd, PBM_SETRANGE32, 0, 100);
        let _ = send_msg(progress_hwnd, PBM_SETPOS, 0, 0);

        // Detail Label (Download MB info)
        let detail_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("กำลังเตรียมการดาวน์โหลด..."),
            WS_CHILD | WS_VISIBLE,
            24, 246, 472, 18,
            Some(hwnd),
            None,
            Some(h_inst),
            None,
        ).unwrap();
        let _ = send_msg(detail_hwnd, WM_SETFONT, font_small.0 as usize, 1);

        // Reinstall Button (Hidden initially, shown if already installed)
        let btn_reinstall = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("ติดตั้งใหม่"),
            WS_CHILD | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            24, 312, 120, 28,
            Some(hwnd),
            Some(HMENU(IDC_BTN_REINSTALL as _)),
            Some(h_inst),
            None,
        ).unwrap();
        let _ = send_msg(btn_reinstall, WM_SETFONT, font_regular.0 as usize, 1);

        // Cancel / Launch Button
        let btn_cancel = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("ยกเลิก"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            376, 312, 120, 28,
            Some(hwnd),
            Some(HMENU(IDC_BTN_CANCEL as _)),
            Some(h_inst),
            None,
        ).unwrap();
        let _ = send_msg(btn_cancel, WM_SETFONT, font_regular.0 as usize, 1);

        let _ = UI_HANDLES.set(UIHandles {
            _main_hwnd: hwnd,
            progress_hwnd,
            status_hwnd,
            pct_hwnd,
            detail_hwnd,
            edit_path,
            chk_desktop,
            chk_start,
            btn_cancel,
            btn_browse,
            btn_lang,
            btn_reinstall,
            lbl_sub,
            lbl_path,
        });

        let _ = ShowWindow(hwnd, SW_SHOW);

        // Spawn background worker thread
        let hwnd_raw = hwnd.0 as usize;
        std::thread::spawn(move || {
            let thread_hwnd = HWND(hwnd_raw as *mut core::ffi::c_void);
            run_installation_worker(thread_hwnd);
        });

        // Windows Message Loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn main() {
    create_gui_and_run();
}
