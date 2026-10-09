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
    let dto = format_share_dto(item, &state.server_state);

    // Auto-copy to clipboard natively if enabled
    if state.app_settings.read().unwrap().auto_copy {
        let link = dto.item.short_url.as_ref().unwrap_or(&dto.public_url);
        let _ = windows::set_clipboard_text(link);
    }

    Ok(dto)
}

// 3. Update share title/description/download mode
#[tauri::command]
pub fn update_share(
    state: State<'_, TauriAppState>,
    id: String,
    title: Option<String>,
    description: Option<String>,
    direct_download: Option<bool>,
) -> Result<ShareItemDto, String> {
    let item = state
        .server_state
        .registry
        .update_meta(&id, title, description, direct_download)?;
    Ok(format_share_dto(item, &state.server_state))
}

// 4. Revoke share
#[tauri::command]
pub fn revoke_share(state: State<'_, TauriAppState>, id: String) -> Result<bool, String> {
    Ok(state.server_state.registry.revoke(&id))
}

// 4b. Restore share (Undo deletion)
#[tauri::command]
pub fn restore_share(
    state: State<'_, TauriAppState>,
    item: crate::registry::ShareItem,
) -> Result<ShareItemDto, String> {
    let restored = state.server_state.registry.restore(item);
    Ok(format_share_dto(restored, &state.server_state))
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

// 9.1 Folder dialog
#[tauri::command]
pub async fn pick_folder_dialog() -> Result<Option<String>, String> {
    let folder = rfd::AsyncFileDialog::new()
        .set_title("เลือกโฟลเดอร์ที่ต้องการแชร์ (Select Folder to Share)")
        .pick_folder()
        .await;

    Ok(folder.map(|f| f.path().to_string_lossy().to_string()))
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

// 13. Copy text natively to Windows clipboard
#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    windows::set_clipboard_text(&text)
}

// 14. List history
#[tauri::command]
pub fn list_history(
    state: State<'_, TauriAppState>,
) -> Result<Vec<crate::server::HistoryItemDto>, String> {
    let items = state.server_state.registry.list_history();
    let dtos = items
        .into_iter()
        .map(|it| crate::server::format_history_dto(it, &state.server_state))
        .collect();
    Ok(dtos)
}

// 15. Delete history items
#[tauri::command]
pub fn delete_history_items(
    state: State<'_, TauriAppState>,
    ids: Vec<String>,
) -> Result<usize, String> {
    Ok(state.server_state.registry.delete_history_items(&ids))
}

// 16. Clear all history
#[tauri::command]
pub fn clear_history(state: State<'_, TauriAppState>) -> Result<bool, String> {
    state.server_state.registry.clear_history();
    Ok(true)
}

// 17. Restore share from history
#[tauri::command]
pub fn restore_from_history(
    state: State<'_, TauriAppState>,
    id: String,
) -> Result<ShareItemDto, String> {
    let restored = state.server_state.registry.restore_from_history(&id)?;
    Ok(format_share_dto(restored, &state.server_state))
}
