use rquest as reqwest;
use reqwest::{header::HeaderMap, Version, Body};
use reqwest::header::HeaderValue;
use chrono::{Utc};
use anyhow::Result;
use serde_json::{Value, json};
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::collections::HashMap;
use anyhow::anyhow;

use crate::prepare::{CookieData, ModelInfo, ShippingInfo, PaymentInfo, ProductInfo, AddressInfo};
use crate::voucher::{Vouchers, VoucherDetail};
use crate::crypt::{self, DeviceInfo};

pub static CO_HEADER_APP: Lazy<HeaderMap> = Lazy::new(|| {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("x-api-source", HeaderValue::from_static("rn"));
    headers.insert("x-shopee-client-timezone", HeaderValue::from_static("Asia/Jakarta"));
    headers.insert("x-sap-access-f", HeaderValue::from_static(""));
    headers.insert("x-requested-with", HeaderValue::from_static("XMLHttpRequest"));
    headers.insert("x-sap-access-t", HeaderValue::from_static(""));
    headers.insert("af-ac-enc-id", HeaderValue::from_static(""));
    headers.insert("af-ac-enc-sz-token", HeaderValue::from_static(""));
    headers.insert("if-none-match-", HeaderValue::from_static("55b03-97d86fe6888b54a9c5bfa268cf3d922d"));
    headers.insert("shopee_http_dns_mode", HeaderValue::from_static("1"));
    headers.insert("x-sap-access-s", HeaderValue::from_static(""));
    headers.insert("user-agent", HeaderValue::from_static("Android app Shopee appver=29347 app_type=1"));
    headers.insert("referer", HeaderValue::from_static("https://mall.shopee.co.id/bridge_cmd?cmd=reactPath%3Ftab%3Dbuy%26path%3Dshopee%252FHOME_PAGE%253Fis_tab%253Dtrue%2526layout%253D%25255Bobject%252520Object%25255D%2526native_render%253Dsearch_prefills%25252Clanding_page_banners%25252Cwallet_bar%25252Cannouncement%25252Chome_squares%25252Cskinny_banners%25252Cnew_user_zone%25252Cearly_life_zone%25252Ccampaign_modules%25252Cflash_sales%25252Clive_streaming%25252Cvideo%25252Cdigital_products%25252Cdeals_nearby%25252Ccutline%25252Cdaily_discover%25252Cfood_order_status"));
    headers.insert("accept", HeaderValue::from_static("application/json"));
    headers.insert("content-type", HeaderValue::from_static("application/json"));
    headers
});

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CheckoutPriceData {
    pub bundle_deals_discount: Option<serde_json::Value>,
    pub buyer_service_fee: i64,
    pub buyer_txn_fee: i64,
    pub credit_card_promotion: Option<serde_json::Value>,
    pub custom_tax_subtotal: i64,
    pub disclaimers: Vec<serde_json::Value>,
    pub icms_amount: i64,
    pub import_tax_amount: i64,
    pub insurance_before_discount_subtotal: i64,
    pub insurance_discount_subtotal: i64,
    pub insurance_subtotal: i64,
    pub iof_amount: i64,
    pub merchandise_subtotal: i64,
    pub payment_plan: Option<serde_json::Value>,
    pub price_adjustment: Option<serde_json::Value>,
    pub promocode_applied: Option<serde_json::Value>,
    pub shipping_discount_subtotal: i64,
    pub shipping_sst_amount: Option<serde_json::Value>,
    pub shipping_subtotal: i64,
    pub shipping_subtotal_before_discount: i64,
    pub shopee_coins_redeemed: Option<serde_json::Value>,
    pub tax_exemption: i64,
    pub tax_payable: i64,
    pub total_payable: i64,
    pub total_savings: i64,
    pub vat_subtotal: i64,
}
impl CheckoutPriceData{
	pub fn parse_checkout_price_data(checkout_price_data: &serde_json::Value) -> Result<CheckoutPriceData, anyhow::Error> {
		let parsed: CheckoutPriceData = serde_json::from_value(checkout_price_data.clone())
			.map_err(|e| anyhow!("Gagal parse checkout_price_data: {}", e))?;
		Ok(parsed)
	}
}

