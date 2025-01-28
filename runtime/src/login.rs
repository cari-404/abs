use rquest as reqwest;
use reqwest::tls::Impersonate;
use reqwest::{ClientBuilder, Version};
use serde::{Deserialize};
use serde_json::{Value};
use anyhow::Result;
use urlencoding::encode as url_encode;

#[derive(Deserialize, Debug)]
pub struct DataQRCode {
    pub qrcode_id: String,
    pub qrcode_id_encoded: String,
    pub qrcode_base64: String,
}

pub async fn get_qrcode() -> Result<DataQRCode, Box<dyn std::error::Error>> {
	let url2 = format!("https://shopee.co.id/api/v2/authentication/gen_qrcode");
	println!("{}", url2);
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

    let qr_data: Value = response.json().await?;
    let qrcode_id = qr_data["data"]["qrcode_id"].as_str().unwrap_or("");
    let qrcode_id_encoded = url_encode(&qrcode_id);
    let qrcode_base64 = qr_data["data"]["qrcode_base64"].as_str().unwrap_or("");
    Ok(DataQRCode {
        qrcode_id: qrcode_id.to_string(),
        qrcode_id_encoded: qrcode_id_encoded.to_string(),
        qrcode_base64: qrcode_base64.to_string(),
    })
}
pub async fn authentication_qrcode(qrcode_data: &DataQRCode) -> Result<(String, String), Box<dyn std::error::Error>> {
	let url2 = format!("https://shopee.co.id/api/v2/authentication/qrcode_status?qrcode_id={}", qrcode_data.qrcode_id_encoded);
	println!("{}", url2);
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

    let text = response.text().await?;
    // Parse the text as JSON
    let resp: Value = serde_json::from_str(&text)?;
    println!("text: {}", text);
    let status = resp["data"]["status"].as_str().unwrap_or("").to_string();
    let qrcode_token = resp["data"]["qrcode_token"].as_str().unwrap_or("").to_string();
    Ok((status, qrcode_token))
}
pub async fn get_cookie(qrcode_token: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url2 = format!("https://shopee.co.id/api/v2/authentication/qrcode_login");
    println!("{}", url2);
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
        .post(&url2)
        .json(&serde_json::json!({
            "qrcode_token": qrcode_token,
            "device_sz_fingerprint": "",
            "client_identifier": { "security_device_fingerprint": "" }
        }))
        .version(Version::HTTP_2) 
        .send()
        .await?;

    let mut cookies = String::new();
    if let Some(set_cookie) = response.headers().get("set-cookie") {
        println!("Success getting cookies.");
        cookies = set_cookie.to_str()?.to_string();
    }
    println!("Cookies: {}", cookies);
    Ok(cookies)
}