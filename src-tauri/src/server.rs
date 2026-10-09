use axum::{
    extract::{Path as AxumPath, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tower_http::cors::{Any, CorsLayer};

use crate::registry::{FileRegistry, ShareItem};
use crate::stream::build_stream_response;

pub type ShareNotifier = Arc<dyn Fn(&ShareItemDto) + Send + Sync + 'static>;
pub type FocusNotifier = Arc<dyn Fn() + Send + Sync + 'static>;

#[derive(Clone)]
pub struct AppState {
    pub registry: FileRegistry,
    pub port: u16,
    pub tunnel_url: Arc<RwLock<Option<String>>>,
    pub tunnel_status: Arc<RwLock<String>>, // "offline" | "starting" | "online" | "error"
    pub tunnel_error: Arc<RwLock<Option<String>>>,
    pub settings: Option<Arc<RwLock<crate::settings::AppSettings>>>,
    pub on_share_added: Option<ShareNotifier>,
    pub on_focus_requested: Option<FocusNotifier>,
    pub thumbnail_cache: Arc<RwLock<std::collections::HashMap<String, Option<Vec<u8>>>>>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareItemDto {
    #[serde(flatten)]
    pub item: ShareItem,
    pub public_url: String,
    pub local_url: String,
    pub thumbnail_url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatusResponse {
    pub server: String,
    pub port: u16,
    pub lan_ip: String,
    pub tunnel: TunnelStatusDto,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelStatusDto {
    pub status: String,
    pub public_url: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddLocalShareRequest {
    pub file_path: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateShareRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub direct_download: Option<bool>,
}

pub fn get_local_ip() -> String {
    local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let k = 1024.0;
    let sizes = ["B", "KB", "MB", "GB", "TB"];
    let i = (bytes as f64).log(k).floor() as usize;
    let i = i.min(sizes.len() - 1);
    let val = (bytes as f64) / k.powi(i as i32);
    format!("{:.2} {}", val, sizes[i])
}

pub fn format_share_dto(item: ShareItem, state: &AppState) -> ShareItemDto {
    let public_base = state
        .tunnel_url
        .read()
        .unwrap()
        .clone()
        .unwrap_or_else(|| format!("http://127.0.0.1:{}", state.port));

    let lan_ip = get_local_ip();
    let local_url = format!("http://{}:{}/d/{}", lan_ip, state.port, item.id);
    let public_url = format!("{}/d/{}", public_base, item.id);
    let thumbnail_url = format!("http://127.0.0.1:{}/thumbnail/{}", state.port, item.id);

    ShareItemDto {
        item,
        public_url,
        local_url,
        thumbnail_url,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItemDto {
    #[serde(flatten)]
    pub item: crate::registry::HistoryItem,
    pub file_exists: bool,
    pub is_active: bool,
    pub active_share_id: Option<String>,
    pub thumbnail_url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteHistoryRequest {
    pub ids: Vec<String>,
}

pub fn format_history_dto(dto: crate::registry::HistoryItemDto, state: &AppState) -> HistoryItemDto {
    let thumbnail_url = format!("http://127.0.0.1:{}/thumbnail/{}", state.port, dto.item.id);
    HistoryItemDto {
        item: dto.item,
        file_exists: dto.file_exists,
        is_active: dto.is_active,
        active_share_id: dto.active_share_id,
        thumbnail_url,
    }
}

pub fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub fn generate_landing_html(share: &ShareItem, base_url: &str) -> String {
    let title = escape_html(&share.title);
    let original_name = escape_html(&share.original_name);
    let size_str = format_bytes(share.size);
    let desc = if share.description.trim().is_empty() {
        format!("{} • ขนาด {}", title, size_str)
    } else {
        escape_html(&share.description)
    };
    let download_url = format!("{}/api/download/{}", base_url, share.id);
    let page_url = format!("{}/d/{}", base_url, share.id);
    let thumbnail_url = format!("{}/thumbnail/{}", base_url, share.id);

    let icon_svg = if share.is_directory {
        r#"<svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
      </svg>"#
    } else {
        r#"<svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
        <polyline points="7 10 12 15 17 10"></polyline>
        <line x1="12" y1="15" x2="12" y2="3"></line>
      </svg>"#
    };

    let badge_text = if share.is_directory {
        format!("📁 โฟลเดอร์ ({} ไฟล์) • ขนาด {}", share.file_count, size_str)
    } else {
        size_str
    };

    let has_desc = !share.description.trim().is_empty();
    let has_diff_orig = share.title != share.original_name;

    let subtitle_html = if has_desc {
        let desc_escaped = escape_html(&share.description);
        if has_diff_orig {
            format!(
                r#"<p class="desc">{}</p><p class="orig-name">{}</p>"#,
                desc_escaped, original_name
            )
        } else {
            format!(r#"<p class="desc">{}</p>"#, desc_escaped)
        }
    } else {
        format!(r#"<p class="orig-name">{}</p>"#, original_name)
    };

    let button_text = if share.is_directory {
        "ดาวน์โหลดโฟลเดอร์ ZIP (Download Folder as ZIP)"
    } else {
        "ดาวน์โหลดไฟล์ทันที (Direct Download)"
    };

    format!(r#"<!DOCTYPE html>
<html lang="th">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title} - ดาวน์โหลดไฟล์</title>

  <!-- Open Graph / Discord / Facebook / LINE -->
  <meta property="og:type" content="website">
  <meta property="og:url" content="{page_url}">
  <meta property="og:title" content="{title}">
  <meta property="og:description" content="{desc}">
  <meta property="og:site_name" content="File Sharer">

  <!-- Twitter Card -->
  <meta name="twitter:card" content="summary">
  <meta name="twitter:title" content="{title}">
  <meta name="twitter:description" content="{desc}">

  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Prompt:wght@400;500;600;700&display=swap" rel="stylesheet">

  <style>
    * {{ box-sizing: border-box; margin: 0; padding: 0; font-family: 'Prompt', -apple-system, sans-serif; }}
    body {{
      background-color: #f8fafc;
      background-image: radial-gradient(circle at 50% -20px, rgba(224, 242, 254, 0.8) 0%, rgba(248, 250, 252, 0) 500px);
      min-height: 100vh;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      padding: 24px;
      color: #0f172a;
    }}
    .card {{
      background: #ffffff;
      border: 1px solid #e2e8f0;
      border-radius: 20px;
      box-shadow: 0 10px 30px -4px rgba(14, 165, 233, 0.1), 0 4px 10px -2px rgba(0, 0, 0, 0.04);
      max-width: 480px;
      width: 100%;
      padding: 36px 28px;
      text-align: center;
    }}
    .icon-box {{
      width: 76px;
      height: 76px;
      border-radius: 20px;
      background: #f0f9ff;
      border: 1px solid #bae6fd;
      color: #0284c7;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      margin-bottom: 20px;
      box-shadow: 0 4px 12px rgba(2, 132, 199, 0.12);
      overflow: hidden;
      padding: 6px;
      position: relative;
    }}
    .item-thumb {{
      width: 100%;
      height: 100%;
      object-fit: contain;
      border-radius: 14px;
      display: block;
    }}
    .fallback-icon {{
      display: flex;
      align-items: center;
      justify-content: center;
      width: 100%;
      height: 100%;
    }}
    .title {{
      font-size: 20px;
      font-weight: 700;
      color: #0f172a;
      line-height: 1.3;
      margin-bottom: 8px;
      word-break: break-word;
    }}
    .desc {{
      font-size: 14px;
      color: #475569;
      margin-bottom: 16px;
      word-break: break-word;
      line-height: 1.5;
      white-space: pre-wrap;
    }}
    .orig-name {{
      font-size: 13px;
      color: #64748b;
      margin-bottom: 16px;
      word-break: break-all;
    }}
    .desc + .orig-name {{
      margin-top: -6px;
      font-size: 12px;
      color: #94a3b8;
    }}
    .badge {{
      display: inline-block;
      background: #f1f5f9;
      border: 1px solid #e2e8f0;
      padding: 4px 12px;
      border-radius: 20px;
      font-size: 13px;
      font-weight: 600;
      color: #334155;
      margin-bottom: 24px;
    }}
    .btn-download {{
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 10px;
      background: #0284c7;
      color: #ffffff;
      text-decoration: none;
      padding: 14px 24px;
      border-radius: 12px;
      font-weight: 700;
      font-size: 15px;
      box-shadow: 0 4px 14px rgba(2, 132, 199, 0.3);
      transition: all 0.2s ease;
      cursor: pointer;
    }}
    .btn-download:hover {{
      background: #0369a1;
      transform: translateY(-1px);
      box-shadow: 0 6px 18px rgba(2, 132, 199, 0.4);
    }}
    .footer-note {{
      margin-top: 24px;
      font-size: 12px;
      color: #94a3b8;
    }}
  </style>
</head>
<body>
  <div class="card">
    <div class="icon-box">
      <img src="{thumbnail_url}" alt="" class="item-thumb" onerror="this.style.display='none'; document.getElementById('fallback-icon').style.display='flex';" />
      <div id="fallback-icon" class="fallback-icon" style="display: none;">
        {icon_svg}
      </div>
    </div>
    <h1 class="title">{title}</h1>
    {subtitle_html}
    <div class="badge">{badge_text}</div>
    <a href="{download_url}" class="btn-download">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
        <polyline points="7 10 12 15 17 10"></polyline>
        <line x1="12" y1="15" x2="12" y2="3"></line>
      </svg>
      {button_text}
    </a>
    <p class="footer-note">ส่งตรงจากเครื่องผู้แชร์ • รองรับการดาวน์โหลดความเร็วสูง</p>
  </div>
</body>
</html>"#)
}

pub fn generate_not_found_html() -> String {
    r#"<!DOCTYPE html>
<html lang="th">
<head>
  <meta charset="UTF-8">
  <title>ไม่พบไฟล์ - File Sharer</title>
  <style>
    body { font-family: sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; background: #f8fafc; color: #0f172a; }
    .box { text-align: center; padding: 32px; background: white; border-radius: 16px; border: 1px solid #e2e8f0; }
    h1 { color: #ef4444; font-size: 20px; margin-bottom: 8px; }
    p { color: #64748b; font-size: 14px; }
  </style>
</head>
<body>
  <div class="box">
    <h1>ไม่พบไฟล์หรือถูกยกเลิกการแชร์แล้ว</h1>
    <p>ลิงก์นี้ไม่สามารถใช้งานได้แล้ว เนื่องจากผู้แชร์ได้ยกเลิกการแชร์ไฟล์ออกจากระบบ</p>
  </div>
</body>
</html>"#.to_string()
}

// Route handlers
async fn landing_page_handler(
    AxumPath(id): AxumPath<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Some(share) = state.registry.get(&id) {
        state.registry.increment_views(&id);

        if share.direct_download {
            return (
                StatusCode::FOUND,
                [(header::LOCATION, format!("/api/download/{}", share.id))],
                Html(String::new()),
            ).into_response();
        }

        let host = headers
            .get(header::HOST)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("localhost");

        let proto = headers
            .get("x-forwarded-proto")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("http");

        let base_url = format!("{}://{}", proto, host);
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            Html(generate_landing_html(&share, &base_url)),
        ).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            Html(generate_not_found_html()),
        ).into_response()
    }
}

async fn download_handler(
    AxumPath(id): AxumPath<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Some(share) = state.registry.get(&id) {
        state.registry.increment_downloads(&id);

        if share.is_directory {
            match crate::stream::build_directory_zip_stream(&share.file_path, &share.original_name).await {
                Ok(resp) => resp,
                Err((status, msg)) => (status, msg).into_response(),
            }
        } else {
            let range_header = headers
                .get(header::RANGE)
                .and_then(|val| val.to_str().ok());

            match build_stream_response(&share.file_path, range_header, &share.original_name).await {
                Ok(resp) => resp,
                Err((status, msg)) => (status, msg).into_response(),
            }
        }
    } else {
        (StatusCode::NOT_FOUND, "File share not found or revoked").into_response()
    }
}

async fn list_shares_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let items = state.registry.list();
    let dtos: Vec<ShareItemDto> = items
        .into_iter()
        .map(|item| format_share_dto(item, &state))
        .collect();

    Json(serde_json::json!({ "shares": dtos }))
}

async fn add_local_share_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddLocalShareRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match state
        .registry
        .register(&payload.file_path, payload.title, payload.description)
    {
        Ok(item) => {
            let dto = format_share_dto(item, &state);

            // Auto-copy to native Windows clipboard if auto_copy is true
            let best_link = if let Some(ref su) = dto.item.short_url {
                su.clone()
            } else if !dto.public_url.is_empty() {
                dto.public_url.clone()
            } else {
                dto.local_url.clone()
            };
            if let Some(ref s) = state.settings {
                if s.read().unwrap().auto_copy {
                    let _ = crate::windows::set_clipboard_text(&best_link);
                }
            }

            // Notify GUI and bring window to front
            if let Some(ref cb) = state.on_share_added {
                cb(&dto);
            }

            Ok((StatusCode::CREATED, Json(serde_json::json!({ "share": dto }))))
        }
        Err(err) => Err((StatusCode::BAD_REQUEST, err)),
    }
}

async fn focus_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if let Some(ref cb) = state.on_focus_requested {
        cb();
    }
    StatusCode::OK
}

async fn version_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn terminate_handler() -> impl IntoResponse {
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(150));
        std::process::exit(0);
    });
    StatusCode::OK
}

async fn update_share_handler(
    AxumPath(id): AxumPath<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateShareRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match state.registry.update_meta(&id, payload.title, payload.description, payload.direct_download) {
        Ok(item) => {
            let dto = format_share_dto(item, &state);
            Ok(Json(dto))
        }
        Err(err) => Err((StatusCode::NOT_FOUND, err)),
    }
}

async fn revoke_share_handler(
    AxumPath(id): AxumPath<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let ok = state.registry.revoke(&id);
    if ok {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn restore_share_handler(
    State(state): State<Arc<AppState>>,
    Json(item): Json<crate::registry::ShareItem>,
) -> impl IntoResponse {
    let restored = state.registry.restore(item);
    let dto = format_share_dto(restored, &state);
    (StatusCode::OK, Json(serde_json::json!({ "share": dto })))
}

async fn status_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let tunnel = TunnelStatusDto {
        status: state.tunnel_status.read().unwrap().clone(),
        public_url: state.tunnel_url.read().unwrap().clone(),
        error_message: state.tunnel_error.read().unwrap().clone(),
    };

    let resp = ServerStatusResponse {
        server: "running".to_string(),
        port: state.port,
        lan_ip: get_local_ip(),
        tunnel,
    };

    Json(resp)
}

async fn thumbnail_handler(
    AxumPath(id): AxumPath<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    if let Some(cached) = state.thumbnail_cache.read().unwrap().get(&id) {
        return match cached {
            Some(bytes) => (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "image/png"),
                    (header::CACHE_CONTROL, "public, max-age=86400"),
                ],
                bytes.clone(),
            )
                .into_response(),
            None => (StatusCode::NOT_FOUND, "Thumbnail not found").into_response(),
        };
    }

    let file_path = if let Some(share) = state.registry.get(&id) {
        Some(share.file_path)
    } else if let Some(hist) = state.registry.get_history(&id) {
        Some(hist.file_path)
    } else {
        None
    };

    if let Some(path) = file_path {
        let thumb = crate::windows::get_file_thumbnail(&path, 256);
        state
            .thumbnail_cache
            .write()
            .unwrap()
            .insert(id.clone(), thumb.clone());

        if let Some(bytes) = thumb {
            return (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "image/png"),
                    (header::CACHE_CONTROL, "public, max-age=86400"),
                ],
                bytes,
            )
                .into_response();
        }
    }

    (StatusCode::NOT_FOUND, "Thumbnail not found").into_response()
}

async fn list_history_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let items = state.registry.list_history();
    let dtos: Vec<HistoryItemDto> = items
        .into_iter()
        .map(|it| format_history_dto(it, &state))
        .collect();
    Json(serde_json::json!({ "history": dtos }))
}

async fn delete_history_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DeleteHistoryRequest>,
) -> impl IntoResponse {
    let deleted_count = state.registry.delete_history_items(&payload.ids);
    Json(serde_json::json!({ "deletedCount": deleted_count }))
}

async fn clear_history_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    state.registry.clear_history();
    StatusCode::OK
}

async fn restore_from_history_handler(
    AxumPath(id): AxumPath<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match state.registry.restore_from_history(&id) {
        Ok(share) => {
            let dto = format_share_dto(share, &state);
            Ok(Json(serde_json::json!({ "share": dto })))
        }
        Err(err) => Err((StatusCode::BAD_REQUEST, err)),
    }
}

pub fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/d/:id", get(landing_page_handler))
        .route("/thumbnail/:id", get(thumbnail_handler))
        .route("/api/shares/:id/thumbnail", get(thumbnail_handler))
        .route("/api/download/:id", get(download_handler))
        .route("/api/shares", get(list_shares_handler))
        .route("/api/shares/local", post(add_local_share_handler))
        .route("/api/shares/:id", put(update_share_handler))
        .route("/api/shares/:id", delete(revoke_share_handler))
        .route("/api/shares/restore", post(restore_share_handler))
        .route("/api/history", get(list_history_handler))
        .route("/api/history/delete", post(delete_history_handler))
        .route("/api/history/clear", post(clear_history_handler))
        .route("/api/history/restore/:id", post(restore_from_history_handler))
        .route("/api/status", get(status_handler))
        .route("/api/app/focus", post(focus_handler))
        .route("/api/app/version", get(version_handler))
        .route("/api/app/terminate", post(terminate_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
