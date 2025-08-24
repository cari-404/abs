use rquest as reqwest;
use reqwest::{header::HeaderMap, Version};
use reqwest::header::HeaderValue;
use serde_json::{json, to_string, Value};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::sync::Arc;
use once_cell::sync::Lazy;

use crate::prepare::{CookieData, ModelInfo, ShippingInfo, PaymentInfo, ProductInfo, AddressInfo};
use crate::crypt::random_hex_string;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vouchers {
    pub promotionid: i64,
    pub voucher_code: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shop_id: Option<i64>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VoucherDetail {
    pub min_spend: i64,
    pub start_time: i32,
    pub end_time: i32,
    pub reward_percentage: i32,
    pub reward_value: i64,
    pub reward_cap: i64,
}

#[derive(Debug, Deserialize)]
pub struct RecomendPlatformResponse {
    pub data: Option<DataOnRecomendPlatform>,
}
#[derive(Debug, Deserialize)]
pub struct DataOnRecomendPlatform {
    pub freeshipping_vouchers: Option<Vec<VoucherOnRecomendPlatform>>,
    pub vouchers: Option<Vec<VoucherOnRecomendPlatform>>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct VoucherOnRecomendPlatform {
    pub promotionid: i64,
    pub voucher_code: String,
    pub signature: String,
    pub fsv_error_message: Option<String>,
    pub fsv_voucher_card_ui_info: Option<FsvVoucherCardUiInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FsvVoucherCardUiInfo {
    pub int_min_spend_fsv_ui_only: i64,
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
pub struct JsonCollectionRequest {
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
pub struct Orders {
    pub shopid: i64,
    pub carrier_ids: Vec<i64>,
    pub shop_vouchers: Vec<ShopVoucher>,
    pub auto_apply: bool,
    pub iteminfos: Vec<ItemInfo>,
    pub carrier_infos: Vec<CarrierInfo>,
    pub selected_carrier_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShopVoucher;

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemInfo {
    pub itemid: i64,
    pub modelid: i64,
    pub quantity: i32,
    pub item_group_id: Option<i64>,
    pub insurances: Vec<Insurance>,
    pub shopid: i64,
    pub shippable: bool,
    pub non_shippable_err: String,
    pub none_shippable_reason: String,
    pub none_shippable_full_reason: String,
    pub add_on_deal_id: i64,
    pub is_add_on_sub_item: bool,
    pub is_pre_order: bool,
    pub is_streaming_price: bool,
    pub checkout: bool,
    pub is_spl_zero_interest: bool,
    pub is_prescription: bool,
    pub offerid: i64,
    pub supports_free_returns: bool,
    pub user_path: i64,
    pub models: Option<Models>,
    pub tier_variations: Option<TierVariations>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CarrierInfo {
    pub carrier_id: i64,
    pub esf: i64,
    pub shippable_item_ids: Vec<i64>,
    pub buyer_address: AddressInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Insurance;

#[derive(Serialize, Deserialize, Debug)]
pub struct Models;

#[derive(Serialize, Deserialize, Debug)]
pub struct TierVariations;

#[derive(Serialize)]
pub struct SelectedPaymentChannelDataOnRecommendPlatform {
    pub version: i64,
    pub option_info: String,
    pub channel_id: i64,
    pub channel_item_option_info: ChannelItemOptionInfoOnRecommendPlatform,
    pub text_info: TextInfo,
}

#[derive(Serialize)]
pub struct ChannelItemOptionInfoOnRecommendPlatform {
   pub option_info: String,
}

#[derive(Serialize)]
pub struct TextInfo {}

#[derive(Serialize)]
pub struct RecommendPlatform {
    pub orders: String,
    pub voucher_market_type: i64,
    pub check_voucher_payment_criteria: bool,
    pub selected_payment_channel_data: SelectedPaymentChannelDataOnRecommendPlatform,
    pub spm_channel_id: i64,
    pub need_wallet_active_info: bool,
    pub sorting_flag: i64,
    pub priority_promotion_ids: Vec<i64>,
    pub has_redeem_coins: bool,
    pub payment_manual_change: bool,
}

pub struct VoucherInfo {
    pub promotion_id: i64,
    pub voucher_code: String,
    pub signature: String,
    pub extra_data: String,
    pub bug: bool,
    pub status: rquest::StatusCode,
}

#[derive(Serialize)]
struct ReqData {
    promotion_id: i64,
    voucher_code: String,
}
#[derive(Serialize)]
pub struct FoodVoucherRequest {
    pub cmd: String,
    pub req_data: String,
}

pub static VC_HEADER_APP: Lazy<HeaderMap> = Lazy::new(|| {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
	headers.insert("x-api-source", HeaderValue::from_static("rn"));
	headers.insert("x-shopee-client-timezone", HeaderValue::from_static("Asia/Jakarta"));
	headers.insert("x-sap-access-f", HeaderValue::from_static(""));
	headers.insert("x-sap-access-t", HeaderValue::from_static(""));
	headers.insert("af-ac-enc-id", HeaderValue::from_static(""));
	headers.insert("af-ac-enc-sz-token", HeaderValue::from_static(""));
	headers.insert("if-none-match-", HeaderValue::from_static("55b03-97d86fe6888b54a9c5bfa268cf3d922d"));
	headers.insert("shopee_http_dns_mode", HeaderValue::from_static("1"));
	headers.insert("x-sap-access-s", HeaderValue::from_static(""));
    headers.insert("user-agent", HeaderValue::from_static("Android app Shopee appver=29347 app_type=1"));
	headers.insert("referer", HeaderValue::from_static("https://mall.shopee.co.id"));
	headers.insert("accept", HeaderValue::from_static("application/json"));
	headers.insert("content-type", HeaderValue::from_static("application/json; charset=utf-8"));
    headers
});

pub async fn save_shop_voucher_by_voucher_code(client: Arc<reqwest::Client>, code: &str, headers: Arc<HeaderMap>, product_info: ProductInfo) -> anyhow::Result<Option<Vouchers>>{
    let body_json = json!({
        "voucher_code": code.to_string(),
        "shopid": product_info.shop_id,
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

        // Buat permintaan HTTP POST
        let response = (*client).clone()
            .post(&url2)
            .header("Content-Type", "application/json")
			.headers((*headers).clone())
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
                        shop_id: Some(product_info.shop_id),
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

pub async fn save_platform_voucher_by_voucher_code(client: Arc<reqwest::Client>, code: &str, headers: Arc<HeaderMap>) -> anyhow::Result<Option<Vouchers>>{
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
        let response = (*client).clone()
            .post(&url2)
            .header("Content-Type", "application/json")
			.headers((*headers).clone())
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
                        shop_id: None,
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

pub async fn save_voucher(client: Arc<reqwest::Client>, start: &str, end: &str, headers: Arc<HeaderMap>) -> anyhow::Result<Option<Vouchers>>{
    let body_json = SaveVoucherRequest {
        voucher_promotionid: start.trim().parse().expect("Input tidak valid"),
        signature: end.to_string(),
        security_device_fingerprint: String::new(),
        signature_source: "0".to_string(),
    };
	
	//let body_str = serde_json::to_string(&body_json)?;
	//println!("{}", body_str);

	//println!("");
	//println!("header:{:#?}", headers);
    let mut vouchers: Option<Vouchers> = None;
	loop {
        // Buat permintaan HTTP POST
        let response = (*client).clone()
            .post("https://mall.shopee.co.id/api/v2/voucher_wallet/save_voucher")
			.headers((*headers).clone())
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
                        shop_id: None,
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

pub async fn get_voucher_data(client: Arc<reqwest::Client>, start: &str, end: &str, headers: Arc<HeaderMap>) -> anyhow::Result<(Option<Vouchers>, Option<VoucherDetail>)>{
	let body_json = GetVoucherRequest{
        promotionid: start.trim().parse().expect("Input tidak valid"),
        voucher_code: "-".to_string(),
        signature: end.to_string(),
        need_basic_info: true,
        need_user_voucher_status: true,
    };
	
	//let body_str = serde_json::to_string(&body_json)?;
	//println!("{}", body_str);

	//println!("");
	//println!("header:{:#?}", headers);
    let mut vouchers: Option<Vouchers> = None;
    let mut detail_vouchers: Option<VoucherDetail> = None;
	loop {
        let response = (*client).clone()
            .post("https://mall.shopee.co.id/api/v2/voucher_wallet/get_voucher_detail")
			.headers((*headers).clone())
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
                    let min_spend = voucher.get("min_spend").and_then(|v| v.as_i64()).unwrap_or_default();
                    let start_time = voucher.get("start_time").and_then(|v| v.as_i64()).unwrap_or_default() as i32;
                    let end_time = voucher.get("end_time").and_then(|v| v.as_i64()).unwrap_or_default() as i32;
                    let reward_percentage = voucher.get("reward_percentage").and_then(|v| v.as_i64()).unwrap_or_default() as i32;
                    let reward_value = voucher.get("reward_value").and_then(|v| v.as_i64()).unwrap_or_default();
                    let reward_cap = voucher.get("reward_cap").and_then(|v| v.as_i64()).unwrap_or_default();
                    println!("promotionid: {}, voucher_code: {}, signature: {}", promotionid, voucher_code, signature);
                    vouchers = Some(Vouchers {
                        promotionid,
                        voucher_code,
                        signature,
                        shop_id: None,
                    });
                    detail_vouchers = Some(VoucherDetail {
                        min_spend: min_spend,
                        start_time: start_time,
                        end_time: end_time,
                        reward_percentage: reward_percentage,
                        reward_value: reward_value,
                        reward_cap: reward_cap,
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
	Ok((vouchers, detail_vouchers))
}

pub async fn get_recommend_platform_vouchers(adjusted_max_price: Option<i64>, buyer_address: &AddressInfo, client: Arc<reqwest::Client>, headers: Arc<HeaderMap>, product_info: &ProductInfo, quantity: i32, chosen_model: &ModelInfo, chosen_payment: &PaymentInfo, chosen_shipping: &ShippingInfo) -> Result<(Option<Vouchers>, Option<Vouchers>)>{
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
            is_spl_zero_interest: false,
            is_prescription: false,
            offerid: 0,
            supports_free_returns: false,
            user_path: 1,
            models: None,
            tier_variations: None,
        }],
        carrier_infos: vec![CarrierInfo {
            carrier_id: chosen_shipping.channelidroot,
            esf: if chosen_shipping.original_cost == 0 { 1 } else { chosen_shipping.original_cost },
            shippable_item_ids: vec![product_info.item_id],
            buyer_address: buyer_address.clone(),
        }],
        selected_carrier_id: chosen_shipping.channelidroot,
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
                option_info: chosen_payment.option_info.to_string(),
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

    // Buat permintaan HTTP POST
    let response = (*client)
        .post(&url2)
        .headers((*headers).clone())
        .json(&body_json)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    println!("Status: get_voucher");
    // Handle response as needed
    //println!("Request Headers:\n{:?}", headers);
    let status = response.status();
    let json_resp: RecomendPlatformResponse = response.json().await?;
    //println!("Body: {}", body_resp);
    // Parse response body as JSON
    let mut freeshipping_voucher: Option<Vouchers> = None;
    let mut vouchers: Option<Vouchers> = None;
    // Extract freeshipping_vouchers
    if status == reqwest::StatusCode::OK {
        if let Some(freeshipping_vouchers_array) = json_resp.data.as_ref().and_then(|data| data.freeshipping_vouchers.as_ref()) {
            if let Some(voucher) = freeshipping_vouchers_array.iter().find(|v| { v.fsv_error_message.is_none() && v.fsv_voucher_card_ui_info.as_ref().map_or(true, |info| adjusted_max_price.map_or(true, |max| info.int_min_spend_fsv_ui_only <= max))}) {
                freeshipping_voucher = Some(Vouchers {
                    promotionid : voucher.promotionid,
                    voucher_code : voucher.voucher_code.clone(),
                    signature : voucher.signature.clone(),
                    shop_id: None,
                });
            }
        }

        // Extract vouchers
        if let Some(vouchers_array) = json_resp.data.as_ref().and_then(|data| data.vouchers.as_ref()) {
            if let Some(voucher) = vouchers_array.iter().find(|v| v.fsv_error_message.is_none()) {
                vouchers = Some(Vouchers {
                    promotionid : voucher.promotionid,
                    voucher_code : voucher.voucher_code.clone(),
                    signature : voucher.signature.clone(),
                    shop_id: None,
                });
            }
        }
    } else {
        println!("Status: {}", status);
    }
    Ok((freeshipping_voucher, vouchers))
}
pub fn headers_checkout(cookie_content: &CookieData) -> HeaderMap {
    let mut headers = VC_HEADER_APP.clone();
    headers.insert("af-ac-enc-dat", HeaderValue::from_str(&format!("{}", random_hex_string(16))).unwrap());
	headers.insert("x-csrftoken", HeaderValue::from_str(&cookie_content.csrftoken).unwrap());
	headers.insert("cookie", HeaderValue::from_str(&cookie_content.cookie_content).unwrap());
    headers
}

pub async fn some_function(client: Arc<reqwest::Client>, start: &str, cookie_content: &CookieData) -> Result<(String, String)> {
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

	// Bentuk struct JsonCollectionRequest
	let json_request = JsonCollectionRequest {
		voucher_collection_request_list: vec![voucher_request],
	};

	// Convert struct to JSON
	//let json_body = serde_json::to_string(&json_request)?;
	//println!("{}", json_body);
	
	loop {
        // Buat permintaan HTTP POST
        let response = (*client).clone()
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
						let (promotion_id, signature) = api_1(client, &cid_1, &headers.clone()).await?;
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

async fn api_1(client: Arc<reqwest::Client>, cid_1: &str, headers: &HeaderMap) -> Result<(String, String)> {
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
	// Bentuk struct JsonCollectionRequest
	let json_request = JsonCollectionRequest {
		voucher_collection_request_list: vec![voucher_request],
	};

	// Convert struct to JSON
	//let json_body = serde_json::to_string(&json_request)?;
	
	loop {
        let response = (*client)
            .post("https://mall.shopee.co.id/api/v1/microsite/get_vouchers_by_collections")
            .headers(cloned_headers.clone())
            .json(&json_request)
            .version(Version::HTTP_2) 
            .send()
            .await?;

		let status = response.status();
		let hasil: Value = response.json().await?;
		if status == reqwest::StatusCode::OK {
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
                                    return Ok((promotion_id, signature));
								}
							}
						}
					}else{
						println!("Bug API 2");
						println!("Tidak ada Info vouchers ditemukan untuk collection_id:{}", cid_1);
					}
				}
			}else {
				println!("Tidak ada data ditemukan untuk collection_id: {}", cid_1);
			}
			break;
		}else {
			println!("POST request gagal untuk collection_id:: {}", cid_1);
			println!("Status: {}", status);
			break;
		}
	}
	Ok((String::new(), String::new()))	
}
pub async fn headers_collection(cookie_content: &CookieData) -> HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("Android app Shopee appver=29347 app_type=1"));
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
    headers.insert("x-csrftoken", reqwest::header::HeaderValue::from_str(&cookie_content.csrftoken).unwrap());
    headers.insert(reqwest::header::COOKIE, reqwest::header::HeaderValue::from_str(&cookie_content.cookie_content).unwrap());
    headers
}
pub async fn get_voucher_by_collection_id(client: Arc<reqwest::Client>, collection_id: &JsonCollectionRequest, headers: &HeaderMap) -> Result<Vec<VoucherInfo>, Box<dyn std::error::Error>> {
    let mut voucher_data = Vec::new();

    let response = client
        .post("https://mall.shopee.co.id/api/v1/microsite/get_vouchers_by_collections")
        .headers(headers.clone())
        .json(&collection_id)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    let status = response.status();
    let hasil: Value = response.json().await?;
    if status == reqwest::StatusCode::OK {
        if let Some(data_array) = hasil.get("data").and_then(|data| data.as_array()) {
            for data_value in data_array {
                if let Some(vouchers_array) = data_value.get("vouchers").and_then(|vouchers| vouchers.as_array()) {
                    for voucher_value in vouchers_array {
                        if let Some(voucher_obj) = voucher_value.get("voucher").and_then(|voucher| voucher.as_object()) {
                            if let Some(voucher_identifier_obj) = voucher_obj.get("voucher_identifier").and_then(|vi| vi.as_object()) {
                                let ui_info_obj = voucher_obj.get("ui_info").and_then(|ui| ui.as_object());
                                voucher_data.push(VoucherInfo {
                                    promotion_id: voucher_identifier_obj.get("promotion_id").and_then(|pi| pi.as_i64()).unwrap_or(0),
                                    voucher_code: voucher_identifier_obj.get("voucher_code").and_then(|vc| vc.as_str()).unwrap_or("").to_string(),
                                    signature: voucher_identifier_obj.get("signature").and_then(|s| s.as_str()).map(String::from).unwrap_or_default(),
                                    extra_data: ui_info_obj.and_then(|ui| ui.get("icon_text").and_then(|ed| ed.as_str()).map(String::from)).unwrap_or_default(),
                                    bug: false,
                                    status,
                                });
                            }
                        }
                    }
                } else {
                    voucher_data.push(VoucherInfo{
                        promotion_id: 0,
                        voucher_code: "".to_string(),
                        signature: "".to_string(),
                        extra_data: "".to_string(),
                        bug: true,
                        status,
                    });
                }
            }
        } else {
            voucher_data.push(VoucherInfo{
                promotion_id: 0,
                voucher_code: "".to_string(),
                signature: "".to_string(),
                extra_data: "none".to_string(),
                bug: false,
                status,
            });
        }
    } else {
        voucher_data.push(VoucherInfo{
            promotion_id: 0,
            voucher_code: "".to_string(),
            signature: "".to_string(),
            extra_data: "".to_string(),
            bug: false,
            status,
        });
    }
    Ok(voucher_data)
}

pub async fn claim_food_voucher(client: Arc<reqwest::Client>, cookie_content: &CookieData, start: &str, code: &str) -> Result<Option<Vouchers>, Box<dyn std::error::Error>> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("okhttp/3.12.4 app_type=1 platform=native_android os_ver=34 appver=34560"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("Accept", HeaderValue::from_static("application/json"));
    headers.insert("Accept-Encoding", HeaderValue::from_static("gzip"));
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert(reqwest::header::COOKIE, reqwest::header::HeaderValue::from_str(&cookie_content.cookie_content)?);
	let start: i64 = start.trim().parse().expect("Input tidak valid");

    let req_data = ReqData {
        promotion_id: start,
        voucher_code: code.to_string(),
    };
    let req_data_string = to_string(&req_data)?;
	let body_json = FoodVoucherRequest {
        cmd: "voucher.core.claim_shopee_food_voucher".to_string(),
        req_data: req_data_string,
	};

    let mut vouchers: Option<Vouchers> = None;
	loop {
        let response = (*client).clone()
            .post("https://foody.shopee.co.id/api/buyer/voucher/-/action/proxy")
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
            if let Some(msg) = parsed.get("msg").and_then(|m| m.as_str()) {
                if msg == "success" {
                    println!("Berhasil: {}", msg);
                } else {
                    println!("Error: {}", msg);
                    continue;
                }
            }
            if let Some(data) = parsed.get("data") {
                if let Some(claim_error) = data.get("claim_error").and_then(|e| e.as_i64()){
                    if claim_error == 0 || claim_error == 1 {
                        println!("Berhasil: {} - {}", claim_error, data.get("debug_msg").unwrap_or(&serde_json::Value::Null));
                        vouchers = Some(Vouchers {
                            promotionid: start,
                            voucher_code: code.to_string(),
                            signature: "".to_string(),
                            shop_id: None,
                        });
                    } else {
                        println!("Error: {} - {}", claim_error, data.get("debug_msg").unwrap_or(&serde_json::Value::Null));
                        continue;
                    }
                }
            }
            break;
		} else {
			println!("Status: {}", status);
			break;
		}
	}
	Ok(vouchers)
}