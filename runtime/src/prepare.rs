use rquest as reqwest;
use reqwest::tls::Impersonate;
use reqwest::{ClientBuilder, header::HeaderMap, Version, StatusCode};
use reqwest::header::HeaderValue;
use serde::Deserialize;
use std::process;
use serde_json::Value;
use anyhow::Result;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use std::io::Read;

// Struct to represent model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub price: i64,
    pub stock: i64,
    pub modelid: i64,
    pub promotionid: i64,
}
#[derive(Debug, Clone)]
pub struct ShippingInfo {
    pub original_cost: i64,
    pub channelid: i64,
    pub channel_name: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PaymentInfo {
    pub name: String,
    pub channel_id: i64,
    pub option_info: String,
    pub version: i64,
    pub txn_fee: i64,
    pub selected_get: Value,
    pub place_order: Value,
}

pub struct ProductInfo {
    pub shop_id: i64,
    pub item_id: i64,
}

pub async fn open_payment_file() -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open("payment.txt").await?;
    let mut json_data = String::new();
    file.read_to_string(&mut json_data).await?;
    Ok(json_data)
}

pub async fn get_payment(json_data: &str) -> Result<Vec<PaymentInfo>, Box<dyn std::error::Error>> {
    let hasil: Value = serde_json::from_str(&json_data)?;

    if let Some(data) = hasil.get("data").and_then(|data| data.as_array()) {
        let payment_info_vec: Vec<PaymentInfo> = data
            .iter()
            .flat_map(|entry| entry.get("payment").and_then(|payment| payment.as_array()))
            .flat_map(|payment_infos| {
                payment_infos.iter().map(|payment_info| {
                    let name = payment_info.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
                    let channel_id = payment_info.get("channelId").and_then(|id| id.as_str().and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
                    let option_info = payment_info.get("optionInfo").and_then(|info| info.as_str()).unwrap_or("Unknown");
                    let version = payment_info.get("version").and_then(|v| v.as_str().and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
                    let txn_fee = payment_info.get("txnFee").and_then(|fee| fee.as_i64()).unwrap_or(0);
                    let selected_get = payment_info.get("get").unwrap_or(&serde_json::Value::Null);
                    let place_order = payment_info.get("place_order").unwrap_or(&serde_json::Value::Null);

                    PaymentInfo {
                        name: name.to_string(),
                        channel_id,
                        option_info: option_info.to_string(),
                        version,
                        txn_fee,
                        selected_get: selected_get.clone(),
                        place_order: place_order.clone(),
                    }
                })
            })
            .collect();

        return Ok(payment_info_vec);
    }

    // Handle the case where there is an error or no payment information is found
	process::exit(1);
}
pub async fn kurir(cookie_content: &str, product_info: &ProductInfo, state: &str, city: &str, district: &str) -> Result<Vec<ShippingInfo>, Box<dyn std::error::Error>> {
	let headers = create_headers(&cookie_content);
	let city_encoded = city.replace(" ", "%20");
    let district_encoded = district.replace(" ", "%20");
    let state_encoded = state.replace(" ", "%20");
	println!("{}-{}-{}", state_encoded, city_encoded, district_encoded);

	let url2 = format!("https://shopee.co.id/api/v4/pdp/get_shipping_info?city={}&district={}&itemid={}&shopid={}&state={}", city_encoded, district_encoded, product_info.item_id, product_info.shop_id, state_encoded);
	println!("{}", url2);
    // Buat klien HTTP
	let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .impersonate(Impersonate::Chrome127)
        .enable_ech_grease(true)
        .permute_extensions(true)
        .gzip(true)
        //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
        .build()?;

    // Buat permintaan HTTP POST
    let response = client
        .get(&url2)
        .headers(headers)
        .version(Version::HTTP_2) 
        .send()
        .await?;

	println!("Status: get_courier");
    //println!("Headers: {:#?}", response.headers());
    let body = response.text().await?;
    //println!("Body: {}", String::from_utf8_lossy(&body));
    
    let hasil: Value = serde_json::from_str(&body)?;
    if let Some(data) = hasil.get("data") {
        if let Some(shipping_infos) = data.get("shipping_infos").and_then(|infos| infos.as_array()) {
            let shipping_info_vec: Vec<ShippingInfo> = shipping_infos
                .iter()
                .map(|shipping_info| {
                    let original_cost = shipping_info.get("original_cost").and_then(|c| c.as_i64()).unwrap_or(0);
                    // Access channel information
                    if let Some(channel_info) = shipping_info.get("channel") {
                        let channelid = channel_info.get("channelid").and_then(|id| id.as_i64()).unwrap_or(0);
                        let channel_name = channel_info.get("name").and_then(|name| name.as_str()).unwrap_or("Unknown");
                        ShippingInfo {
                            original_cost: original_cost,
                            channelid: channelid,
                            channel_name: channel_name.to_string(),
                        }
                    } else {
                        // If channel information is not available, return a default value
                        ShippingInfo {
                            original_cost: 0,
                            channelid: 0,
                            channel_name: "Unknown".to_string(),
                        }
                    }
                })
                .collect::<Vec<_>>();  // Remove the semicolon here

            return Ok(shipping_info_vec);
        }
    }
    // Handle the case where there is an error or no shipping information is found
    process::exit(1);
}
pub async fn get_product(product_info: &ProductInfo, cookie_content: &str) -> Result<(String, Vec<ModelInfo>, String, String), Box<dyn std::error::Error>> {
    let csrftoken = extract_csrftoken(&cookie_content);
    let refe = format!("https://shopee.co.id/product/{}/{}", product_info.shop_id, product_info.item_id);
    let url2 = format!("https://shopee.co.id/api/v4/item/get?itemid={}&shopid={}", product_info.item_id, product_info.shop_id);
    println!("{}", url2);
    println!("sending Get Shopee request...");
	
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("Android app Shopee appver=29339 app_type=1"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("x-shopee-language", HeaderValue::from_static("id"));
    headers.insert("if-none-match-", HeaderValue::from_static("55b03-8476c83de1a4cf3b74cc77b08ce741f9"));
    headers.insert("x-api-source", HeaderValue::from_static("rn"));
    headers.insert("origin", HeaderValue::from_static("https://shopee.co.id"));
    headers.insert("referer", reqwest::header::HeaderValue::from_str(&refe)?);
    headers.insert("accept-language", HeaderValue::from_static("id-ID,id;q=0.9,en-US;q=0.8,en;q=0.7,fr;q=0.6,es;q=0.5"));
    headers.insert("x-csrftoken", reqwest::header::HeaderValue::from_str(csrftoken)?);
    headers.insert("cookie", reqwest::header::HeaderValue::from_str(cookie_content)?);

    // Buat klien HTTP
	let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .impersonate_without_headers(Impersonate::Chrome130)
        .enable_ech_grease(true)
        .permute_extensions(true)
        .gzip(true)
        //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
        .build()?;

    // Buat permintaan HTTP POST
    let response = client
        .get(&url2)
        .header("Content-Type", "application/json")
        .headers(headers)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    let mut models_info = Vec::new();
    let mut name = "INVALID".to_string();
    let mut is_official_shop = false;

    //println!("Status: {}", response.status());
    //println!("Headers: {:#?}", response.headers());
    let status_code = response.status().to_string();
    let body = response.text().await?;
    //println!("Body: {}", &body);

    let hasil: Value = serde_json::from_str(&body)?;
    if let Some(data) = hasil.get("data") {
        name = data.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown").to_string();

        models_info = if let Some(models_array) = data.get("models").and_then(|m| m.as_array()) {
            models_array
                .iter()
                .map(|model| {
                    let model_name = model.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
                    let model_price = model.get("price").and_then(|p| p.as_i64()).unwrap_or(0);
                    let model_stock = model.get("stock").and_then(|s| s.as_i64()).unwrap_or(0);
                    let model_modelid = model.get("modelid").and_then(|m| m.as_i64()).unwrap_or(0);
                    let model_promotionid = model.get("promotionid").and_then(|p| p.as_i64()).unwrap_or(0);

                    ModelInfo {
                        name: model_name.to_string(),
                        price: model_price,
                        stock: model_stock,
                        modelid: model_modelid,
                        promotionid: model_promotionid,
                    }
                })
                .collect::<Vec<_>>()
        } else {
            vec![ModelInfo {
                name: "Unknown".to_string(),
                price: 0,
                stock: 0,
                modelid: 0,
                promotionid: 0,
            }]
        };
        is_official_shop = data.get("is_official_shop").and_then(|i| i.as_bool()).unwrap_or(false);
    } else {
        println!("Status: {}", status_code);
    }
	Ok((name.to_string(), models_info, is_official_shop.to_string(), status_code))
}
pub async fn address(cookie_content: &str) -> Result<(String, String, String, String), Box<dyn std::error::Error>> {
	let headers = create_headers(&cookie_content);
	let url2 = format!("https://shopee.co.id/api/v4/account/address/get_user_address_list");
	println!("{}", url2);
	let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .impersonate(Impersonate::Chrome127)
        .enable_ech_grease(true)
        .permute_extensions(true)
        .gzip(true)
        //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
        .build()?;

    // Buat permintaan HTTP POST
    let response = client
        .get(&url2)
        .headers(headers)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    //println!("Headers: {:#?}", response.headers());
    let status = response.status();
    let body = response.text().await?;
    //println!("Body: {}", String::from_utf8_lossy(&body));
    
    let hasil: Value = serde_json::from_str(&body)?;
    // Access specific values using serde_json::Value methods
    if let Some(data) = hasil.get("data") {
        if let Some(addresses) = data.get("addresses").and_then(|addr| addr.as_array()) {
            for address in addresses {
                let id = address.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
                let state = address.get("state").and_then(|s| s.as_str()).unwrap_or("Unknown");
                let city = address.get("city").and_then(|c| c.as_str()).unwrap_or("Unknown");
                let district = address.get("district").and_then(|d| d.as_str()).unwrap_or("Unknown");
                return Ok((state.to_string(), city.to_string(), district.to_string(), id.to_string()));
            }
        }
    // If the loop doesn't execute (zero elements in addresses), return a default value
    Ok(("LOGOUT (WARNING)".to_string(), "LOGOUT (WARNING)".to_string(), "LOGOUT (WARNING)".to_string(), "LOGOUT (WARNING)".to_string()))
    } else {
        println!("Status: {}", status);
        println!("Harap Ganti akun");
        process::exit(1);
    }
}
pub async fn info_akun(cookie_content: &str) -> Result<(String, String, String), Box<dyn std::error::Error>> {
	let headers = create_headers(&cookie_content);
	let url2 = format!("https://shopee.co.id/api/v4/account/basic/get_account_info");
	println!("{}", url2);
	let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .impersonate(Impersonate::Chrome127)
        .enable_ech_grease(true)
        .permute_extensions(true)
        .gzip(true)
        //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
        .build()?;

    // Buat permintaan HTTP POST
    let response = client
        .get(&url2)
        .headers(headers)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    if response.status() == StatusCode::OK {
        //println!("Headers: {:#?}", response.headers());
        let body = response.text().await?;
        //println!("Body: {}", String::from_utf8_lossy(&body));
        
        let hasil: Value = serde_json::from_str(&body)?;
        // Access specific values using serde_json::Value methods
        if let Some(data) = hasil.get("data") {
            let username = data.get("username").and_then(|u| u.as_str()).unwrap_or("LOGOUT (WARNING)");
            let email = data.get("email").and_then(|e| e.as_str()).unwrap_or("LOGOUT (WARNING)");
            let phone = data.get("phone").and_then(|p| p.as_str()).unwrap_or("LOGOUT (WARNING)");
            return Ok((username.to_string(), email.to_string(), phone.to_string()));
        } else {
            Ok(("LOGOUT (WARNING)".to_string(), "LOGOUT (WARNING)".to_string(), "LOGOUT (WARNING)".to_string()))
        }
    } else {
        println!("Status: {}", response.status());
        println!("Harap Ganti akun");
        process::exit(1);
    }
}
pub fn process_url(url: &str) -> ProductInfo {
    let mut shop_id = String::new();
    let mut item_id = String::new();
    if !url.is_empty() {
        if !url.contains("/product/") {
            let split: Vec<&str> = url.split('.').collect();
            if split.len() >= 2 {
                shop_id = split[split.len() - 2].to_string();
                item_id = split[split.len() - 1].split('?').next().unwrap_or("").to_string();
            }
        } else {
            let split2: Vec<&str> = url.split('/').collect();
            if split2.len() >= 2 {
                shop_id = split2[split2.len() - 2].to_string();
                item_id = split2[split2.len() - 1].split('?').next().unwrap_or("").to_string();
            }
        }
    }
    // Konversi ke i64, gunakan 0 jika parsing gagal
    let shop_id = shop_id.parse::<i64>().unwrap_or(0);
    let item_id = item_id.parse::<i64>().unwrap_or(0);

    ProductInfo { shop_id, item_id }
}
fn create_headers(cookie_content: &str) -> HeaderMap {
    let csrftoken = extract_csrftoken(&cookie_content);
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("sec-ch-ua", HeaderValue::from_static("\"Chromium\";v=\"127\", \"Not)A;Brand\";v=\"24\", \"Google Chrome\";v=\"127\""));
    headers.insert("x-shopee-language", HeaderValue::from_static("id"));
    headers.insert("x-requested-with", HeaderValue::from_static("XMLHttpRequest"));
    headers.insert("x-csrftoken", HeaderValue::from_str(&csrftoken).unwrap());
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Windows\""));
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
	headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36"));
    headers.insert("x-api-source", HeaderValue::from_static("pc"));
    headers.insert("accept", HeaderValue::from_static("*/*"));
    headers.insert("origin", HeaderValue::from_static("https://shopee.co.id"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    headers.insert("accept-language", HeaderValue::from_static("id-ID,id;q=0.9,en-US;q=0.8,en;q=0.7,fr;q=0.6,es;q=0.5"));
    headers.insert("cookie", HeaderValue::from_str(cookie_content).unwrap());
    // Return the created headers
    headers
}
pub fn extract_csrftoken(cookie_string: &str) -> &str {
    let mut csrftoken = " ";

    if let Some(token_index) = cookie_string.find("csrftoken=") {
        let token_start = token_index + "csrftoken=".len();
        if let Some(token_end) = cookie_string[token_start..].find(';') {
            csrftoken = &cookie_string[token_start..token_start + token_end];
        }
    }
    csrftoken
}
pub fn read_cookie_file(file_name: &str) -> String {
    let file_path = format!("./akun/{}", file_name);
    let file = std::fs::File::open(&file_path);
    let mut cookie_content = String::new();
    let _ = file.expect("REASON").read_to_string(&mut cookie_content);
    let trimmed_content = cookie_content.trim().to_string();
    if trimmed_content.is_empty() {
        " ".to_string()
    } else {
        trimmed_content
    }
}