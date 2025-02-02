use rquest as reqwest;
use reqwest::tls::Impersonate;
use reqwest::{ClientBuilder, header::HeaderMap, Version};
use reqwest::header::HeaderValue;
use serde_json::{json, to_string, Value};
use serde::{Serialize, Deserialize};
use anyhow::Result;

use crate::prepare::{CookieData, ModelInfo, ShippingInfo, PaymentInfo, ProductInfo};
use crate::crypt::random_hex_string;

#[derive(Debug, Clone)]
pub struct Vouchers {
    pub promotionid: i64,
    pub voucher_code: String,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
struct RecomendPlatformResponse {
    data: Option<DataOnRecomendPlatform>,
}
#[derive(Debug, Deserialize)]
struct DataOnRecomendPlatform {
    freeshipping_vouchers: Option<Vec<VoucherOnRecomendPlatform>>,
    vouchers: Option<Vec<VoucherOnRecomendPlatform>>,
}
#[derive(Debug, Deserialize, Clone)]
struct VoucherOnRecomendPlatform {
    promotionid: i64,
    voucher_code: String,
    signature: String,
    fsv_error_message: Option<String>,
}

#[derive(Serialize)]
struct SaveVoucherRequest {
	voucher_promotionid: i64,
	signature: String,
	security_device_fingerprint: String,
	signature_source: String,
}

#[derive(Serialize)]
struct GetVoucherRequest {
    promotionid: i64,
    voucher_code: String,
    signature: String,
    need_basic_info: bool,
    need_user_voucher_status: bool,
}

#[derive(Serialize)]
struct JsonRequest {
    voucher_collection_request_list: Vec<VoucherCollectionRequest>,
}
#[derive(Serialize)]
struct VoucherCollectionRequest {
    collection_id: String,
    component_type: i64,
    component_id: i64,
    limit: i64,
    microsite_id: i64,
    offset: i64,
    number_of_vouchers_per_row: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Orders {
    shopid: i64,
    carrier_ids: Vec<i64>,
    shop_vouchers: Vec<ShopVoucher>,
    auto_apply: bool,
    iteminfos: Vec<ItemInfo>,
    selected_carrier_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct ShopVoucher;

#[derive(Serialize, Deserialize, Debug)]
struct ItemInfo {
    itemid: i64,
    modelid: i64,
    quantity: i32,
    item_group_id: Option<i64>,
    insurances: Vec<Insurance>,
    shopid: i64,
    shippable: bool,
    non_shippable_err: String,
    none_shippable_reason: String,
    none_shippable_full_reason: String,
    add_on_deal_id: i64,
    is_add_on_sub_item: bool,
    is_pre_order: bool,
    is_streaming_price: bool,
    checkout: bool,
    categories: Vec<Category>,
    is_spl_zero_interest: bool,
    is_prescription: bool,
    offerid: i64,
    supports_free_returns: bool,
    user_path: i64,
    models: Option<Models>,
    tier_variations: Option<TierVariations>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Category {
    catids: Vec<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Insurance;

#[derive(Serialize, Deserialize, Debug)]
struct Models;

#[derive(Serialize, Deserialize, Debug)]
struct TierVariations;

#[derive(Serialize)]
struct SelectedPaymentChannelDataOnRecommendPlatform {
    version: i64,
    option_info: String,
    channel_id: i64,
    channel_item_option_info: ChannelItemOptionInfoOnRecommendPlatform,
    text_info: TextInfo,
}

#[derive(Serialize)]
struct ChannelItemOptionInfoOnRecommendPlatform {
    option_info: String,
}

#[derive(Serialize)]
struct TextInfo {}

#[derive(Serialize)]
struct RecommendPlatform {
    orders: String,
    voucher_market_type: i64,
    check_voucher_payment_criteria: bool,
    selected_payment_channel_data: SelectedPaymentChannelDataOnRecommendPlatform,
    spm_channel_id: i64,
    need_wallet_active_info: bool,
    sorting_flag: i64,
    priority_promotion_ids: Vec<i64>,
    has_redeem_coins: bool,
    payment_manual_change: bool,
}

pub async fn save_shop_voucher_by_voucher_code(code: &str, cookie_content: &CookieData, product_info: &ProductInfo) -> Result<Option<Vouchers>>{
	let shop_id = product_info.shop_id;
    let headers = headers_checkout(&cookie_content).await;

    let body_json = json!({
        "voucher_code": code.to_string(),
        "shopid": shop_id
    });

    // Convert struct to JSON
    let body_str = serde_json::to_string(&body_json).unwrap();
    //println!("{:?}", body_str);
    //println!("{:?}", body);
    //println!("Request Headers:\n{:?}", headers);
    let mut vouchers: Option<Vouchers> = None;
	loop {
        let url2 = format!("https://mall.shopee.co.id/api/v2/voucher_wallet/save_shop_voucher_by_voucher_code");
        println!("{}", url2);
        // Buat klien HTTP
        let client = ClientBuilder::new()
            .http2_keep_alive_while_idle(true)
        .danger_accept_invalid_certs(true)
            .impersonate_without_headers(Impersonate::Chrome130)
            .enable_ech_grease(true)
            .permute_extensions(true)
            .gzip(true)
            //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
            .build()?;

        // Buat permintaan HTTP POST
        let response = client
            .post(&url2)
            .header("Content-Type", "application/json")
			.headers(headers.clone())
			.body(body_str.clone())
            .version(Version::HTTP_2) 
            .send()
            .await?;

        println!("Status: get_voucher");
        // Handle response as needed
        //println!("Request Headers:\n{:?}", headers);
		let status = response.status();
		println!("{}", status);
		let parsed: Value = response.json().await?;
        //println!("Body: {}", body);
        // Parse response body as JSON
        if status == reqwest::StatusCode::OK {
            if let Some(error) = parsed.get("error").and_then(|e| e.as_i64()) {
                if error == 5 || error == 0 {
                    println!("Berhasil: {} - {}", error, parsed.get("error_msg").unwrap_or(&serde_json::Value::Null));
                } else {
                    println!("Error: {} - {}", error, parsed.get("error_msg").unwrap_or(&serde_json::Value::Null));
                    continue;
                }
            }
            if let Some(data) = parsed.get("data") {
                if let Some(voucher) = data.get("voucher") {
                    let promotionid = voucher.get("promotionid").and_then(|v| v.as_i64()).unwrap_or_default();
                    let voucher_code = code.to_string();
                    let signature = voucher.get("signature").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    println!("promotionid: {}, voucher_code: {}, signature: {}", promotionid, voucher_code, signature);
                    vouchers = Some(Vouchers {
                        promotionid,
                        voucher_code,
                        signature,
                    });
                }
            }
            break;
        } else if status == reqwest::StatusCode::IM_A_TEAPOT {
            println!("Gagal, status code: 418 - I'm a teapot. Mencoba kembali...");
            println!("{}", parsed);
            continue;
        }else {
            println!("Status: {}", status);
            break;
        }
    }
    Ok(vouchers)
}

pub async fn save_platform_voucher_by_voucher_code(code: &str, cookie_content: &CookieData) -> Result<Option<Vouchers>>{
    let headers = headers_checkout(&cookie_content).await;

    let body_json = json!({
        "voucher_code": code.to_string(),
        "need_user_voucher_status":true
    });

    // Convert struct to JSON
    let body_str = serde_json::to_string(&body_json).unwrap();
    //println!("{:?}", body_str);
    //println!("{:?}", body);
    //println!("Request Headers:\n{:?}", headers);
    let mut vouchers: Option<Vouchers> = None;
	loop {
        let url2 = format!("https://mall.shopee.co.id/api/v2/voucher_wallet/save_voucher");
        println!("{}", url2);
        // Buat klien HTTP
        let client = ClientBuilder::new()
            .http2_keep_alive_while_idle(true)
        .danger_accept_invalid_certs(true)
            .impersonate_without_headers(Impersonate::Chrome130)
            .enable_ech_grease(true)
            .permute_extensions(true)
            .gzip(true)
            //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
            .build()?;

        // Buat permintaan HTTP POST
        let response = client
            .post(&url2)
            .header("Content-Type", "application/json")
			.headers(headers.clone())
			.body(body_str.clone())
            .version(Version::HTTP_2) 
            .send()
            .await?;

        println!("Status: get_voucher");
        // Handle response as needed
        //println!("Request Headers:\n{:?}", headers);
		let status = response.status();
		println!("{}", status);
		let parsed: Value = response.json().await?;
        //println!("Body: {}", body);
        // Parse response body as JSON
        if status == reqwest::StatusCode::OK {
            if let Some(error) = parsed.get("error").and_then(|e| e.as_i64()) {
                if error == 5 || error == 0 {
                    println!("Berhasil: {} - {}", error, parsed.get("error_msg").unwrap_or(&serde_json::Value::Null));
                } else {
                    println!("Error: {} - {}", error, parsed.get("error_msg").unwrap_or(&serde_json::Value::Null));
                    continue;
                }
            }
            if let Some(data) = parsed.get("data") {
                if let Some(voucher) = data.get("voucher") {
                    let promotionid = voucher.get("promotionid").and_then(|v| v.as_i64()).unwrap_or_default();
                    let voucher_code = voucher.get("voucher_code").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    let signature = voucher.get("signature").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    println!("promotionid: {}, voucher_code: {}, signature: {}", promotionid, voucher_code, signature);
                    vouchers = Some(Vouchers {
                        promotionid,
                        voucher_code,
                        signature,
                    });
                }
            }
            break;
        } else if status == reqwest::StatusCode::IM_A_TEAPOT {
            println!("Gagal, status code: 418 - I'm a teapot. Mencoba kembali...");
            println!("{}", parsed);
            continue;
        }else {
            println!("Status: {}", status);
            break;
        }
    }
    Ok(vouchers)
}

pub async fn save_voucher(start: &str, end: &str, cookie_content: &CookieData) -> Result<Option<Vouchers>>{
    let headers = headers_checkout(&cookie_content).await;
	let start: i64 = start.trim().parse().expect("Input tidak valid");

	let body_json = SaveVoucherRequest {
	  voucher_promotionid: start as i64,
	  signature: end.to_string(),
	  security_device_fingerprint: String::new(),
	  signature_source: 0.to_string(),
	};
	
	let body_str = serde_json::to_string(&body_json)?;

	println!("{}", body_str);

	//println!("");
	//println!("header:{:#?}", headers);
    let mut vouchers: Option<Vouchers> = None;
	loop {
        let client = ClientBuilder::new()
            .http2_keep_alive_while_idle(true)
            .danger_accept_invalid_certs(true)
            .impersonate_without_headers(Impersonate::Chrome130)
            .enable_ech_grease(true)
            .permute_extensions(true)
            .gzip(true)
            //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
            .build()?;
    
        // Buat permintaan HTTP POST
        let response = client
            .post("https://mall.shopee.co.id/api/v2/voucher_wallet/save_voucher")
			.headers(headers.clone())
			.json(&body_json)
            .version(Version::HTTP_2) 
            .send()
            .await?;
		// Check for HTTP status code indicating an error
		//let http_version = response.version(); 		// disable output features
		//println!("HTTP Version: {:?}", http_version); // disable output features
		let status = response.status();
		println!("{}", status);
		let parsed: Value = response.json().await?;
		if status == reqwest::StatusCode::OK {
            if let Some(error) = parsed.get("error").and_then(|e| e.as_i64()) {
                if error == 5 || error == 0 {
                    println!("Berhasil: {} - {}", error, parsed.get("error_msg").unwrap_or(&serde_json::Value::Null));
                } else {
                    println!("Error: {} - {}", error, parsed.get("error_msg").unwrap_or(&serde_json::Value::Null));
                    continue;
                }
            }
            if let Some(data) = parsed.get("data") {
                if let Some(voucher) = data.get("voucher") {
                    let promotionid = voucher.get("promotionid").and_then(|v| v.as_i64()).unwrap_or_default();
                    let voucher_code = voucher.get("voucher_code").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    let signature = voucher.get("signature").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    println!("promotionid: {}, voucher_code: {}, signature: {}", promotionid, voucher_code, signature);
                    vouchers = Some(Vouchers {
                        promotionid,
                        voucher_code,
                        signature,
                    });
                }
            }
            break;
		} else if status == reqwest::StatusCode::IM_A_TEAPOT {
			println!("Gagal, status code: 418 - I'm a teapot. Mencoba kembali...");
			println!("{}", parsed);
			continue;
		}else {
			println!("Status: {}", status);
			break;
		}
	}
	Ok(vouchers)
}

pub async fn get_voucher_data(start: &str, end: &str, cookie_content: &CookieData) -> Result<Option<Vouchers>>{
    let headers = headers_checkout(&cookie_content).await;
	let start: i64 = start.trim().parse().expect("Input tidak valid");

	let body_json = GetVoucherRequest{
        promotionid: start as i64,
        voucher_code: "-".to_string(),
        signature: end.to_string(),
        need_basic_info: true,
        need_user_voucher_status: true,
    };
	
	let body_str = serde_json::to_string(&body_json)?;

	println!("{}", body_str);

	//println!("");
	//println!("header:{:#?}", headers);
    let mut vouchers: Option<Vouchers> = None;
	loop {
        let client = ClientBuilder::new()
            .http2_keep_alive_while_idle(true)
            .danger_accept_invalid_certs(true)
            .impersonate_without_headers(Impersonate::Chrome130)
            .enable_ech_grease(true)
            .permute_extensions(true)
            .gzip(true)
            //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
            .build()?;
    
        // Buat permintaan HTTP POST
        let response = client
            .post("https://mall.shopee.co.id/api/v2/voucher_wallet/get_voucher_detail")
			.headers(headers.clone())
			.json(&body_json)
            .version(Version::HTTP_2) 
            .send()
            .await?;
		// Check for HTTP status code indicating an error
		//let http_version = response.version(); 		// disable output features
		//println!("HTTP Version: {:?}", http_version); // disable output features
		let status = response.status();
		println!("{}", status);
		let parsed: Value = response.json().await?;
		if status == reqwest::StatusCode::OK {
            if let Some(error) = parsed.get("error").and_then(|e| e.as_i64()) {
                if error == 5 || error == 0 {
                    println!("Berhasil: {} - {}", error, parsed.get("error_msg").unwrap_or(&serde_json::Value::Null));
                } else {
                    println!("Error: {} - {}", error, parsed.get("error_msg").unwrap_or(&serde_json::Value::Null));
                    continue;
                }
            }
            if let Some(data) = parsed.get("data") {
                if let Some(voucher) = data.get("voucher_basic_info") {
                    let promotionid = voucher.get("promotionid").and_then(|v| v.as_i64()).unwrap_or_default();
                    let voucher_code = voucher.get("voucher_code").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    let signature = voucher.get("signature").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    println!("promotionid: {}, voucher_code: {}, signature: {}", promotionid, voucher_code, signature);
                    vouchers = Some(Vouchers {
                        promotionid,
                        voucher_code,
                        signature,
                    });
                }
            }
            break;
		} else if status == reqwest::StatusCode::IM_A_TEAPOT {
			println!("Gagal, status code: 418 - I'm a teapot. Mencoba kembali...");
			println!("{}", parsed);
			continue;
		}else {
			println!("Status: {}", status);
			break;
		}
	}
	Ok(vouchers)
}

pub async fn get_recommend_platform_vouchers(cookie_content: &CookieData, product_info: &ProductInfo, quantity: i32, chosen_model: &ModelInfo, chosen_payment: &PaymentInfo, chosen_shipping: &ShippingInfo) -> Result<(Option<Vouchers>, Option<Vouchers>)>{
    let headers = headers_checkout(&cookie_content).await;
	let optioninfo: String = chosen_payment.option_info.clone();
    let orders_json = vec![Orders {
        shopid: product_info.shop_id,
        carrier_ids: vec![8005, 8003, 80099, 80055, 8006, 80021],
        shop_vouchers: vec![],
        auto_apply: true,
        iteminfos: vec![ItemInfo {
            itemid: product_info.item_id,
            modelid: chosen_model.modelid,
            quantity,
            item_group_id: None,
            insurances: vec![],
            shopid: product_info.shop_id,
            shippable: true,
            non_shippable_err: String::new(),
            none_shippable_reason: String::new(),
            none_shippable_full_reason: String::new(),
            add_on_deal_id: 0,
            is_add_on_sub_item: false,
            is_pre_order: false,
            is_streaming_price: false,
            checkout: true,
            categories: vec![Category {
                catids: vec![100013, 100073],
            }],
            is_spl_zero_interest: false,
            is_prescription: false,
            offerid: 0,
            supports_free_returns: false,
            user_path: 1,
            models: None,
            tier_variations: None,
        }],
        selected_carrier_id: chosen_shipping.channelid,
    }];
    // Konversi orders_json menjadi string
    let orders_string = to_string(&orders_json)?;
    let body_json = RecommendPlatform {
        orders: orders_string,
        voucher_market_type: 1,
        check_voucher_payment_criteria: true,
        selected_payment_channel_data: SelectedPaymentChannelDataOnRecommendPlatform {
            version: chosen_payment.version,
            option_info: String::new(),
            channel_id: chosen_payment.channel_id,
            channel_item_option_info: ChannelItemOptionInfoOnRecommendPlatform {
                option_info: optioninfo,
            },
            text_info: TextInfo {},
        },
        spm_channel_id: chosen_payment.channel_id,
        need_wallet_active_info: true,
        sorting_flag: 8,
        priority_promotion_ids: vec![],
        has_redeem_coins: false,
        payment_manual_change: true,
    };

    // Convert struct to JSON
    //let body_str = serde_json::to_string(&body_json)?;
    //println!("{:?}", body_str);
    //println!("{:?}", body);
    //println!("Request Headers:\n{:?}", headers);

    let url2 = format!("https://mall.shopee.co.id/api/v2/voucher_wallet/get_recommend_platform_vouchers");
    println!("{}", url2);
    // Buat klien HTTP
    let client = ClientBuilder::new()
        .http2_keep_alive_while_idle(true)
        .danger_accept_invalid_certs(true)
        .impersonate_without_headers(Impersonate::Chrome130)
        .enable_ech_grease(true)
        .permute_extensions(true)
        .gzip(true)
        .http2_max_concurrent_streams(1000)
        //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
        .build()?;

    // Buat permintaan HTTP POST
    let response = client
        .post(&url2)
        .headers(headers)
        .json(&body_json)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    println!("Status: get_voucher");
    // Handle response as needed
    //println!("Request Headers:\n{:?}", headers);
    println!("Status: {}", response.status());
    let json_resp: RecomendPlatformResponse = response.json().await?;
    //println!("Body: {}", body_resp);
    // Parse response body as JSON
    let mut freeshipping_voucher: Option<Vouchers> = None;
    let mut vouchers: Option<Vouchers> = None;
    // Extract freeshipping_vouchers
    if let Some(freeshipping_vouchers_array) = json_resp.data.as_ref().and_then(|data| data.freeshipping_vouchers.as_ref()) {
        for voucher in freeshipping_vouchers_array {
            if voucher.fsv_error_message.is_none() {
                freeshipping_voucher = Some(Vouchers {
                    promotionid : voucher.promotionid.clone(),
                    voucher_code : voucher.voucher_code.clone(),
                    signature : voucher.signature.clone(),
                });
                break; // Found one valid voucher, so break
            }
        }
    }

    // Extract vouchers
    if let Some(vouchers_array) = json_resp.data.as_ref().and_then(|data| data.vouchers.as_ref()) {
        for voucher in vouchers_array {
            if voucher.fsv_error_message.is_none() {
                vouchers = Some(Vouchers {
                    promotionid : voucher.promotionid.clone(),
                    voucher_code : voucher.voucher_code.clone(),
                    signature : voucher.signature.clone(),
                });
                break; // Found one valid voucher, so break
            }
        }
    }
    Ok((freeshipping_voucher, vouchers))
}
async fn headers_checkout(cookie_content: &CookieData) -> HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
	headers.insert("x-api-source", HeaderValue::from_static("rn"));
	headers.insert("x-shopee-client-timezone", HeaderValue::from_static("Asia/Jakarta"));
	headers.insert("x-sap-access-f", HeaderValue::from_static(""));
	headers.insert("x-sap-access-t", HeaderValue::from_static(""));
	headers.insert("af-ac-enc-dat", HeaderValue::from_str(&format!("{}", random_hex_string(16))).unwrap());
	headers.insert("af-ac-enc-id", HeaderValue::from_static(""));
	headers.insert("af-ac-enc-sz-token", HeaderValue::from_static(""));
	headers.insert("if-none-match-", HeaderValue::from_static("55b03-97d86fe6888b54a9c5bfa268cf3d922d"));
	headers.insert("shopee_http_dns_mode", HeaderValue::from_static("1"));
	headers.insert("x-sap-access-s", HeaderValue::from_static(""));
	headers.insert("x-csrftoken", HeaderValue::from_str(&cookie_content.csrftoken).unwrap());
	headers.insert("user-agent", HeaderValue::from_static("Android app Shopee appver=29339 app_type=1"));
	headers.insert("referer", HeaderValue::from_static("https://mall.shopee.co.id"));
	headers.insert("accept", HeaderValue::from_static("application/json"));
	headers.insert("content-type", HeaderValue::from_static("application/json; charset=utf-8"));
	headers.insert("cookie", HeaderValue::from_str(&cookie_content.cookie_content).unwrap());
    // Return the created headers
    headers
}

pub async fn some_function(start: &str, cookie_content: &CookieData) -> Result<(String, String)> {
	let voucher_request = VoucherCollectionRequest {
		collection_id: start.to_string(),
		component_type: 2,
		component_id: 1712077200,
		limit: 100,
		microsite_id: 63749,
		offset: 0,
		number_of_vouchers_per_row: 2,
	};
	
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("Android app Shopee appver=29335 app_type=1"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("Accept", HeaderValue::from_static("application/json"));
    headers.insert("Accept-Encoding", HeaderValue::from_static("gzip"));
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("x-api-source", HeaderValue::from_static("rn"));
    headers.insert("if-none-match-", HeaderValue::from_static("55b03-1e991df3597baecb4f87bfbe85b99329"));
    headers.insert("af-ac-enc-dat", HeaderValue::from_static(""));
    headers.insert("af-ac-enc-sz-token", HeaderValue::from_static(""));
    headers.insert("shopee_http_dns_mode", HeaderValue::from_static("1"));
    headers.insert("af-ac-enc-id", HeaderValue::from_static(""));
    headers.insert("x-sap-access-t", HeaderValue::from_static(""));
    headers.insert("x-sap-access-s", HeaderValue::from_static(""));
    headers.insert("x-sap-access-f", HeaderValue::from_static(""));
    headers.insert("x-shopee-client-timezone", HeaderValue::from_static("Asia/Jakarta"));
    headers.insert("referer", HeaderValue::from_static("https://mall.shopee.co.id/"));
    headers.insert("x-csrftoken", reqwest::header::HeaderValue::from_str(&cookie_content.csrftoken)?);
    headers.insert(reqwest::header::COOKIE, reqwest::header::HeaderValue::from_str(&cookie_content.cookie_content)?);

	// Bentuk struct JsonRequest
	let json_request = JsonRequest {
		voucher_collection_request_list: vec![voucher_request],
	};

	// Convert struct to JSON
	//let json_body = serde_json::to_string(&json_request)?;
	//println!("{}", json_body);
	
	loop {
        let client = ClientBuilder::new()
            .http2_keep_alive_while_idle(true)
        .danger_accept_invalid_certs(true)
            .impersonate_without_headers(Impersonate::Chrome130)
            .enable_ech_grease(true)
            .permute_extensions(true)
            .gzip(true)
            //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
            .build()?;

        // Buat permintaan HTTP POST
        let response = client
            .post("https://mall.shopee.co.id/api/v1/microsite/get_vouchers_by_collections")
            .headers(headers.clone())
            .json(&json_request)
            .version(Version::HTTP_2) 
            .send()
            .await?;

		// Check for HTTP status code indicating an error
		//let http_version = response.version(); 		// disable output features
		//println!("HTTP Version: {:?}", http_version); // disable output features
		let status = response.status();
		let hasil: Value = response.json().await?;
		//println!("{}", text);
		if status == reqwest::StatusCode::OK {
			/*let error_res = hasil.get("error").and_then(|er| er.as_i64()).unwrap_or(0);
			let error_res_str = error_res.to_string();*/
			// Access specific values using serde_json::Value methods
			if let Some(data_array) = hasil.get("data").and_then(|data| data.as_array()) {
				for data_value in data_array {
					if let Some(vouchers_array) = data_value.get("vouchers").and_then(|vouchers| vouchers.as_array()) {
						for voucher_value in vouchers_array {
							if let Some(voucher_obj) = voucher_value.get("voucher").and_then(|voucher| voucher.as_object()) {
								if let Some(voucher_identifier_obj) = voucher_obj.get("voucher_identifier").and_then(|vi| vi.as_object()) {
									let promotion_id_temp = voucher_identifier_obj.get("promotion_id").and_then(|pi| pi.as_i64()).unwrap_or(0);
									let signature_temp = voucher_identifier_obj.get("signature").and_then(|s| s.as_str()).unwrap_or("");
									let promotion_id = promotion_id_temp.to_string();
                                    let signature = signature_temp.to_string();
									/*println!("{}", promotion_id);
									println!("{}", signature);*/
                                    return Ok((promotion_id, signature));
								}
							}
						}
					}else{
						println!("API Checker 1");
						let cid_1 = start.to_string();
						let (promotion_id, signature) = api_1(&cid_1, &headers.clone()).await?;
						return Ok((promotion_id.to_string(), signature.to_string()));
					}
				}
			/*} else if !error_res_str.is_empty() {
				interactive_print(&pb, &println!("error: {}", error_res_str));*/
			}else {
				println!("Tidak ada data ditemukan untuk collection_id: {}", start.to_string());
			}
			break;
		}else if status == reqwest::StatusCode::IM_A_TEAPOT {
			println!("POST request gagal untuk collection_id:: {}", start.to_string());
			println!("Gagal, status code: 418 - I'm a teapot. Mencoba kembali...");
			println!("{}", hasil);
			continue;
		}else {
			println!("POST request gagal untuk collection_id:: {}", start.to_string());
			println!("Status: {}", status);
			break;
		}
	}
	Ok((String::new(), String::new()))	
}

async fn api_1(cid_1: &str, headers: &HeaderMap) -> Result<(String, String)> {
	let cloned_headers = headers.clone();
	let voucher_request = VoucherCollectionRequest {
		collection_id: cid_1.to_string(),
		component_type: 1,
		component_id: 1708068524282,
		limit: 100,
		microsite_id: 62902,
		offset: 0,
		number_of_vouchers_per_row: 1,
	};
	// Bentuk struct JsonRequest
	let json_request = JsonRequest {
		voucher_collection_request_list: vec![voucher_request],
	};

	// Convert struct to JSON
	//let json_body = serde_json::to_string(&json_request)?;
	
	loop {
        let client = ClientBuilder::new()
            .http2_keep_alive_while_idle(true)
        .danger_accept_invalid_certs(true)
            .impersonate_without_headers(Impersonate::Chrome130)
            .enable_ech_grease(true)
            .permute_extensions(true)
            .gzip(true)
            //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
            .build()?;

        // Buat permintaan HTTP POST
        let response = client
            .post("https://mall.shopee.co.id/api/v1/microsite/get_vouchers_by_collections")
            .headers(cloned_headers.clone())
            .json(&json_request)
            .version(Version::HTTP_2) 
            .send()
            .await?;
		// Check for HTTP status code indicating an error
		//let http_version = response.version(); 		// disable output features
		//println!("HTTP Version: {:?}", http_version); // disable output features
		let status = response.status();
		let hasil: Value = response.json().await?;
		if status == reqwest::StatusCode::OK {
			/*let error_res = hasil.get("error").and_then(|er| er.as_i64()).unwrap_or(0);
			let error_res_str = error_res.to_string();*/
			// Access specific values using serde_json::Value methods
			if let Some(data_array) = hasil.get("data").and_then(|data| data.as_array()) {
				for data_value in data_array {
					if let Some(vouchers_array) = data_value.get("vouchers").and_then(|vouchers| vouchers.as_array()) {
						for voucher_value in vouchers_array {
							if let Some(voucher_obj) = voucher_value.get("voucher").and_then(|voucher| voucher.as_object()) {
								if let Some(voucher_identifier_obj) = voucher_obj.get("voucher_identifier").and_then(|vi| vi.as_object()) {
									let promotion_id_temp = voucher_identifier_obj.get("promotion_id").and_then(|pi| pi.as_i64()).unwrap_or(0);
									let signature_temp = voucher_identifier_obj.get("signature").and_then(|s| s.as_str()).unwrap_or("");
									let promotion_id = promotion_id_temp.to_string();
                                    let signature = signature_temp.to_string();
									/*println!("{}", promotion_id);
									println!("{}", signature);*/
                                    return Ok((promotion_id, signature));
								}
							}
						}
					}else{
						println!("Bug API 2");
						println!("Tidak ada Info vouchers ditemukan untuk collection_id:{}", cid_1);
					}
				}
			/*} else if !error_res_str.is_empty() {
				interactive_print(&pb, &println!("error: {}", error_res_str));*/
			}else {
				println!("Tidak ada data ditemukan untuk collection_id: {}", cid_1);
			}
			break;
		}else if status == reqwest::StatusCode::IM_A_TEAPOT {
			println!("POST request gagal untuk collection_id:: {}", cid_1);
			println!("Gagal, status code: 418 - I'm a teapot. Mencoba kembali...");
			println!("{}", hasil);
			continue;
		}else {
			println!("POST request gagal untuk collection_id:: {}", cid_1);
			println!("Status: {}", status);
			break;
		}
	}
	Ok((String::new(), String::new()))	
}