pub async fn place_order(client: Arc<reqwest::Client>, cookie_content: &CookieData, body_json: &serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut headers = headers_checkout(&cookie_content);
    let data = crypt::random_hex_string(16);
    headers.insert("af-ac-enc-dat", HeaderValue::from_str(&data).unwrap());
    // Convert struct to JSON
    let body_str = serde_json::to_string(&body_json).unwrap();
    let body = Body::from(body_str.clone());
    println!("Status: Start Place_Order");
    //println!("{:?}", body_str);
    //println!("Request Headers:\n{:?}", headers);

    let url2 = format!("https://mall.shopee.co.id/api/v4/checkout/place_order");
    println!("{}", url2);

    // Buat permintaan HTTP POST
    let response = client
        .post(&url2)
        .header("Content-Type", "application/json")
        .headers(headers)
        .body(body)
        .version(Version::HTTP_2) 
        .send()
        .await?;
    
    println!("Status: Done Place_Order");
    //println!("Status: {}", response.status());
    let hasil: Value = response.json().await?;
    println!("Body: {}", hasil);
    Ok(hasil)
}

pub async fn place_order_builder(device_info: &DeviceInfo, checkout_price_data: &serde_json::Value, order_update_info: &serde_json::Value, dropshipping_info: &serde_json::Value, promotion_data: &serde_json::Value, chosen_payment: &PaymentInfo, shoporders: &serde_json::Value, shipping_orders: &serde_json::Value, display_meta_data: &serde_json::Value, fsv_selection_infos: &serde_json::Value, buyer_info: &serde_json::Value, client_event_info: &serde_json::Value, buyer_txn_fee_info: &serde_json::Value, disabled_checkout_info: &serde_json::Value, buyer_service_fee_info: &serde_json::Value, iof_info: &serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let channel_id = chosen_payment.channel_id;
    let version = chosen_payment.version;
    let optioninfo: String = chosen_payment.option_info.clone();
    let current_time = Utc::now();
    let selected_payment_channel_data = if chosen_payment.place_order.is_null(){
        json!({
            "channel_id": channel_id,
            "channel_item_option_info": {
              "option_info": optioninfo
            },
            "version": version
        })
    }else{
        chosen_payment.place_order.clone()
    };
    let body_json = json!({
        "client_id": 5,
        "cart_type": 1,
        "timestamp": current_time.timestamp(),
        "checkout_price_data": checkout_price_data,
        "order_update_info": order_update_info,
        "dropshipping_info": dropshipping_info,
        "promotion_data": promotion_data,
        "selected_payment_channel_data": selected_payment_channel_data,
        "shoporders": shoporders,
        "shipping_orders": shipping_orders,
        "display_meta_data": display_meta_data,
        "fsv_selection_infos": fsv_selection_infos,
        "buyer_info": buyer_info,
        "client_event_info": client_event_info,
        "captcha_id": "",
        "buyer_txn_fee_info": buyer_txn_fee_info,
        "disabled_checkout_info": disabled_checkout_info,
        "can_checkout": true,
        "buyer_service_fee_info": buyer_service_fee_info, 
        "iof_info": iof_info,
        "add_to_cart_info": {},
        "ignored_errors": [0],
        "ignore_warnings": false,
        "captcha_version": 1,
        "captcha_signature": "",
        "extra_data": {
          "snack_click_id": null
        },
        "device_info": device_info,
        "device_type": "mobile",
        "_cft": vec![4227792767 as i64, 36961919]
      });
    //println!("{body_json}");
    Ok(body_json)
}
pub async fn get_wtoken_builder(token: &str, device_info: &DeviceInfo, product_info: &ProductInfo, address_info: &AddressInfo, quantity: i32, chosen_model: &ModelInfo, chosen_payment: &PaymentInfo, chosen_shipping: &ShippingInfo) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let optioninfo: String = chosen_payment.option_info.clone();

    let body_json = json!({
      "shoporders": [
        {
          "shop": {
            "shopid": product_info.shop_id
          },
          "items": [
            {
              "itemid": product_info.item_id,
              "modelid": chosen_model.modelid as i64,
              "quantity": quantity as i32,
              "insurances": [],
              "channel_exclusive_info": {
                "source_id": 1,
                "token": token
              },
            }
          ],
        }
      ],
      "selected_payment_channel_data": {
        "page": "OPC_PAYMENT_SELECTION",
        "removed_vouchers": [],
        "channel_id": chosen_payment.channel_id,
        "version": chosen_payment.version,
        "group_id": 0,
        "channel_item_option_info": {
          "option_info": optioninfo
        },
        "additional_info": {}
      },
      "promotion_data": {
        "auto_apply_shop_voucher": true,
        "check_shop_voucher_entrances": true,
        "auto_apply_platform_voucher": true,
        "auto_apply_spl_voucher": true,
        "spl_voucher_info": null,
        "free_shipping_voucher_info": {},
        "platform_vouchers": [],
        "shop_vouchers": [],
        "use_coins": false
      },
      "fsv_selection_infos": [],
      "device_info": device_info,
      "buyer_info": {
        "kyc_info": null,
        "checkout_email": ""
      },
      "cart_type": 1,
      "client_id": 5,
      "tax_info": {
        "tax_id": ""
      },
      "client_event_info": {
        "is_fsv_changed": false,
        "is_platform_voucher_changed": false
      },
      "add_to_cart_info": {},
      "_cft": [469696383],
      "dropshipping_info": {},
      "shipping_orders": [
        {
          "sync": true,
          "buyer_address_data": {
            "addressid": address_info.id,
            "address_type": 0,
            "tax_address": ""
          },
          "selected_logistic_channelid": chosen_shipping.channelid,
          "shipping_id": 1,
          "shoporder_indexes": [
            0
          ],
          "selected_preferred_delivery_time_option_id": 0,
          "prescription_info": {
            "images": []
          },
          "fulfillment_info": {
            "fulfillment_flag": 18,
            "fulfillment_source": "IDE",
            "managed_by_sbs": false,
            "order_fulfillment_type": 1,
            "warehouse_address_id": 0,
            "is_from_overseas": false
          }
        }
      ],
      "order_update_info": {}
    });
    //println!("{body_json}");
    Ok(body_json)
}
pub async fn get_builder(device_info: &DeviceInfo, address_info: &AddressInfo, chosen_model: &[ModelInfo], chosen_payment: &PaymentInfo, chosen_shipping: &ShippingInfo, freeshipping_voucher: Option<Vouchers>, platform_vouchers_target: Option<Vouchers>, shop_vouchers_target: Option<Vouchers>) -> Result<serde_json::Value, anyhow::Error> {
    let optioninfo: String = chosen_payment.option_info.clone();
    let current_time = Utc::now();
    let mut grouped: HashMap<i64, Vec<Value>> = HashMap::new();
    for chosen in chosen_model {
        let item = json!({
            "itemid": chosen.item_id,
            "modelid": chosen.modelid as i64,
            "quantity": chosen.quantity as i32,
            "add_on_deal_id": 0,
            "is_add_on_sub_item": false,
            "item_group_id": Value::Null,
            "insurances": [],
            "channel_exclusive_info": {
                "source_id": 0,
                "token": "",
                "is_live_stream": false,
                "is_short_video": false
            },
            "supports_free_returns": false
        });

        grouped.entry(chosen.shop_id).or_default().push(item);
    }
    let mut raw_shoporders = Vec::new();
    for (index, (shop_id, items)) in grouped.into_iter().enumerate() {
        let shoporder = json!({
            "shop": {
                "shopid": shop_id
            },
            "items": items,
            "shipping_id": index + 1
        });
        raw_shoporders.push(shoporder);
    }

    let shoporders = Value::Array(raw_shoporders);
    let selected_payment_channel_data = if chosen_payment.selected_get.is_null(){
        json!({
            "page": "OPC_PAYMENT_SELECTION",
            "removed_vouchers": [],
            "channel_id": chosen_payment.channel_id,
            "version": chosen_payment.version,
            "group_id": 0,
            "channel_item_option_info": {
              "option_info": optioninfo
            },
            "additional_info": {}
          })
    } else if chosen_payment.channel_id == 0{
        json!({})
    } else{
        chosen_payment.selected_get.clone()
    };

    let shop_vouchers = if let Some(shop) = shop_vouchers_target {
        json!([
            {
              "shopid": shop.shop_id,
              "promotionid": shop.promotionid,
              "voucher_code": shop.voucher_code,
              "applied_voucher_code": shop.voucher_code,
              "invalid_message_code": 0,
              "reward_type": 0,
              "shipping_order_distributions": []
            }
          ])
    } else {
        json!([])
    };
    let platform_vouchers = if let Some(platform) = platform_vouchers_target {
        json!([{
            "voucher_code": platform.voucher_code,
            "promotionid": platform.promotionid
        }])
    } else {
        json!([])
    };
    let free_shipping_voucher_info = if let Some(ref shipping_vc) = freeshipping_voucher {
        json!({
            "free_shipping_voucher_id": shipping_vc.promotionid,
            "free_shipping_voucher_code": shipping_vc.voucher_code,
            "disabled_reason": null,
            "disabled_reason_code": 0,
            "banner_info": {
                "banner_type": 5,
                "learn_more_msg": "",
                "msg": "Berhasil mendapatkan Gratis Ongkir"
            },
            "required_be_channel_ids": [],
            "required_spm_channels": []
        })
    }else{
        json!({
            "free_shipping_voucher_id": 0,
            "disabled_reason": "",
            "description": "",
            "disabled_reason_code": 0
        })
    };
    let fsv_selection_infos = if let Some(shipping_vca) = freeshipping_voucher {
        json!([{
          "fsv_id": shipping_vca.promotionid,
          "selected_shipping_ids": [1],
          "potentially_applied_shipping_ids": [1]
        }])
    } else {
        json!([])
    };

    let shipping_orders = if chosen_shipping.channelid == 0 {
        vec![]
    } else {
        vec![
            json!({
                "sync": true,
                "buyer_address_data": {
                    "addressid": address_info.id,
                    "address_type": 0,
                    "tax_address": ""
                },
                "selected_logistic_channelid": chosen_shipping.channelid,
                "shipping_id": 1,
                "shoporder_indexes": [0],
                "selected_preferred_delivery_time_option_id": 0,
                "prescription_info": {
                    "images": []
                },
                "fulfillment_info": {
                    "fulfillment_flag": 18,
                    "fulfillment_source": "IDE",
                    "managed_by_sbs": false,
                    "order_fulfillment_type": 1,
                    "warehouse_address_id": 0,
                    "is_from_overseas": false
                }
            })
        ]
    };
    let body_json = json!({
      "timestamp": current_time.timestamp(),
      "shoporders": shoporders,
      "selected_payment_channel_data": selected_payment_channel_data,
      "promotion_data": {
        "use_coins": true,
        "free_shipping_voucher_info": free_shipping_voucher_info,
        "platform_vouchers": platform_vouchers,
        "shop_vouchers": shop_vouchers,
        "check_shop_voucher_entrances": true,
        "auto_apply_shop_voucher": false
      },
      "fsv_selection_infos": fsv_selection_infos,
      "device_info": device_info,
      "buyer_info": {
        "kyc_info": null,
        "checkout_email": ""
      },
      "cart_type": 1,
      "client_id": 5,
      "tax_info": {
        "tax_id": ""
      },
      "client_event_info": {
        "is_fsv_changed": false,
        "is_platform_voucher_changed": false
      },
      "add_to_cart_info": {},
      "_cft": vec![4227792767 as i64, 36961919],
      "dropshipping_info": {},
      "shipping_orders": shipping_orders,
      "order_update_info": {}
    });
    //println!("{body_json}");
    Ok(body_json)
}
pub async fn checkout_get(client: Arc<reqwest::Client>, base_headers: Arc<HeaderMap>, body_json: &serde_json::Value) -> Result<(serde_json::Value, serde_json::Value, serde_json::Value, serde_json::Value, serde_json::Value, serde_json::Value, serde_json::Value, serde_json::Value, serde_json::Value, serde_json::Value, serde_json::Value, serde_json::Value, serde_json::Value, serde_json::Value, serde_json::Value), anyhow::Error> {
    let mut headers = (*base_headers).clone();
    headers.insert("af-ac-enc-dat", HeaderValue::from_str(&crypt::random_hex_string(16)).unwrap());
    // Convert struct to JSON
    let body_str = serde_json::to_string(&body_json).unwrap();
    let body = Body::from(body_str.clone());
    println!("Status: Start Checkout");
    //println!("{:?}", body_str);
    //println!("Request Headers:\n{:?}", headers);

    let url2 = format!("https://mall.shopee.co.id/api/v4/checkout/get");
    println!("{}", url2);
    // Buat permintaan HTTP POST
    let response = client
        .post(&url2)
        .header("Content-Type", "application/json")
        .headers(headers)
        .body(body)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    println!("Status: Done Checkout");
    // Handle response as needed
    //println!("Request Headers:\n{:?}", headers);
    println!("Status: {}", response.status());
    let v: Value = response.json().await?;
    //println!("Body: {}", body_resp);
    // Mengambil data checkout_price_data
    // Mengambil referensi langsung tanpa cloning
    let checkout_price_data = v.get("checkout_price_data").unwrap_or(&serde_json::Value::Null);
    let order_update_info = v.get("order_update_info").unwrap_or(&serde_json::Value::Null);
    let dropshipping_info = v.get("dropshipping_info").unwrap_or(&serde_json::Value::Null);
    let promotion_data = v.get("promotion_data").unwrap_or(&serde_json::Value::Null);
    let selected_payment_channel_data = v.get("selected_payment_channel_data").unwrap_or(&serde_json::Value::Null);
    let shoporders = v.get("shoporders").unwrap_or(&serde_json::Value::Null);
    let shipping_orders = v.get("shipping_orders").unwrap_or(&serde_json::Value::Null);
    let display_meta_data = v.get("display_meta_data").unwrap_or(&serde_json::Value::Null);
    let fsv_selection_infos = v.get("fsv_selection_infos").unwrap_or(&serde_json::Value::Null);
    let buyer_info = v.get("buyer_info").unwrap_or(&serde_json::Value::Null);
    let client_event_info = v.get("client_event_info").unwrap_or(&serde_json::Value::Null);
    let buyer_txn_fee_info = v.get("buyer_txn_fee_info").unwrap_or(&serde_json::Value::Null);
    let disabled_checkout_info = v.get("disabled_checkout_info").unwrap_or(&serde_json::Value::Null);
    let buyer_service_fee_info = v.get("buyer_service_fee_info").unwrap_or(&serde_json::Value::Null);
    let iof_info = v.get("iof_info").unwrap_or(&serde_json::Value::Null);

    Ok((
        checkout_price_data.clone(),
        order_update_info.clone(),
        dropshipping_info.clone(),
        promotion_data.clone(),
        selected_payment_channel_data.clone(),
        shoporders.clone(),
        shipping_orders.clone(),
        display_meta_data.clone(),
        fsv_selection_infos.clone(),
        buyer_info.clone(),
        client_event_info.clone(),
        buyer_txn_fee_info.clone(),
        disabled_checkout_info.clone(),
        buyer_service_fee_info.clone(),
        iof_info.clone(),
    ))
}

