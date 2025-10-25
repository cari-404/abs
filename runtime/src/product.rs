use rquest as reqwest;
use reqwest::Version;
use serde::{Serialize, Deserialize};
use crate::prepare::{self, CookieData, ProductInfo, FSInfo};
use anyhow::Result;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
struct UniversalResp<T> {
    data: Option<T>,
}
#[derive(Serialize, Deserialize, Debug)]
struct SessionData {
    sessions: Vec<FSInfo>,
}
#[derive(Serialize, Deserialize, Debug)]
struct ItemIdsData {
    item_brief_list: Vec<ItemId>,
}
#[derive(Serialize, Deserialize, Debug)]
struct ItemId {
    itemid: i64,
}

pub async fn get_current_fsid(client: Arc<reqwest::Client>, cookie_content: &CookieData) -> Result<Vec<FSInfo>, anyhow::Error>   {
    let url2 = format!("https://mall.shopee.co.id/api/v4/flash_sale/get_all_sessions");
    println!("{}", url2);
    println!("Get flash sale promotion_id...");
	
    let mut headers = prepare::FS_BASE_HEADER.clone();
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
    let hasil: UniversalResp<SessionData> = response.json().await?;

    let sessions = hasil.data.map(|d| d.sessions).unwrap_or_default();

    Ok(sessions)
}

pub async fn get_itemids_from_fsid(client: Arc<reqwest::Client>, fsid: &FSInfo, cookie_content: &CookieData) -> Result<Vec<ProductInfo>, anyhow::Error> {
    let url2 = format!("https://shopee.co.id/api/v4/flash_sale/get_all_itemids?need_personalize=true&promotionid={}&sort_soldout=true", fsid.promotionid);
    println!("{}", url2);
    println!("Get itemids from flash sale promotion_id...");
    
    let mut headers = prepare::FS_BASE_HEADER.clone();
    headers.insert("x-csrftoken", reqwest::header::HeaderValue::from_str(&cookie_content.csrftoken)?);
    headers.insert("cookie", reqwest::header::HeaderValue::from_str(&cookie_content.cookie_content)?);

    let response = client
        .get(&url2)
        .headers(headers)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    //println!("Status: {}", response.status());
    //println!("Headers: {:#?}", response.headers());
    let hasil: UniversalResp<ItemIdsData> = response.json().await?;

    let item_brief_list = hasil.data.map(|d| d.item_brief_list).unwrap_or_default();

    let product_info_list = item_brief_list.into_iter()
        .map(|item| ProductInfo {
            item_id: item.itemid,
            shop_id: 0, // default atau nanti dilengkapi
        })
        .collect();
    
    Ok(product_info_list)
}

/*pub async fn get_pdp(product_info: &ProductInfo, cookie_content: &CookieData) -> Result<(String, Vec<ModelInfo>, bool, FSInfo, String), anyhow::Error>  {
    let url2 = format!("https://mall.shopee.co.id/api/v4/pdp/get_lite?item_id={}&shopid={}", product_info.item_id, product_info.shop_id);
    println!("{}", url2);
    println!("sending Get Shopee request...");
	
    let mut headers = prepare::FS_BASE_HEADER.clone();
    headers.insert("referer", reqwest::header::HeaderValue::from_str(&format!("https://shopee.co.id/product/{}/{}", product_info.shop_id, product_info.item_id))?);
    headers.insert("x-csrftoken", reqwest::header::HeaderValue::from_str(&cookie_content.csrftoken)?);
    headers.insert("cookie", reqwest::header::HeaderValue::from_str(&cookie_content.cookie_content)?);

    // Buat klien HTTP
	let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .impersonate_skip_headers(Impersonate::Chrome130)
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

    //println!("Status: {}", response.status());
    //println!("Headers: {:#?}", response.headers());
    let status_code = response.status().to_string();
    let hasil: GetProduct = response.json().await?;
    //println!("Body: {}", &body);

    let (name, models_info, is_official_shop, fs_info) = if let Some(data) = hasil.data {
        let name = data.name.unwrap_or("Unknown".to_string());
        let models_info = data.models.unwrap_or_else(|| vec![ModelInfo {
            name: "Unknown".to_string(),
            price: 0,
            stock: 0,
            modelid: 0,
            promotionid: 0,
        }]);
        let is_official_shop = data.is_official_shop.unwrap_or(false);
        let fs_info = data.upcoming_flash_sale.unwrap_or(FSInfo {
            promotionid: 0,
            start_time: 0,
            end_time: 0,
        });
        (name, models_info, is_official_shop, fs_info)
    } else {
        println!("Status: {}", status_code);
        ("INVALID".to_string(), Vec::new(), false, FSInfo {
            promotionid: 0,
            start_time: 0,
            end_time: 0,
        })
    };
	Ok((name, models_info, is_official_shop, fs_info, status_code))
}*/