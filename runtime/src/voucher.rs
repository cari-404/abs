use rquest as reqwest;
use reqwest::tls::Impersonate;
use reqwest::{ClientBuilder, header::HeaderMap, Version, Body};
use reqwest::header::HeaderValue;
use serde_json::{json, to_string, Value};
use serde::Serialize;
use anyhow::Result;

use crate::prepare::ShippingInfo;
use crate::prepare::ModelInfo;
use crate::prepare::PaymentInfo;
use crate::prepare::extract_csrftoken;

#[derive(Debug, Clone)]
pub struct Vouchers {
    pub promotionid: i64,
    pub voucher_code: String,
    pub signature: String,
}

#[derive(Serialize)]
struct SaveVoucherRequest {
	voucher_promotionid: i64,
	signature: String,
	security_device_fingerprint: String,
	signature_source: String,
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

pub async fn save_platform_voucher_by_voucher_code(code: &str, cookie_content: &str) -> Result<Option<Vouchers>, Box<dyn std::error::Error>>{
    let headers = headers_checkout(&cookie_content);

    let body_json = json!({
        "voucher_code": code.to_string()
    });

    // Convert struct to JSON
    let body_str = serde_json::to_string(&body_json).unwrap();
    //println!("{:?}", body_str);
    //println!("{:?}", body);
    //println!("Request Headers:\n{:?}", headers);
    let mut vouchers: Option<Vouchers> = None;
	loop {
        let url2 = format!("https://mall.shopee.co.id/api/v2/voucher_wallet/save_platform_voucher_by_voucher_code");
        println!("{}", url2);
        // Buat klien HTTP
        let client = ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .impersonate_with_headers(Impersonate::Chrome127, false)
            .enable_ech_grease()
            .permute_extensions()
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
		let text = response.text().await?;	
        //println!("Body: {}", body);
        // Parse response body as JSON
        if status == reqwest::StatusCode::OK {
            let parsed: serde_json::Value = serde_json::from_str(&text).expect("JSON parsing failed");
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
            println!("{}", text);
            continue;
        }else {
            println!("Status: {}", status);
            break;
        }
    }
    Ok(vouchers)
}

pub async fn save_voucher(start: &str, end: &str, cookie_content: &str) -> Result<Option<Vouchers>, Box<dyn std::error::Error>>{
	let cookie_content_owned = cookie_content.to_string();

	// Pass the cloned String to extract_csrftoken
	let csrftoken = extract_csrftoken(&cookie_content_owned);
	//println!("csrftoken: {}", csrftoken);
	let csrftoken_string = csrftoken.to_string();
	let start: i64 = start.trim().parse().expect("Input tidak valid");

	let body_json = SaveVoucherRequest {
	  voucher_promotionid: start as i64,
	  signature: end.to_string(),
	  security_device_fingerprint: String::new(),
	  signature_source: 0.to_string(),
	};
	
	let body_str = serde_json::to_string(&body_json)?;

	println!("{}", body_str);
	
	let mut headers = reqwest::header::HeaderMap::new();
	headers.insert("User-Agent", reqwest::header::HeaderValue::from_static("Android app Shopee appver=29330 app_type=1"));
	headers.insert("accept", reqwest::header::HeaderValue::from_static("application/json"));
	headers.insert("Content-Type", reqwest::header::HeaderValue::from_static("application/json"));
	headers.insert("x-api-source", reqwest::header::HeaderValue::from_static("rn"));
	headers.insert("if-none-match-", reqwest::header::HeaderValue::from_static("55b03-97d86fe6888b54a9c5bfa268cf3d922f"));
	headers.insert("shopee_http_dns_mode", reqwest::header::HeaderValue::from_static("1"));
	headers.insert("x-shopee-client-timezone", reqwest::header::HeaderValue::from_static("Asia/Jakarta"));
	headers.insert("af-ac-enc-dat", reqwest::header::HeaderValue::from_static(""));
	headers.insert("af-ac-enc-id", reqwest::header::HeaderValue::from_static(""));
	headers.insert("x-sap-access-t", reqwest::header::HeaderValue::from_static(""));
	headers.insert("x-sap-access-f", reqwest::header::HeaderValue::from_static(""));
	headers.insert("referer", reqwest::header::HeaderValue::from_static("https://mall.shopee.co.id/"));
	headers.insert("x-csrftoken", reqwest::header::HeaderValue::from_str(&csrftoken_string)?);
	headers.insert("af-ac-enc-sz-token", reqwest::header::HeaderValue::from_static(""));
	headers.insert(reqwest::header::COOKIE, reqwest::header::HeaderValue::from_str(&cookie_content)?);

	//println!("");
	//println!("header:{:#?}", headers);
    let mut vouchers: Option<Vouchers> = None;
	loop {
        let client = ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .impersonate_with_headers(Impersonate::Chrome127, false)
            .enable_ech_grease()
            .permute_extensions()
            .gzip(true)
            //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
            .build()?;
    
        // Buat permintaan HTTP POST
        let response = client
            .post("https://mall.shopee.co.id/api/v2/voucher_wallet/save_voucher")
            .header("Content-Type", "application/json")
			.headers(headers.clone())
			.body(body_str.clone())
            .version(Version::HTTP_2) 
            .send()
            .await?;
		// Check for HTTP status code indicating an error
		//let http_version = response.version(); 		// disable output features
		//println!("HTTP Version: {:?}", http_version); // disable output features
		let status = response.status();
		println!("{}", status);
		let text = response.text().await?;	
		if status == reqwest::StatusCode::OK {
            let parsed: serde_json::Value = serde_json::from_str(&text).expect("JSON parsing failed");
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
			println!("{}", text);
			continue;
		}else {
			println!("Status: {}", status);
			break;
		}
	}
	Ok(vouchers)
}

pub async fn get_recommend_platform_vouchers(cookie_content: &str, shop_id_str: &str, item_id_str: &str, addressid_str: &str, quantity_str: &str, chosen_model: &ModelInfo, chosen_payment: &PaymentInfo, chosen_shipping: &ShippingInfo) -> Result<(Option<Vouchers>, Option<Vouchers>), Box<dyn std::error::Error>>{
    let headers = headers_checkout(&cookie_content);
    let shop_id = shop_id_str.parse::<i64>().expect("Failed to parse shop_id");
	let addressid = addressid_str.parse::<i64>().expect("Failed to parse addressid");
	let item_id = item_id_str.parse::<i64>().expect("Failed to parse item_id");
	let quantity = quantity_str.parse::<i64>().expect("Failed to parse quantity");
	let channel_id: u64 = chosen_payment.channel_id.parse().expect("Failed to parse channel_id");
	let version: u64 = chosen_payment.version.parse().expect("Failed to parse version");
	let optioninfo: String = chosen_payment.option_info.clone();
    let orders_json = json!([{
        "shopid": shop_id,
        "carrier_ids": [8005, 8003, 80099, 80055, 8006, 80021],
        "shop_vouchers": [],
        "auto_apply": true,
        "iteminfos": [{
            "itemid": item_id,
            "modelid": chosen_model.modelid,
            "quantity": quantity,
            "item_group_id": null,
            "insurances": [],
            "shopid": shop_id,
            "shippable": true,
            "non_shippable_err": "",
            "none_shippable_reason": "",
            "none_shippable_full_reason": "",
            "add_on_deal_id": 0,
            "is_add_on_sub_item": false,
            "is_pre_order": false,
            "is_streaming_price": false,
            "checkout": true,
            "categories": [{
                "catids": [100013, 100073]
            }],
            "is_spl_zero_interest": false,
            "is_prescription": false,
            "offerid": 0,
            "supports_free_returns": false,
            "user_path": 1,
            "models": null,
            "tier_variations": null
        }],
        "selected_carrier_id": chosen_shipping.channelid
    }]);
    // Konversi orders_json menjadi string
    let orders_string = to_string(&orders_json).unwrap();
    let body_json = json!({
        "orders": orders_string,
        "voucher_market_type": 1,
        "check_voucher_payment_criteria": true,
        "selected_payment_channel_data": {
            "version": version,
            "option_info": "",
            "channel_id": channel_id,
            "channel_item_option_info": {
                "option_info": optioninfo
            },
            "text_info": {}
        },
        "spm_channel_id": channel_id,
        "need_wallet_active_info": true,
        "sorting_flag": 8,
        "priority_promotion_ids": [],
        "has_redeem_coins": false
    });

    // Convert struct to JSON
    let body_str = serde_json::to_string(&body_json).unwrap();
    let body = Body::from(body_str.clone());
    //println!("{:?}", body_str);
    //println!("{:?}", body);
    //println!("Request Headers:\n{:?}", headers);

    let url2 = format!("https://mall.shopee.co.id/api/v2/voucher_wallet/get_recommend_platform_vouchers");
    println!("{}", url2);
    // Buat klien HTTP
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .impersonate_with_headers(Impersonate::Chrome127, false)
        .enable_ech_grease()
        .permute_extensions()
        .gzip(true)
        //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
        .build()?;

    // Buat permintaan HTTP POST
    let response = client
        .post(&url2)
        .header("Content-Type", "application/json")
        .headers(headers)
        .body(body)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    println!("Status: get_voucher");
    // Handle response as needed
    //println!("Request Headers:\n{:?}", headers);
    println!("Status: {}", response.status());
    let body = response.text().await?;
    //println!("Body: {}", body);
    // Parse response body as JSON
    let json: Value = serde_json::from_str(&body)?;
    let mut freeshipping_voucher: Option<Vouchers> = None;
    let mut vouchers: Option<Vouchers> = None;
    // Extract freeshipping_vouchers
    if let Some(freeshipping_vouchers_array) = json["data"]["freeshipping_vouchers"].as_array() {
        for voucher in freeshipping_vouchers_array {
            if voucher["fsv_error_message"].is_null() {
                let promotionid = voucher["promotionid"].as_i64().unwrap_or_default();
                let voucher_code = voucher["voucher_code"].as_str().unwrap_or_default().to_string();
                let signature = voucher["signature"].as_str().unwrap_or_default().to_string();
                freeshipping_voucher = Some(Vouchers {
                    promotionid,
                    voucher_code,
                    signature,
                });
                //println!("Freeshipping Voucher: {}, {}, {}", promotionid, voucher_code, signature);
                break; // Found one valid voucher, so break
            }
        }
    }

    // Extract vouchers
    if let Some(vouchers_array) = json["data"]["vouchers"].as_array() {
        for voucher in vouchers_array {
            if voucher["fsv_error_message"].is_null() {
                let promotionid = voucher["promotionid"].as_i64().unwrap_or_default();
                let voucher_code = voucher["voucher_code"].as_str().unwrap_or_default().to_string();
                let signature = voucher["signature"].as_str().unwrap_or_default().to_string();
                vouchers = Some(Vouchers {
                    promotionid,
                    voucher_code,
                    signature,
                });
                //println!("Voucher: {}, {}, {}", promotionid, voucher_code, signature);
                break; // Found one valid voucher, so break
            }
        }
    }
    Ok((freeshipping_voucher, vouchers))
}
fn headers_checkout(cookie_content: &str) -> HeaderMap {
    let csrftoken = extract_csrftoken(&cookie_content);
    let mut headers = reqwest::header::HeaderMap::new();
	headers.insert("x-api-source", HeaderValue::from_static("rn"));
	headers.insert("x-shopee-client-timezone", HeaderValue::from_static("Asia/Jakarta"));
	headers.insert("x-sap-access-f", HeaderValue::from_static(" "));
	headers.insert("x-sap-access-t", HeaderValue::from_static(" "));
	headers.insert("af-ac-enc-dat", HeaderValue::from_static("f2b1b94227db5eab"));
	headers.insert("af-ac-enc-id", HeaderValue::from_static(" "));
	headers.insert("af-ac-enc-sz-token", HeaderValue::from_static(" "));
	headers.insert("if-none-match-", HeaderValue::from_static("55b03-97d86fe6888b54a9c5bfa268cf3d922d"));
	headers.insert("shopee_http_dns_mode", HeaderValue::from_static("1"));
	headers.insert("x-sap-access-s", HeaderValue::from_static(" "));
	headers.insert("x-csrftoken", HeaderValue::from_str(csrftoken).unwrap());
	headers.insert("user-agent", HeaderValue::from_static("Android app Shopee appver=29313 app_type=1"));
	headers.insert("referer", HeaderValue::from_static("https://mall.shopee.co.id"));
	headers.insert("accept", HeaderValue::from_static("application/json"));
	headers.insert("content-type", HeaderValue::from_static("application/json; charset=utf-8"));
	headers.insert("cookie", HeaderValue::from_str(cookie_content).unwrap());
    // Return the created headers
    headers
}

pub async fn some_function(start: &str, cookie_content: &str) -> Result<(String, String)> {
    let cookie_content_owned = cookie_content.to_string();
    let csrftoken = extract_csrftoken(&cookie_content_owned);
    println!("csrftoken: {}", csrftoken);
	let csrftoken_string = csrftoken.to_string();
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
    headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("accept", HeaderValue::from_static("application/json"));
    headers.insert("Accept-Encoding", HeaderValue::from_static("gzip"));
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("sec-ch-ua", HeaderValue::from_static("\"Not)A;Brand\";v=\"99\", \"Google Chrome\";v=\"127\", \"Chromium\";v=\"127\""));
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert("x-sz-sdk-version", HeaderValue::from_static("1.10.12"));
    headers.insert("x-api-source", HeaderValue::from_static("pc"));
    headers.insert("x-sap-ri", HeaderValue::from_static("8fab8288812ce5572fd20624a59333cea398a23b43b3f793"));
    headers.insert("x-shopee-language", HeaderValue::from_static("id"));
    headers.insert("x-requested-with", HeaderValue::from_static("XMLHttpRequest"));
    headers.insert("af-ac-enc-dat", HeaderValue::from_static("d4fd3f0079b47b69"));
    headers.insert("af-ac-enc-sz-token", HeaderValue::from_static(" "));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Windows\""));
    headers.insert("origin", HeaderValue::from_static("https://shopee.co.id"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    headers.insert("accept-language", HeaderValue::from_static("en-US,en;q=0.9,id;q=0.8"));
    headers.insert("referer", HeaderValue::from_static("https://shopee.co.id/"));
    headers.insert("x-csrftoken", reqwest::header::HeaderValue::from_str(&csrftoken_string)?);
    headers.insert(reqwest::header::COOKIE, reqwest::header::HeaderValue::from_str(&cookie_content)?);

	// Bentuk struct JsonRequest
	let json_request = JsonRequest {
		voucher_collection_request_list: vec![voucher_request],
	};

	// Convert struct to JSON
	let json_body = serde_json::to_string(&json_request)?;
	//println!("{}", json_body);
	
	loop {
        let client = ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .impersonate(Impersonate::Chrome127)
            .enable_ech_grease()
            .permute_extensions()
            .gzip(true)
            //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
            .build()?;

        // Buat permintaan HTTP POST
        let response = client
            .post("https://mall.shopee.co.id/api/v1/microsite/get_vouchers_by_collections")
            .header("Content-Type", "application/json")
            .headers(headers.clone())
            .body(json_body.clone())
            .version(Version::HTTP_2) 
            .send()
            .await?;

		// Check for HTTP status code indicating an error
		//let http_version = response.version(); 		// disable output features
		//println!("HTTP Version: {:?}", http_version); // disable output features
		let status = response.status();
		let text = response.text().await?;
		//println!("{}", text);
		if status == reqwest::StatusCode::OK {
			let hasil: Value = serde_json::from_str(&text)?;
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
			println!("{}", text);
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
	let json_body = serde_json::to_string(&json_request)?;
	
	loop {
        let client = ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .impersonate(Impersonate::Chrome127)
            .enable_ech_grease()
            .permute_extensions()
            .gzip(true)
            //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
            .build()?;

        // Buat permintaan HTTP POST
        let response = client
            .post("https://mall.shopee.co.id/api/v1/microsite/get_vouchers_by_collections")
            .header("Content-Type", "application/json")
            .headers(cloned_headers.clone())
            .body(json_body.clone())
            .version(Version::HTTP_2) 
            .send()
            .await?;
		// Check for HTTP status code indicating an error
		//let http_version = response.version(); 		// disable output features
		//println!("HTTP Version: {:?}", http_version); // disable output features
		let status = response.status();
		let text = response.text().await?;
		if status == reqwest::StatusCode::OK {
			let hasil: Value = serde_json::from_str(&text)?;
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
			println!("{}", text);
			continue;
		}else {
			println!("POST request gagal untuk collection_id:: {}", cid_1);
			println!("Status: {}", status);
			break;
		}
	}
	Ok((String::new(), String::new()))	
}