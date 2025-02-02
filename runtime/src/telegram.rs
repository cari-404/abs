use rquest as reqwest;
use reqwest::tls::Impersonate;
use reqwest::{ClientBuilder, Version, StatusCode};
use serde::Deserialize;
use urlencoding::encode as url_encode;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Deserialize, Debug, Clone)]
pub struct TeleInfo {
    pub telegram_notif: bool,
    pub telegram_token: String,
    pub telegram_chat_id: String,
}

pub async fn save_config_file(content: String) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "config.json";
    tokio::fs::write(file_path, content).await?;
    Ok(())
}
pub async fn open_config_file() -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open("config.json").await?;
    let mut json_data = String::new();
    file.read_to_string(&mut json_data).await?;
    Ok(json_data)
}
pub async fn get_config(json_data: &str) -> Result<TeleInfo, Box<dyn std::error::Error>> {
    let response: TeleInfo = serde_json::from_str(&json_data)?;
    Ok(response)
}

pub fn get_data(token: &str, chat_id: &str) -> TeleInfo {
    TeleInfo{
        telegram_notif: false,
        telegram_token: token.to_string(),
        telegram_chat_id: chat_id.to_string(),
    }
}

pub async fn send_msg(data: &TeleInfo, msg: &str) -> Result<(), reqwest::Error> {
    let msg_encoded = url_encode(msg);
    let url2 = format!("https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}", data.telegram_token, data.telegram_chat_id, msg_encoded);
	let client = ClientBuilder::new()
        .http2_keep_alive_while_idle(true)
        .danger_accept_invalid_certs(true)
        .impersonate(Impersonate::Chrome130)
        .enable_ech_grease(true)
        .permute_extensions(true)
        .gzip(true)
        //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
        .build()?;

    // Buat permintaan HTTP POST
    let response = client
        .get(&url2)
        .version(Version::HTTP_2) 
        .send()
        .await?;
    
    if response.status() == StatusCode::OK {
        Ok(())
    }else{
        Err(response.error_for_status().unwrap_err())
    }
}