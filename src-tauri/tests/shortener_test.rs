use local_file_sharer_lib::shortener::shorten_url;
use local_file_sharer_lib::tunnel::{find_cloudflared_binary, TunnelManager};

#[tokio::test]
async fn test_shortener_fallback() {
    let original = "http://127.0.0.1:3030/d/f_123456";
    // Returns either a shortened url or original if network offline
    let result = shorten_url(original).await.unwrap();
    assert!(result.starts_with("http"));
}

#[tokio::test]
async fn test_tunnel_manager_state() {
    let mgr = TunnelManager::new();
    assert_eq!(*mgr.status.read().unwrap(), "offline");

    mgr.set_status("starting", None, None);
    assert_eq!(*mgr.status.read().unwrap(), "starting");

    mgr.set_status("online", Some("https://test.trycloudflare.com".to_string()), None);
    assert_eq!(*mgr.status.read().unwrap(), "online");
    assert_eq!(*mgr.public_url.read().unwrap(), Some("https://test.trycloudflare.com".to_string()));

    mgr.stop().await;
    assert_eq!(*mgr.status.read().unwrap(), "offline");
}

#[test]
fn test_find_cloudflared_binary() {
    let bin = find_cloudflared_binary();
    assert!(bin.is_some(), "cloudflared binary should be found in resources or bin");
}
