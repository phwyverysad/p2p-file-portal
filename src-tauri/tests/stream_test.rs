use axum::http::StatusCode;
use local_file_sharer_lib::stream::{build_stream_response, parse_range_header};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_parse_range_header() {
    let size = 1000;

    assert_eq!(parse_range_header("bytes=0-499", size), Some((0, 499)));
    assert_eq!(parse_range_header("bytes=500-", size), Some((500, 999)));
    assert_eq!(parse_range_header("bytes=-200", size), Some((800, 999)));
    assert_eq!(parse_range_header("bytes=0-999", size), Some((0, 999)));
    assert_eq!(parse_range_header("bytes=0-2000", size), Some((0, 999)));

    // Invalid ranges
    assert_eq!(parse_range_header("bytes=1500-", size), None);
    assert_eq!(parse_range_header("bytes=500-200", size), None);
    assert_eq!(parse_range_header("invalid", size), None);
}

#[tokio::test]
async fn test_full_stream_response() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("video.mp4");
    let mut f = File::create(&file_path).unwrap();
    f.write_all(b"01234567890123456789").unwrap(); // 20 bytes

    let response = build_stream_response(file_path.to_str().unwrap(), None, "video.mp4")
        .await
        .expect("should return response");

    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(headers.get("accept-ranges").unwrap(), "bytes");
    assert_eq!(headers.get("content-length").unwrap(), "20");
}

#[tokio::test]
async fn test_range_stream_response() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("sample.bin");
    let mut f = File::create(&file_path).unwrap();
    f.write_all(b"01234567890123456789").unwrap(); // 20 bytes

    let response = build_stream_response(file_path.to_str().unwrap(), Some("bytes=0-9"), "sample.bin")
        .await
        .expect("should return response");

    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    let headers = response.headers();
    assert_eq!(headers.get("accept-ranges").unwrap(), "bytes");
    assert_eq!(headers.get("content-range").unwrap(), "bytes 0-9/20");
    assert_eq!(headers.get("content-length").unwrap(), "10");
}
