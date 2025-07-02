use runtime::prepare;
use chromiumoxide::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::network::Cookie;
use std::time::Duration;
use futures::StreamExt;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let config = BrowserConfig::builder()
        .with_head()
        .viewport(None)
        .build().map_err(|e| anyhow!(e))?;
    let (browser, mut handler) =
        Browser::launch(config).await?;

    let handle = tokio::spawn(async move {
        loop {
            let _event = handler.next().await.unwrap();
        }
    });

    let page = browser.new_page("https://shopee.co.id/cart").await?;
    let pages = browser.pages().await?;
    for p in pages {
        if let Ok(Some(url)) = p.url().await {
            if url.contains("chrome://new-tab-page/") {
                p.close().await?;
            }
        }
    }
    let page_clone = page.clone();
    let monitor = tokio::spawn(async move {
        let client = Arc::new(prepare::universal_client_skip_headers().await);
        loop {
            let cookies: Vec<Cookie> = match page_clone.get_cookies().await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Gagal mengambil cookies: {e}");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };
            let cookie_header = cookies
                .iter()
                .map(|c| format!("{}={}", c.name, c.value))
                .collect::<Vec<_>>()
                .join("; ");

            println!("Cookie Header: {}", cookie_header);

            // Langsung gunakan cookie_header, tidak perlu read_cookie_file
            let cookie_data = prepare::create_cookie(&cookie_header);
            println!("csrftoken: {}", cookie_data.csrftoken);

            let base_headers = prepare::create_headers(&cookie_data);
            match prepare::info_akun(client.clone(), Arc::new(base_headers)).await {
                Ok(user) if user.userid != 0 => {
                    println!("Username: {}", user.username);
                    println!("User ID: {}", user.userid);
                    break;
                }
                Ok(_) => {
                    eprintln!("Gagal mendapatkan informasi akun, pastikan Anda sudah login.");
                }
                Err(e) => {
                    eprintln!("Gagal mendapatkan informasi akun: {e}");
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        Ok::<(), anyhow::Error>(())
    });
    monitor.await??;
    // ...existing code...

    // Ambil semua cookie dari domain

    let _ = handle.await;

    Ok(())
}
