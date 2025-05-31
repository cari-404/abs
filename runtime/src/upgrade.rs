use rquest as reqwest;
use serde_json::Value;
use tokio::io;
use std::cmp::Ordering;
use std::sync::Arc;

use crate::prepare;

pub async fn get_latest_release(url: &str) -> Option<String> {
    let client = prepare::universal_client_skip_headers();
    let response = client
        .await.get(url)
        .header("User-Agent", "rust-updater")
        .send()
        .await
        .ok()?;
    
    if response.status().is_success() {
        let json: Value = response.json().await.ok()?;
        json["tag_name"].as_str().map(|s| s.trim_start_matches('v').to_string())
    } else {
        None
    }
}
pub async fn fetch_download_response(client: Arc<reqwest::Client>, url: &str) -> Result<(reqwest::Response, u64), io::Error> {
    let response = client.get(url).send().await.map_err(|e| {
        eprintln!("Gagal mengunduh: {}", e);
        io::Error::new(io::ErrorKind::Other, "Gagal mengunduh file")
    })?;

    let total_size = response.content_length().unwrap_or(0);
    println!("Status: {}", response.status());
    Ok((response, total_size))
}
pub fn compare_versions(local: &str, remote: &str) -> Ordering {
    let parse = |s: &str| s.split('.').filter_map(|p| p.parse::<u32>().ok()).collect::<Vec<_>>();
    let local_parts = parse(local);
    let remote_parts = parse(remote);
    local_parts.cmp(&remote_parts)
}
