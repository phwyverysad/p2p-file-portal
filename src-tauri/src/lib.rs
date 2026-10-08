pub mod commands;
pub mod registry;
pub mod server;
pub mod settings;
pub mod shortener;
pub mod stream;
pub mod tunnel;
pub mod windows;

use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tauri_plugin_single_instance::init as init_single_instance;

use registry::FileRegistry;
use server::{create_app, AppState};
use settings::AppSettings;
use tunnel::TunnelManager;

pub struct TauriAppState {
    pub server_state: Arc<AppState>,
    pub tunnel_manager: Arc<TunnelManager>,
    pub settings_path: PathBuf,
    pub app_settings: Arc<RwLock<AppSettings>>,
    pub pending_shares: Arc<std::sync::Mutex<Vec<String>>>,
}

fn try_forward_to_running_instance() -> bool {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let addr = "127.0.0.1:3030";
    let Ok(mut stream) = TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_millis(250)) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));

    // Collect file arguments
    let mut files = Vec::new();
    for arg in std::env::args().skip(1) {
        let p = std::path::Path::new(&arg);
        if p.exists() && p.is_file() {
            files.push(arg);
        }
    }

    if files.is_empty() {
        let req = "POST /api/app/focus HTTP/1.1\r\nHost: 127.0.0.1:3030\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let _ = stream.write_all(req.as_bytes());
        let _ = stream.flush();
        let mut buf = [0u8; 128];
        let _ = stream.read(&mut buf);
        return true;
    }

    for file in files {
        let Ok(mut s) = TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_millis(250)) else {
            return false;
        };
        let _ = s.set_read_timeout(Some(Duration::from_millis(1000)));
        let _ = s.set_write_timeout(Some(Duration::from_millis(1000)));
        let body = serde_json::json!({ "filePath": file }).to_string();
        let req = format!(
            "POST /api/shares/local HTTP/1.1\r\nHost: 127.0.0.1:3030\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = s.write_all(req.as_bytes());
        let _ = s.flush();
        let mut buf = [0u8; 256];
        let _ = s.read(&mut buf);
    }

    true
}

pub fn run() {
    // Fast-path: if an instance is already running, forward arguments and exit instantly without WebView2 collision
    if try_forward_to_running_instance() {
        std::process::exit(0);
    }

    let base_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("FileSharer");
    let _ = std::fs::create_dir_all(&base_dir);

    let storage_path = base_dir.join("shares.json");
    let settings_path = base_dir.join("settings.json");

    let registry = FileRegistry::new(storage_path);
    let app_settings = AppSettings::load(&settings_path);

    // Sync context menu on launch if enabled
    if app_settings.context_menu {
        if let Ok(exe) = std::env::current_exe() {
            let _ = windows::register_context_menu(&exe.to_string_lossy());
        }
    }

    // Capture any initial file arguments passed when launched (e.g. from right-click context menu)
    let mut initial_pending = Vec::new();
    for arg in std::env::args().skip(1) {
        let path = std::path::Path::new(&arg);
        if path.exists() && path.is_file() {
            initial_pending.push(arg);
        }
    }
    let pending_shares = Arc::new(std::sync::Mutex::new(initial_pending));

    let port = 3030;
    let tunnel_manager = Arc::new(TunnelManager::new());
    let app_handle_slot: Arc<RwLock<Option<tauri::AppHandle>>> = Arc::new(RwLock::new(None));
    let app_settings_arc = Arc::new(RwLock::new(app_settings));

    let app_handle_for_share = app_handle_slot.clone();
    let on_share_added = Some(Arc::new(move |dto: &server::ShareItemDto| {
        if let Ok(guard) = app_handle_for_share.read() {
            if let Some(app) = guard.as_ref() {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
                let _ = app.emit("share-file-added", dto);
            }
        }
    }) as server::ShareNotifier);

    let app_handle_for_focus = app_handle_slot.clone();
    let on_focus_requested = Some(Arc::new(move || {
        if let Ok(guard) = app_handle_for_focus.read() {
            if let Some(app) = guard.as_ref() {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
        }
    }) as server::FocusNotifier);

    let server_state = Arc::new(AppState {
        registry,
        port,
        tunnel_url: tunnel_manager.public_url.clone(),
        tunnel_status: tunnel_manager.status.clone(),
        tunnel_error: tunnel_manager.error_message.clone(),
        settings: Some(app_settings_arc.clone()),
        on_share_added,
        on_focus_requested,
    });

    let tauri_state = TauriAppState {
        server_state: server_state.clone(),
        tunnel_manager: tunnel_manager.clone(),
        settings_path,
        app_settings: app_settings_arc.clone(),
        pending_shares: pending_shares.clone(),
    };

    let server_state_bg = server_state.clone();
    let tunnel_mgr_bg = tunnel_manager.clone();
    let pending_shares_si = pending_shares.clone();
    let app_handle_slot_setup = app_handle_slot.clone();

    tauri::Builder::default()
        .plugin(init_single_instance(move |app, args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }

            // If launched with file argument (e.g. from context menu while already running)
            for arg in args.iter().skip(1) {
                let p = std::path::Path::new(arg);
                if p.exists() && p.is_file() {
                    if let Ok(mut pending) = pending_shares_si.lock() {
                        pending.push(arg.clone());
                    }
                    let _ = app.emit("share-file-requested", arg.clone());
                }
            }
        }))
        .manage(tauri_state)
        .setup(move |app| {
            *app_handle_slot_setup.write().unwrap() = Some(app.handle().clone());
            // Spawn HTTP Web Server in background runtime
            let s_state = server_state_bg;
            tauri::async_runtime::spawn(async move {
                let app_router = create_app(s_state);
                let addr = format!("0.0.0.0:{}", port);
                if let Ok(listener) = tokio::net::TcpListener::bind(&addr).await {
                    let _ = axum::serve(listener, app_router).await;
                }
            });

            // Start Cloudflare Tunnel in background
            let t_mgr = tunnel_mgr_bg;
            tauri::async_runtime::spawn(async move {
                t_mgr.start(port).await;
            });

            // Setup System Tray
            let quit_i = MenuItem::with_id(app, "quit", "ปิดโปรแกรม (Quit)", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "เปิดหน้าต่างหลัก (Open)", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("File Sharer - Running in background")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _tray = tray_builder.build(app)?;

            // Handle close to tray
            if let Some(window) = app.get_webview_window("main") {
                let win_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_shares,
            commands::add_local_share,
            commands::update_share,
            commands::revoke_share,
            commands::shorten_share_url,
            commands::get_status,
            commands::get_app_settings,
            commands::save_app_settings,
            commands::pick_file_dialog,
            commands::show_in_folder,
            commands::retry_tunnel,
            commands::get_pending_shares,
            commands::copy_to_clipboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
