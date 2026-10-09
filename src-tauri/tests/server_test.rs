use axum::body::to_bytes;
use axum::http::{Request, StatusCode};
use local_file_sharer_lib::registry::FileRegistry;
use local_file_sharer_lib::server::{create_app, AppState};
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, RwLock};
use tempfile::tempdir;
use tower::ServiceExt;

#[tokio::test]
async fn test_landing_page_and_download_routes() {
    let dir = tempdir().unwrap();
    let sample_file = dir.path().join("archive.zip");
    let mut f = File::create(&sample_file).unwrap();
    f.write_all(b"PK0304zipcontent").unwrap();

    let storage_path = dir.path().join("shares.json");
    let registry = FileRegistry::new(storage_path);
    let share = registry
        .register(sample_file.to_str().unwrap(), Some("My Archive".to_string()), None)
        .unwrap();

    let state = Arc::new(AppState {
        registry: registry.clone(),
        port: 3030,
        tunnel_url: Arc::new(RwLock::new(Some("https://mock.trycloudflare.com".to_string()))),
        tunnel_status: Arc::new(RwLock::new("online".to_string())),
        tunnel_error: Arc::new(RwLock::new(None)),
        settings: None,
        on_share_added: None,
        on_focus_requested: None,
        thumbnail_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
    });

    let app = create_app(state);

    // 1. Test landing page GET /d/:id
    let req = Request::builder()
        .uri(format!("/d/{}", share.id))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(body_str.contains("My Archive"));
    assert!(body_str.contains("archive.zip"));
    assert!(body_str.contains("/api/download/"));

    // Verify views incremented
    let updated = registry.get(&share.id).unwrap();
    assert_eq!(updated.views, 1);

    // 2. Test download endpoint GET /api/download/:id
    let dl_req = Request::builder()
        .uri(format!("/api/download/{}", share.id))
        .body(axum::body::Body::empty())
        .unwrap();

    let dl_resp = app.clone().oneshot(dl_req).await.unwrap();
    assert_eq!(dl_resp.status(), StatusCode::OK);
    assert_eq!(
        dl_resp.headers().get("content-disposition").unwrap(),
        "attachment; filename=\"archive.zip\""
    );

    // Verify downloads incremented
    let updated2 = registry.get(&share.id).unwrap();
    assert_eq!(updated2.downloads, 1);

    // 3. Test non-existent share returns 404
    let not_found_req = Request::builder()
        .uri("/d/f_nonexistent")
        .body(axum::body::Body::empty())
        .unwrap();
    let nf_resp = app.clone().oneshot(not_found_req).await.unwrap();
    assert_eq!(nf_resp.status(), StatusCode::NOT_FOUND);

    // 4. Test thumbnail route GET /thumbnail/:id
    let thumb_req = Request::builder()
        .uri(format!("/thumbnail/{}", share.id))
        .body(axum::body::Body::empty())
        .unwrap();
    let thumb_resp = app.clone().oneshot(thumb_req).await.unwrap();
    // archive.zip thumbnail is either 200 (if shell generates icon) or 404
    assert!(thumb_resp.status() == StatusCode::OK || thumb_resp.status() == StatusCode::NOT_FOUND);

    // 5. Test revoking and then restoring via POST /api/shares/restore
    registry.revoke(&share.id);
    assert!(registry.get(&share.id).is_none());

    let restore_body = serde_json::to_vec(&share).unwrap();
    let restore_req = Request::builder()
        .method("POST")
        .uri("/api/shares/restore")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(restore_body))
        .unwrap();
    let restore_resp = app.oneshot(restore_req).await.unwrap();
    assert_eq!(restore_resp.status(), StatusCode::OK);
    assert!(registry.get(&share.id).is_some());
}

#[tokio::test]
async fn test_landing_page_renders_description() {
    let dir = tempdir().unwrap();
    let sample_file = dir.path().join("Claude Setup.exe");
    let mut f = File::create(&sample_file).unwrap();
    f.write_all(b"exe content").unwrap();

    let storage_path = dir.path().join("shares.json");
    let registry = FileRegistry::new(storage_path);
    let share = registry
        .register(
            sample_file.to_str().unwrap(),
            Some("Claude Setup.exe".to_string()),
            Some("ไฟล์ดาวโหลด".to_string()),
        )
        .unwrap();

    let state = Arc::new(AppState {
        registry: registry.clone(),
        port: 3030,
        tunnel_url: Arc::new(RwLock::new(None)),
        tunnel_status: Arc::new(RwLock::new("offline".to_string())),
        tunnel_error: Arc::new(RwLock::new(None)),
        settings: None,
        on_share_added: None,
        on_focus_requested: None,
        thumbnail_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
    });

    let app = create_app(state);

    let req = Request::builder()
        .uri(format!("/d/{}", share.id))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(body_str.contains("Claude Setup.exe"));
    assert!(body_str.contains("ไฟล์ดาวโหลด"));
    assert!(body_str.contains(r#"<p class="desc">ไฟล์ดาวโหลด</p>"#));
}

