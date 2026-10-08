use tauri::State;

use crate::server::{format_share_dto, ServerStatusResponse, ShareItemDto};
use crate::settings::AppSettings;
use crate::shortener;
use crate::windows;
use crate::TauriAppState;

// 1. List active shares
#[tauri::command]
pub fn list_shares(state: State<'_, TauriAppState>) -> Result<Vec<ShareItemDto>, String> {
    let items = state.server_state.registry.list();
    let dtos = items
        .into_iter()
        .map(|item| format_share_dto(item, &state.server_state))
        .collect();
    Ok(dtos)
}

// 2. Add local file to shares
#[tauri::command]
pub fn add_local_share(
    state: State<'_, TauriAppState>,
    file_path: String,
    title: Option<String>,
) -> Result<ShareItemDto, String> {
    let item = state
        .server_state
        .registry
        .register(&file_path, title, None)?;
    Ok(format_share_dto(item, &state.server_state))
}

// 3. Update share title/description
#[tauri::command]
pub fn update_share(
    state: State<'_, TauriAppState>,
    id: String,
    title: Option<String>,
    description: Option<String>,
) -> Result<ShareItemDto, String> {
    let item = state
        .server_state
        .registry
        .update_meta(&id, title, description)?;
    Ok(format_share_dto(item, &state.server_state))
}

// 4. Revoke share
#[tauri::command]
pub fn revoke_share(state: State<'_, TauriAppState>, id: String) -> Result<bool, String> {
    Ok(state.server_state.registry.revoke(&id))
}

// 5. Shorten public URL
#[tauri::command]
pub async fn shorten_share_url(
    state: State<'_, TauriAppState>,
    id: String,
) -> Result<String, String> {
    let item = state
        .server_state
        .registry
        .get(&id)
        .ok_or_else(|| "Share not found".to_string())?;

    let dto = format_share_dto(item, &state.server_state);
    let short = shortener::shorten_url(&dto.public_url).await?;
    state.server_state.registry.set_short_url(&id, short.clone());
    Ok(short)
}

// 6. Get status
#[tauri::command]
pub fn get_status(state: State<'_, TauriAppState>) -> Result<ServerStatusResponse, String> {
    let tunnel = crate::server::TunnelStatusDto {
        status: state.server_state.tunnel_status.read().unwrap().clone(),
        public_url: state.server_state.tunnel_url.read().unwrap().clone(),
        error_message: state.server_state.tunnel_error.read().unwrap().clone(),
    };

    Ok(ServerStatusResponse {
        server: "running".to_string(),
        port: state.server_state.port,
        lan_ip: crate::server::get_local_ip(),
        tunnel,
    })
}

// 7. Get app settings
#[tauri::command]
pub fn get_app_settings(state: State<'_, TauriAppState>) -> Result<AppSettings, String> {
    Ok(state.app_settings.read().unwrap().clone())
}

// 8. Save app settings
#[tauri::command]
pub fn save_app_settings(
    state: State<'_, TauriAppState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    settings.save(&state.settings_path)?;
    *state.app_settings.write().unwrap() = settings.clone();

    // Sync Windows Context Menu
    if settings.context_menu {
        if let Ok(exe) = std::env::current_exe() {
            let _ = windows::register_context_menu(&exe.to_string_lossy());
        }
    } else {
        let _ = windows::unregister_context_menu();
    }

    Ok(settings)
}

// 9. File dialog
#[tauri::command]
pub async fn pick_file_dialog() -> Result<Option<String>, String> {
    let file = rfd::AsyncFileDialog::new()
        .set_title("เลือกไฟล์ที่ต้องการแชร์ (Select File to Share)")
        .pick_file()
        .await;

    Ok(file.map(|f| f.path().to_string_lossy().to_string()))
}

// 10. Show in folder
#[tauri::command]
pub fn show_in_folder(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if let Some(parent) = p.parent() {
        let _ = open::that(parent);
    } else {
        let _ = open::that(p);
    }
    Ok(())
}

// 11. Retry tunnel
#[tauri::command]
pub async fn retry_tunnel(state: State<'_, TauriAppState>) -> Result<(), String> {
    let tunnel_mgr = state.tunnel_manager.clone();
    let port = state.server_state.port;
    tokio::spawn(async move {
        tunnel_mgr.start(port).await;
    });
    Ok(())
}

// 12. Get pending share files from launch args or second instance
#[tauri::command]
pub fn get_pending_shares(state: State<'_, TauriAppState>) -> Vec<String> {
    if let Ok(mut pending) = state.pending_shares.lock() {
        let items = pending.clone();
        pending.clear();
        items
    } else {
        Vec::new()
    }
}
