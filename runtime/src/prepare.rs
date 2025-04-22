use rquest as reqwest;
use reqwest::tls::Impersonate;
use reqwest::{ClientBuilder, header::HeaderMap, Version, StatusCode};
use reqwest::header::HeaderValue;
use reqwest::redirect::Policy as RedirectPolicy;
use serde::{Serialize, Deserialize, Deserializer};
use std::process;
use serde_json::{Value};
use anyhow::Result;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use std::io::Read;
use std::sync::Arc;
use urlencoding::encode as url_encode;
use once_cell::sync::Lazy;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct FSItems {
    pub itemid: i64,
    pub shopid: i64,
    pub modelids: Option<Vec<i64>>,
    pub raw_discount: i64,
    pub price_before_discount: i64,
    pub stock: i64,
    pub hidden_price_display: Option<String>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BodyGetFSItems {
    limit: i64,
    promotionid: i64,
    itemids: Vec<i64>,
    sort_soldout: bool,
    with_dp_items: bool,
}
// Struct to represent model information
#[derive(Deserialize, Debug, Clone)]
pub struct RawModelInfo {
    pub name: String,
    pub price: i64,
    pub stock: i64,
    pub modelid: i64,
    pub promotionid: i64,
}
// Struct to represent model information
#[derive(Deserialize, Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub price: i64,
    pub stock: i64,
    pub modelid: i64,
    pub promotionid: i64,
    pub shop_id: i64, 
    pub item_id: i64,
    pub quantity: i32,
}
#[derive(Deserialize, Debug, Clone)]
pub struct CookieData {
    pub cookie_content: String,
    pub csrftoken: String,
}
#[derive(Deserialize, Debug)]
struct ProductData {
    name: Option<String>,
    models: Option<Vec<RawModelInfo>>,
    is_official_shop: Option<bool>,
    upcoming_flash_sale: Option<FSInfo>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FSInfo {
    pub promotionid: i64,
    pub start_time: i64,
    pub end_time: i64,
}
#[derive(Deserialize, Debug)]
struct RespFS {
    data: Option<RespFSData>,
}
#[derive(Deserialize, Debug)]
struct RespFSData {
    items: Option<Vec<FSItems>>,
}
#[derive(Deserialize, Debug)]
struct GetProduct {
    data: Option<ProductData>,
}
#[derive(Debug, Clone)]
pub struct ShippingInfo {
    pub original_cost: i64,
    pub channelid: i64,
    pub channelidroot: i64,
    pub channel_name: String,
}
#[derive(Debug, Deserialize)]
pub struct KurirResponse {
    pub data: Option<ShippingData>,
}
#[derive(Debug, Deserialize)]
pub struct ShippingData {
    pub shipping_infos: Option<Vec<ShippingInfoRaw>>,
}
#[derive(Debug, Deserialize)]
pub struct ShippingInfoRaw {
    pub original_cost: Option<i64>,
    pub channel: Option<ChannelInfo>,
}
#[derive(Debug, Deserialize)]
pub struct ChannelInfo {
    pub channelid: Option<i64>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaymentInfo {
    #[serde(default = "default_unknown")]
    pub name: String,
    #[serde(deserialize_with = "custom_parse_i64", rename = "channelId", default)]
    pub channel_id: i64,
    #[serde(rename = "optionInfo", default = "default_unknown")]
    pub option_info: String,
    #[serde(deserialize_with = "custom_parse_i64", default)]
    pub version: i64,
    #[serde(deserialize_with = "deserialize_txn_fee", rename = "txnFee", default)]
    pub txn_fee: i64,
    #[serde(rename = "get", default)]
    pub selected_get: serde_json::Value,
    #[serde(default)]
    pub place_order: serde_json::Value,
}

#[derive(Deserialize)]
struct Entry {
    payment: Vec<PaymentInfo>,
}

#[derive(Deserialize)]
struct PaymentData {
    data: Option<Vec<Entry>>,
}
#[derive(Clone, Default, Debug)]
pub struct ProductInfo {
    pub shop_id: i64,
    pub item_id: i64,
}

#[derive(Deserialize, Debug)]
pub struct UserData {
    pub username: String,
    pub email: String,
    pub phone: String,
    pub userid: i64,
}
#[derive(Deserialize, Debug)]
struct InfoAkun {
    data: Option<UserData>,
}

#[derive(Deserialize, Debug)]
struct DataOnAddress {
    addresses: Option<Vec<AddressInfo>>,
}
#[derive(Deserialize, Debug)]
struct AddressResp {
    data: Option<DataOnAddress>,
}
#[derive(Deserialize, Debug, Clone)]
pub struct AddressInfo  {
    pub state: String,
    pub city: String,
    pub district: String,
    pub id: i64,
}
impl Default for AddressInfo {
    fn default() -> Self {
        AddressInfo {
            state: "LOGOUT (WARNING)".to_string(),
            city: "LOGOUT (WARNING)".to_string(),
            district: "LOGOUT (WARNING)".to_string(),
            id: 0,
        }
    }
}

pub static BASE_HEADER: Lazy<HeaderMap> = Lazy::new(|| {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("sec-ch-ua", HeaderValue::from_static("\"Chromium\";v=\"130\", \"Not)A;Brand\";v=\"24\", \"Google Chrome\";v=\"130\""));
    headers.insert("x-shopee-language", HeaderValue::from_static("id"));
    headers.insert("x-requested-with", HeaderValue::from_static("XMLHttpRequest"));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Windows\""));
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
	headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36"));
    headers.insert("x-api-source", HeaderValue::from_static("pc"));
    headers.insert("accept", HeaderValue::from_static("*/*"));
    headers.insert("origin", HeaderValue::from_static("https://shopee.co.id"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    headers.insert("accept-language", HeaderValue::from_static("id-ID,id;q=0.9,en-US;q=0.8,en;q=0.7,fr;q=0.6,es;q=0.5"));
    headers
});
pub static FS_BASE_HEADER: Lazy<HeaderMap> = Lazy::new(|| {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("Android app Shopee appver=29344 app_type=1"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("Accept", HeaderValue::from_static("application/json"));
    headers.insert("Accept-Encoding", HeaderValue::from_static("gzip"));
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("x-shopee-language", HeaderValue::from_static("id"));
    headers.insert("if-none-match-", HeaderValue::from_static("8001"));
    headers.insert("x-api-source", HeaderValue::from_static("rn"));
    headers.insert("origin", HeaderValue::from_static("https://mall.shopee.co.id"));
    headers.insert("af-ac-enc-dat", HeaderValue::from_static(""));
    headers
});
pub static PRODUCT_BASE_HEADER: Lazy<HeaderMap> = Lazy::new(|| {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("Android app Shopee appver=29339 app_type=1"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("x-shopee-language", HeaderValue::from_static("id"));
    headers.insert("if-none-match-", HeaderValue::from_static("55b03-8476c83de1a4cf3b74cc77b08ce741f9"));
    headers.insert("x-api-source", HeaderValue::from_static("rn"));
    headers.insert("origin", HeaderValue::from_static("https://shopee.co.id"));
    headers.insert("accept-language", HeaderValue::from_static("id-ID,id;q=0.9,en-US;q=0.8,en;q=0.7,fr;q=0.6,es;q=0.5"));
    headers
});

fn default_unknown() -> String {
    "Unknown".to_string()
}
fn custom_parse_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<i64>().map_err(serde::de::Error::custom)
}
fn deserialize_txn_fee<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s.parse::<i64>().unwrap_or(0)),  // Jika String, parse ke i64
        Value::Number(n) => Ok(n.as_i64().unwrap_or(0)),        // Jika Number, ambil nilai i64
        _ => Ok(0),                                         // Default 0 jika nilai tidak sesuai
    }
}
pub async fn open_payment_file() -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open("payment.txt").await?;
    let mut json_data = String::new();
    file.read_to_string(&mut json_data).await?;
    Ok(json_data)
}
pub async fn get_payment(json_data: &str) -> Result<Vec<PaymentInfo>, Box<dyn std::error::Error>> {
    let response: PaymentData = serde_json::from_str(&json_data)?;

    if let Some(data) = response.data {
        let payment_info_vec: Vec<PaymentInfo> = data
            .iter()
            .flat_map(|entry| entry.payment.iter())
            .map(|payment_info| {
                let name = payment_info.name.clone();
                let channel_id = payment_info.channel_id;
                let option_info = payment_info.option_info.clone();
                let version = payment_info.version;
                let txn_fee = payment_info.txn_fee;
                let selected_get = payment_info.selected_get.clone();
                let place_order = payment_info.place_order.clone();
    
                PaymentInfo {
                    name,
                    channel_id,
                    option_info,
                    version,
                    txn_fee,
                    selected_get,
                    place_order,
                }
            })
            .collect();
        return Ok(payment_info_vec);
    }
    // Handle the case where there is an error or no payment information is found
	process::exit(1);
}
pub async fn kurir(client: Arc<reqwest::Client>, headers: Arc<HeaderMap>, product_info: &ProductInfo, address_info: &AddressInfo) -> Result<Vec<ShippingInfo>, Box<dyn std::error::Error>> {
	let city_encoded = url_encode(&address_info.city);
    let district_encoded = url_encode(&address_info.district);
    let state_encoded = url_encode(&address_info.state);
	println!("{}-{}-{}", state_encoded, city_encoded, district_encoded);

	let url2 = format!("https://shopee.co.id/api/v4/pdp/get_shipping_info?city={}&district={}&itemid={}&shopid={}&state={}", city_encoded, district_encoded, product_info.item_id, product_info.shop_id, state_encoded);
	println!("{}", url2);
    // Buat permintaan HTTP POST
    let response = client
        .get(&url2)
        .headers((*headers).clone())
        .version(Version::HTTP_2) 
        .send()
        .await?;

	println!("Status: get_courier");
    //println!("Headers: {:#?}", response.headers());
    let hasil: KurirResponse = response.json().await?;
    //println!("Body: {}", String::from_utf8_lossy(&body));
    let shipping_info_vec = hasil.data.and_then(|data| data.shipping_infos).map(|infos| {
            infos.into_iter().map(|shipping_info| {
                    let original_cost = shipping_info.original_cost.unwrap_or(0);
                    let (channelid, channel_name) = shipping_info.channel.map(|channel| {
                            let channelid = channel.channelid.unwrap_or(0);
                            let channel_name = channel.name.unwrap_or_else(|| "Unknown".to_string());
                            (channelid, channel_name)
                        }).unwrap_or((0, "Unknown".to_string()));
                    ShippingInfo {
                        original_cost,
                        channelid,
                        channelidroot: channelid,
                        channel_name,
                    }
                }).collect::<Vec<_>>()
        }).unwrap_or_default();

    if shipping_info_vec.is_empty() {
        eprintln!("No shipping information found.");
        process::exit(1);
    }
    Ok(shipping_info_vec)
}
pub async fn get_flash_sale_batch_get_items(client: Arc<reqwest::Client>, cookie_content: &CookieData, product_info: &[ProductInfo], fs_info: &FSInfo) -> Result<Vec<FSItems>, anyhow::Error> {
    let itemids: Vec<i64> = product_info.iter().map(|p| p.item_id).collect();
    let refe = format!("https://mall.shopee.co.id/bridge_cmd?cmd=reactPath%3Ftab%3Dbuy%26path%3Dshopee%252FHOME_PAGE%253Fis_tab%253Dtrue%2526layout%253D%25255Bobject%252520Object%25255D%2526native_render%253Dsearch_prefills%25252Clanding_page_banners%25252Cwallet_bar%25252Chome_squares%25252Cskinny_banners%25252Cnew_user_zone%25252Ccutline%25252Cfood_order_status");
    let url2 = format!("https://mall.shopee.co.id/api/v4/flash_sale/flash_sale_batch_get_items");
    println!("{}", url2);
    println!("sending Get Shopee request...");
    
    let mut headers = FS_BASE_HEADER.clone();
    headers.insert("referer", reqwest::header::HeaderValue::from_str(&refe)?);
    headers.insert("x-csrftoken", reqwest::header::HeaderValue::from_str(&cookie_content.csrftoken)?);
    headers.insert("cookie", reqwest::header::HeaderValue::from_str(&cookie_content.cookie_content)?);

    let body_json = BodyGetFSItems {
        limit: 16,
        promotionid: fs_info.promotionid,
        itemids,
        sort_soldout: true,
        with_dp_items: false,
    };

    let response = client
        .post(&url2)
        .headers(headers)
        .json(&body_json)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    let status_code = response.status().to_string();
    let hasil: RespFS = response.json().await?;
    //println!("Body: {}", &body);

    let items = if let Some(data) = hasil.data {
        data.items.unwrap_or_default()
    } else {
        println!("Status: {}", status_code);
        Vec::new()
    };
    Ok(items)
}
pub async fn get_product(client: Arc<reqwest::Client>, product_info: &ProductInfo, cookie_content: &CookieData) -> Result<(String, Vec<ModelInfo>, bool, FSInfo, String), anyhow::Error> {
    let url2 = format!("https://shopee.co.id/api/v4/item/get?itemid={}&shopid={}", product_info.item_id, product_info.shop_id);
    println!("{}", url2);
    println!("sending Get Shopee request...");
	
    let mut headers = FS_BASE_HEADER.clone();
    headers.insert("referer", reqwest::header::HeaderValue::from_str(&format!("https://shopee.co.id/product/{}/{}", product_info.shop_id, product_info.item_id))?);
    headers.insert("x-csrftoken", reqwest::header::HeaderValue::from_str(&cookie_content.csrftoken)?);
    headers.insert("cookie", reqwest::header::HeaderValue::from_str(&cookie_content.cookie_content)?);
    // Buat permintaan HTTP POST
    let response = client
        .get(&url2)
        .headers(headers)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    //println!("Status: {}", response.status());
    //println!("Headers: {:#?}", response.headers());
    let status_code = response.status().to_string();
    let hasil: GetProduct = response.json().await?;
    //println!("Body: {}", &body);

    let (name, raw_models, is_official_shop, fs_info) = if let Some(data) = hasil.data {
        (
            data.name.unwrap_or_else(|| "Unknown".into()),
            data.models.unwrap_or_default(),          // bisa kosong
            data.is_official_shop.unwrap_or(false),
            data.upcoming_flash_sale.unwrap_or(FSInfo {
                promotionid: 0,
                start_time: 0,
                end_time: 0,
            }),
        )
    } else {
        println!("Status: {}", status_code);
        ("INVALID".to_string(), Vec::new(), false, FSInfo {
            promotionid: 0,
            start_time: 0,
            end_time: 0,
        })
    };
    let models_info: Vec<ModelInfo> = raw_models
        .into_iter()
        .map(|m| ModelInfo {
            name: m.name,
            price: m.price,
            stock: m.stock,
            modelid: m.modelid,
            promotionid: m.promotionid,
            shop_id:   product_info.shop_id,   // override di *sini*
            item_id:   product_info.item_id,
            quantity:  1,
        })
        .collect();
	Ok((name, models_info, is_official_shop, fs_info, status_code))
}
pub async fn address(client: Arc<reqwest::Client>, headers: Arc<HeaderMap>) -> Result<AddressInfo, Box<dyn std::error::Error>> {
	let url2 = format!("https://shopee.co.id/api/v4/account/address/get_user_address_list");
	println!("{}", url2);
    // Buat permintaan HTTP POST
    let response = client
        .get(&url2)
        .headers((*headers).clone())
        .version(Version::HTTP_2) 
        .send()
        .await?;

    //println!("Headers: {:#?}", response.headers());
    if response.status() == StatusCode::OK {
        let hasil: AddressResp = response.json().await?;
        if let Some(data) = hasil.data {
            if let Some(addresses) = data.addresses {
                for address in addresses {
                    return Ok(address);
                }
            }
        }
        Ok(AddressInfo::default())
    } else {
        println!("Status: {}", response.status());
        println!("Harap Ganti akun");
        process::exit(1);
    }
}
pub async fn info_akun(client: Arc<reqwest::Client>, headers: Arc<HeaderMap>) -> Result<UserData, Box<dyn std::error::Error>> {
	let url2 = format!("https://shopee.co.id/api/v4/account/basic/get_account_info");
	println!("{}", url2);
    // Buat permintaan HTTP POST
    let response = client
        .get(&url2)
        .headers((*headers).clone())
        .version(Version::HTTP_2) 
        .send()
        .await?;

    if response.status() == StatusCode::OK {
        let hasil: InfoAkun = response.json().await?;
        if let Some(data) = hasil.data {
            Ok(data)
        } else {
            Ok(UserData {
                username: "LOGOUT (WARNING)".to_string(),
                email: "LOGOUT (WARNING)".to_string(),
                phone: "LOGOUT (WARNING)".to_string(),
                userid: 0,
            })
        }
    } else {
        println!("Status: {}", response.status());
        println!("Harap Ganti akun");
        process::exit(1);
    }
}
pub async fn get_redirect_url(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = ClientBuilder::new()
        .redirect(RedirectPolicy::limited(10))
        .danger_accept_invalid_certs(true)
        .impersonate(Impersonate::Chrome130)
        .enable_ech_grease(true)
        .permute_extensions(true)
        .gzip(true)
        .build()?;

    let res = client.get(url)
        .send()
        .await?;

    let final_url = res.url().clone();
    println!("Final URL: {}", final_url);
    Ok(final_url.to_string())
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
pub fn create_headers(cookie_content: &CookieData) -> HeaderMap {
    let mut headers = BASE_HEADER.clone();
    headers.insert("x-csrftoken", HeaderValue::from_str(&cookie_content.csrftoken).unwrap());
    headers.insert("cookie", HeaderValue::from_str(&cookie_content.cookie_content).unwrap());
    headers
}
pub fn create_cookie(cookie_content: &str) -> CookieData {
    let csrftoken = extract_csrftoken(&cookie_content);
    
    let datas = CookieData {
        cookie_content: cookie_content.to_string(),
        csrftoken: csrftoken.to_string(),
    };
    datas
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
pub async fn universal_client_skip_headers() -> reqwest::Client {
    ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .impersonate_skip_headers(Impersonate::Chrome130)
        .enable_ech_grease(true)
        .permute_extensions(true)
        .gzip(true)
        .build()
        .expect("Failed to create HTTP client")
}
pub fn url_to_voucher_data(url: &str) -> (String, String){
    let mut promotion_id = String::new();
    let mut signature = String::new();
    if let Some(query_str) = url.split('?').nth(1) {
        for param in query_str.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                match key {
                    "promotionId" | "promotionid" => promotion_id = value.to_string(),
                    "signature" => signature = value.to_string(),
                    _ => {}
                }
            }
        }
    }
    (promotion_id, signature)
}