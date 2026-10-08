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
}

pub fn run() {
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

    let port = 3030;
    let tunnel_manager = Arc::new(TunnelManager::new());

    let server_state = Arc::new(AppState {
        registry,
        port,
        tunnel_url: tunnel_manager.public_url.clone(),
        tunnel_status: tunnel_manager.status.clone(),
        tunnel_error: tunnel_manager.error_message.clone(),
    });

    let tauri_state = TauriAppState {
        server_state: server_state.clone(),
        tunnel_manager: tunnel_manager.clone(),
        settings_path,
        app_settings: Arc::new(RwLock::new(app_settings)),
    };

    let server_state_bg = server_state.clone();
    let tunnel_mgr_bg = tunnel_manager.clone();

    tauri::Builder::default()
        .plugin(init_single_instance(|app, args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();

                // If launched with file argument (e.g. from context menu)
                if args.len() > 1 {
                    let file_arg = args[1].clone();
                    if std::path::Path::new(&file_arg).exists() {
                        let _ = app.emit("share-file-requested", file_arg);
                    }
                }
            }
        }))
        .manage(tauri_state)
        .setup(move |app| {
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

            let _tray = TrayIconBuilder::new()
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
                })
                .build(app)?;

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
