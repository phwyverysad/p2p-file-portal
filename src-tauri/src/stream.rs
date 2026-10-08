use axum::{
    body::Body,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use std::io::SeekFrom;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncSeekExt;
use tokio_util::io::ReaderStream;
use futures_util::StreamExt;

pub fn parse_range_header(range_str: &str, total_size: u64) -> Option<(u64, u64)> {
    if total_size == 0 {
        return None;
    }

    let range_str = range_str.trim();
    if !range_str.starts_with("bytes=") {
        return None;
    }

    let parts = &range_str["bytes=".len()..];
    // Handle first range if multiple comma-separated ranges requested
    let part = parts.split(',').next()?.trim();
    let mut split = part.split('-');

    let start_str = split.next()?.trim();
    let end_str = split.next()?.trim();

    if start_str.is_empty() {
        // Suffix range: bytes=-500
        let suffix_len: u64 = end_str.parse().ok()?;
        if suffix_len == 0 {
            return None;
        }
        let start = total_size.saturating_sub(suffix_len);
        let end = total_size - 1;
        Some((start, end))
    } else if end_str.is_empty() {
        // Prefix range: bytes=500-
        let start: u64 = start_str.parse().ok()?;
        if start >= total_size {
            return None;
        }
        let end = total_size - 1;
        Some((start, end))
    } else {
        // Explicit range: bytes=500-1000
        let start: u64 = start_str.parse().ok()?;
        let end: u64 = end_str.parse().ok()?;
        if start > end || start >= total_size {
            return None;
        }
        let end = end.min(total_size - 1);
        Some((start, end))
    }
}

pub async fn build_stream_response(
    file_path: &str,
    range_header: Option<&str>,
    download_name: &str,
) -> Result<Response, (StatusCode, String)> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    let mut file = File::open(path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to open file: {}", e),
        )
    })?;

    let metadata = file.metadata().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read metadata: {}", e),
        )
    })?;

    let total_size = metadata.len();
    let mime_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    let safe_download_name = download_name.replace('"', "_");
    let content_disposition = format!("attachment; filename=\"{}\"", safe_download_name);

    if let Some(range_val) = range_header {
        if let Some((start, end)) = parse_range_header(range_val, total_size) {
            let chunk_size = end - start + 1;
            file.seek(SeekFrom::Start(start)).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Seek error: {}", e),
                )
            })?;

            let stream = ReaderStream::new(tokio::io::AsyncReadExt::take(file, chunk_size))
                .map(|result| result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));

            let body = Body::from_stream(stream);

            let content_range = format!("bytes {}-{}/{}", start, end, total_size);

            let mut response = (StatusCode::PARTIAL_CONTENT, body).into_response();
            let headers = response.headers_mut();
            headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&mime_type).unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&content_disposition).unwrap_or(HeaderValue::from_static("attachment")),
            );
            headers.insert(
                header::CONTENT_RANGE,
                HeaderValue::from_str(&content_range).unwrap(),
            );
            headers.insert(header::CONTENT_LENGTH, HeaderValue::from(chunk_size));

            return Ok(response);
        } else {
            // Invalid range header -> 416 Range Not Satisfiable
            let mut response = StatusCode::RANGE_NOT_SATISFIABLE.into_response();
            let headers = response.headers_mut();
            let content_range = format!("bytes */{}", total_size);
            headers.insert(
                header::CONTENT_RANGE,
                HeaderValue::from_str(&content_range).unwrap(),
            );
            return Ok(response);
        }
    }

    // Full 200 OK stream
    let stream = ReaderStream::new(file)
        .map(|result| result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));
    let body = Body::from_stream(stream);

    let mut response = (StatusCode::OK, body).into_response();
    let headers = response.headers_mut();
    headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime_type).unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&content_disposition).unwrap_or(HeaderValue::from_static("attachment")),
    );
    headers.insert(header::CONTENT_LENGTH, HeaderValue::from(total_size));

    Ok(response)
}

fn collect_files_in_dir(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                result.extend(collect_files_in_dir(&p));
            } else if p.is_file() {
                result.push(p);
            }
        }
    }
    result
}

pub async fn build_directory_zip_stream(
    dir_path: &str,
    download_name: &str,
) -> Result<Response, (StatusCode, String)> {
    let path = Path::new(dir_path);
    if !path.exists() || !path.is_dir() {
        return Err((StatusCode::NOT_FOUND, "Directory not found".to_string()));
    }

    let root_path = path.to_path_buf();
    let files = collect_files_in_dir(&root_path);

    let zip_filename = if download_name.ends_with(".zip") {
        download_name.to_string()
    } else {
        format!("{}.zip", download_name)
    };
    let safe_download_name = zip_filename.replace('"', "_");
    let content_disposition = format!("attachment; filename=\"{}\"", safe_download_name);

    let (writer, reader) = tokio::io::duplex(64 * 1024);
    let stream = ReaderStream::new(reader)
        .map(|res| res.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));
    let body = Body::from_stream(stream);

    tokio::spawn(async move {
        let mut zip_writer = async_zip::tokio::write::ZipFileWriter::with_tokio(writer);

        for file_path in files {
            if let Ok(rel) = file_path.strip_prefix(&root_path) {
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                let entry = async_zip::ZipEntryBuilder::new(rel_str.into(), async_zip::Compression::Deflate);

                if let Ok(mut entry_writer) = zip_writer.write_entry_stream(entry).await {
                    if let Ok(mut file) = tokio::fs::File::open(&file_path).await {
                        use futures_util::AsyncWriteExt;
                        use tokio::io::AsyncReadExt;
                        let mut buf = [0u8; 64 * 1024];
                        while let Ok(n) = file.read(&mut buf).await {
                            if n == 0 {
                                break;
                            }
                            if entry_writer.write_all(&buf[..n]).await.is_err() {
                                break;
                            }
                        }
                    }
                    let _ = entry_writer.close().await;
                }
            }
        }

        let _ = zip_writer.close().await;
    });

    let mut response = (StatusCode::OK, body).into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/zip"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&content_disposition).unwrap_or(HeaderValue::from_static("attachment")),
    );

    Ok(response)
}