pub fn headers_checkout(cookie_content: &CookieData) -> HeaderMap {
    let mut headers = CO_HEADER_APP.clone();
    headers.insert("x-csrftoken", HeaderValue::from_str(&cookie_content.csrftoken).unwrap());
    headers.insert("cookie", HeaderValue::from_str(&cookie_content.cookie_content).unwrap());
    headers
}
pub fn recalculate_shipping_subtotal(checkout_price_data: &mut serde_json::Value, adjusted_max_price: i64) {
    let shipping_subtotal = checkout_price_data["shipping_subtotal"].as_i64().unwrap_or(0);
    let insurance_subtotal = checkout_price_data["insurance_subtotal"].as_i64().unwrap_or(0);
    let buyer_service_fee = checkout_price_data["buyer_service_fee"].as_i64().unwrap_or(0);
    let buyer_txn_fee = checkout_price_data["buyer_txn_fee"].as_i64().unwrap_or(0);
    let promocode_applied = checkout_price_data["promocode_applied"].as_i64().unwrap_or(0); //Need Api Comunication
    let shopee_coins_redeemed = checkout_price_data["shopee_coins_redeemed"].as_i64().unwrap_or(0); //Need Api Comunication
    let fixed_total_payable = adjusted_max_price + shipping_subtotal + insurance_subtotal + buyer_service_fee + buyer_txn_fee - promocode_applied - shopee_coins_redeemed;
    if let Some(obj) = checkout_price_data.as_object_mut() {
        obj.remove("price_breakdown");
        obj.remove("total_savings");
        obj.insert("merchandise_subtotal".to_string(), serde_json::Value::from(adjusted_max_price));
        obj.insert("total_payable".to_string(), serde_json::Value::from(fixed_total_payable));
    }
}
pub fn recalculate_client_voucher(checkout_price_data: &mut serde_json::Value, detail: &VoucherDetail) {
    let merchandise_subtotal = checkout_price_data["merchandise_subtotal"].as_i64().unwrap_or(0);
    let shipping_subtotal = checkout_price_data["shipping_subtotal"].as_i64().unwrap_or(0);
    let insurance_subtotal = checkout_price_data["insurance_subtotal"].as_i64().unwrap_or(0);
    let buyer_service_fee = checkout_price_data["buyer_service_fee"].as_i64().unwrap_or(0);
    let buyer_txn_fee = checkout_price_data["buyer_txn_fee"].as_i64().unwrap_or(0);
    let diskon_normal = (merchandise_subtotal * (detail.reward_percentage as i64)) / 100; // aman overflow
    let max_diskon = detail.reward_cap as i64;
    let promocode_applied = std::cmp::min(diskon_normal, max_diskon);
    let shopee_coins_redeemed = checkout_price_data["shopee_coins_redeemed"].as_i64().unwrap_or(0); //Need Api Comunication

    let fixed_total_payable = if merchandise_subtotal == promocode_applied {
      0
    }else{
      merchandise_subtotal + shipping_subtotal + insurance_subtotal + buyer_service_fee + buyer_txn_fee - promocode_applied - shopee_coins_redeemed
    };
    if let Some(obj) = checkout_price_data.as_object_mut() {
        obj.remove("price_breakdown");
        obj.remove("total_savings");
        obj.insert("total_payable".to_string(), serde_json::Value::from(fixed_total_payable));
        obj.insert("promocode_applied".to_string(), serde_json::Value::from(promocode_applied));
        if fixed_total_payable == 0 {
            obj.insert("buyer_service_fee".to_string(), serde_json::Value::Null);
        }
    }
}