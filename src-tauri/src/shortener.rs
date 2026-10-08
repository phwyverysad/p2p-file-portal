use std::time::Duration;

pub async fn shorten_url(original_url: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(6))
        .build()
        .map_err(|e| e.to_string())?;

    // Try da.gd first (very fast, simple GET)
    let dagd_url = format!("https://da.gd/s?url={}", urlencoding::encode(original_url));
    if let Ok(resp) = client.get(&dagd_url).send().await {
        if resp.status().is_success() {
            if let Ok(text) = resp.text().await {
                let trimmed = text.trim();
                if trimmed.starts_with("http") {
                    return Ok(trimmed.to_string());
                }
            }
        }
    }

    // Try spoo.me fallback
    let spoo_res = client
        .post("https://spoo.me")
        .header("Accept", "application/json")
        .form(&[("url", original_url)])
        .send()
        .await;

    if let Ok(resp) = spoo_res {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(short) = json.get("short_url").and_then(|v| v.as_str()) {
                    return Ok(short.to_string());
                }
            }
        }
    }

    // If both fail or network offline, return original URL
    Ok(original_url.to_string())
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut encoded = String::new();
        for b in s.bytes() {
            match b {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(b as char)
                }
                _ => encoded.push_str(&format!("%{:02X}", b)),
            }
        }
        encoded
    }
}
