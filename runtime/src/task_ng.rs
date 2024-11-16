use rquest as reqwest;
use reqwest::tls::Impersonate;
use reqwest::{ClientBuilder, Version};
use reqwest::header::HeaderValue;
use chrono::{Utc};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use rayon::prelude::*;

use crate::prepare::ShippingInfo;
use crate::prepare::ModelInfo;
use crate::prepare::PaymentInfo;
use crate::voucher::Vouchers;
use crate::task::headers_checkout;
use crate::crypt::{self};

#[derive(Serialize, Deserialize, Debug)]
pub struct PlaceOrderBody {
    client_id: i64,
    cart_type: i64,
    timestamp: i64,
    checkout_price_data: serde_json::Value, // Define more specific types if known
    order_update_info: serde_json::Value,
    dropshipping_info: serde_json::Value,
    promotion_data: serde_json::Value,
    selected_payment_channel_data: serde_json::Value,
    shoporders: serde_json::Value,
    shipping_orders: serde_json::Value,
    display_meta_data: serde_json::Value,
    fsv_selection_infos: serde_json::Value,
    buyer_info: serde_json::Value,
    client_event_info: serde_json::Value,
    captcha_id: String,
    buyer_txn_fee_info: serde_json::Value,
    disabled_checkout_info: serde_json::Value,
    can_checkout: bool,
    buyer_service_fee_info: serde_json::Value,
    iof_info: serde_json::Value,
    add_to_cart_info: HashMap<String, serde_json::Value>,
    ignored_errors: Vec<i64>,
    ignore_warnings: bool,
    captcha_version: i64,
    captcha_signature: String,
    extra_data: ExtraData,
    device_info: serde_json::Value,
    device_type: String,
    _cft: Vec<i64>,
}
#[derive(Serialize, Deserialize, Debug)]
struct ExtraData {
    snack_click_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct BodyJson {
    timestamp: i64,
    shoporders: Vec<ShopOrder>,
    selected_payment_channel_data: serde_json::Value,
    promotion_data: PromotionData,
    fsv_selection_infos: serde_json::Value,
    device_info: serde_json::Value,
    buyer_info: BuyerInfo,
    cart_type: i32,
    client_id: i32,
    tax_info: TaxInfo,
    client_event_info: ClientEventInfo,
    add_to_cart_info: AddToCartInfo,
    _cft: Vec<i64>,
    dropshipping_info: DropshippingInfo,
    shipping_orders: Vec<ShippingOrder>,
    order_update_info: OrderUpdateInfo,
}

#[derive(Serialize, Deserialize)]
struct ShopOrder {
    shop: Shop,
    items: Vec<Item>,
    shipping_id: i32,
}

#[derive(Serialize, Deserialize)]
struct Shop {
    shopid: i64,
}

#[derive(Serialize, Deserialize)]
struct Item {
    itemid: i64,
    modelid: i64,
    quantity: i64,
    add_on_deal_id: i32,
    is_add_on_sub_item: bool,
    item_group_id: Option<i64>,
    insurances: Vec<String>,
    channel_exclusive_info: ChannelExclusiveInfo,
    supports_free_returns: bool,
}

#[derive(Serialize, Deserialize)]
struct ChannelExclusiveInfo {
    source_id: i32,
    token: String,
    is_live_stream: bool,
    is_short_video: bool,
}

#[derive(Serialize, Deserialize)]
struct PromotionData {
    use_coins: bool,
    free_shipping_voucher_info: serde_json::Value,
    platform_vouchers: serde_json::Value,
    shop_vouchers: serde_json::Value,
    check_shop_voucher_entrances: bool,
    auto_apply_shop_voucher: bool,
}

#[derive(Serialize, Deserialize)]
struct BuyerInfo {
    kyc_info: Option<String>,
    checkout_email: String,
}

#[derive(Serialize, Deserialize)]
struct TaxInfo {
    tax_id: String,
}

#[derive(Serialize, Deserialize)]
struct ClientEventInfo {
    is_fsv_changed: bool,
    is_platform_voucher_changed: bool,
}

#[derive(Serialize, Deserialize)]
struct AddToCartInfo {}

#[derive(Serialize, Deserialize)]
struct DropshippingInfo {}

#[derive(Serialize, Deserialize)]
struct ShippingOrder {
    sync: bool,
    buyer_address_data: BuyerAddressData,
    selected_logistic_channelid: i64,
    shipping_id: i32,
    shoporder_indexes: Vec<i32>,
    selected_preferred_delivery_time_option_id: i32,
    prescription_info: PrescriptionInfo,
    fulfillment_info: FulfillmentInfo,
}

#[derive(Serialize, Deserialize)]
struct BuyerAddressData {
    addressid: i64,
    address_type: i32,
    tax_address: String,
}

#[derive(Serialize, Deserialize)]
struct PrescriptionInfo {
    images: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct FulfillmentInfo {
    fulfillment_flag: i32,
    fulfillment_source: String,
    managed_by_sbs: bool,
    order_fulfillment_type: i32,
    warehouse_address_id: i64,
    is_from_overseas: bool,
}

#[derive(Serialize, Deserialize)]
struct OrderUpdateInfo {}

#[derive(Serialize)]
struct ShopVoucher {
    shopid: i64,
    promotionid: i64,
    voucher_code: String,
    applied_voucher_code: String,
    invalid_message_code: i32,
    reward_type: i32,
    shipping_order_distributions: Vec<()>, // Jika tipe ini kosong, bisa gunakan Vec<()>, atau tipe lain jika ada data
}

#[derive(Serialize)]
struct PlatformVoucher {
    voucher_code: String,
    promotionid: i64,
}

#[derive(Serialize)]
struct FreeShippingVoucherInfo {
    free_shipping_voucher_id: i64,
    free_shipping_voucher_code: Option<String>,
    disabled_reason: Option<String>,
    disabled_reason_code: i64,
    banner_info: Option<BannerInfo>,  // Optional, will be included only if Some
    required_be_channel_ids: Option<Vec<String>>,  // Optional, will be included only if Some
    required_spm_channels: Option<Vec<String>>,  // Optional, will be included only if Some
}

#[derive(Serialize)]
struct BannerInfo {
    banner_type: i64,
    learn_more_msg: String,
    msg: String,
}

#[derive(Serialize)]
struct FsvSelectionInfo {
    fsv_id: i64,
    selected_shipping_ids: Vec<i64>,
    potentially_applied_shipping_ids: Vec<i64>,
}

#[derive(Serialize, Clone)]
struct ChannelItemOptionInfo {
    option_info: String,
}

#[derive(Serialize, Clone)]
struct SelectedPaymentChannelData {
    page: String,
    removed_vouchers: Vec<String>,
    channel_id: u64,
    version: u64,
    group_id: u64,
    channel_item_option_info: ChannelItemOptionInfo,
    additional_info: serde_json::Value,
}

pub async fn place_order_ng(cookie_content: &str, place_body: &PlaceOrderBody) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
	let mut headers = headers_checkout(&cookie_content);
	let data = crypt::random_hex_string(16);
	headers.insert("af-ac-enc-dat", HeaderValue::from_str(&data).unwrap());
    // Convert struct to JSON
	let body_str = serde_json::to_string(&place_body)?;
	println!("Status: Start Place_Order");
	//println!("{:?}", body_str);
	//println!("Request Headers:\n{:?}", headers);

	let url2 = format!("https://mall.shopee.co.id/api/v4/checkout/place_order");
	println!("{}", url2);
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
        .post(&url2)
        .header("Content-Type", "application/json")
        .headers(headers)
		.body(body_str)
        .version(Version::HTTP_2) 
        .send()
        .await?;
	
	println!("Status: Done Place_Order");
	//println!("Status: {}", response.status());
	let body_resp = response.text().await?;
	let v: serde_json::Value = serde_json::from_str(&body_resp).unwrap();
	println!("Body: {}", body_resp);
	Ok(v)
}

pub async fn get_builder(cookie_content: &str, 
    device_info: serde_json::Value, 
    shop_id_str: &str, item_id_str: &str, 
    addressid_str: &str, quantity_str: &str, 
    chosen_model: &ModelInfo, chosen_payment: &PaymentInfo, 
    chosen_shipping: &ShippingInfo, freeshipping_voucher: Option<Vouchers>, 
    platform_vouchers_target: Option<Vouchers>, shop_vouchers_target: Option<Vouchers>) -> Result<PlaceOrderBody, Box<dyn std::error::Error>> {
	let shop_id = shop_id_str.parse::<i64>().expect("Failed to parse shop_id");
	let addressid = addressid_str.parse::<i64>().expect("Failed to parse addressid");
	let item_id = item_id_str.parse::<i64>().expect("Failed to parse item_id");
	let quantity = quantity_str.parse::<i64>().expect("Failed to parse quantity");
	let channel_id: u64 = chosen_payment.channel_id.parse().expect("Failed to parse channel_id");
	let version: u64 = chosen_payment.version.parse().expect("Failed to parse version");
	let optioninfo: String = chosen_payment.option_info.clone();
	let current_time = Utc::now();

    let mut selected_payment_channel_data = None;
    let mut shop_vouchers = None;
    let mut platform_vouchers = None;
    let mut free_shipping_voucher_info = None;
    let mut fsv_selection_infos = None;
    
    rayon::scope(|s| {
        // Task 1
        s.spawn(|_| {
            selected_payment_channel_data = Some(if chosen_payment.selected_get.is_null() {
                serde_json::to_value(SelectedPaymentChannelData {
                    page: "OPC_PAYMENT_SELECTION".to_string(),
                    removed_vouchers: vec![],
                    channel_id,
                    version,
                    group_id: 0,
                    channel_item_option_info: ChannelItemOptionInfo {
                        option_info: optioninfo.clone(),
                    },
                    additional_info: json!({}),
                })
            } else {
                Ok(chosen_payment.selected_get.clone())
            });
        });
    
        // Task 2
        s.spawn(|_| {
            let shop_vouchers_data = if let Some(shop) = shop_vouchers_target {
                vec![ShopVoucher {
                    shopid: shop_id,
                    promotionid: shop.promotionid,
                    voucher_code: shop.voucher_code.clone(),
                    applied_voucher_code: shop.voucher_code.clone(),
                    invalid_message_code: 0,
                    reward_type: 0,
                    shipping_order_distributions: vec![],
                }]
            } else {
                vec![]
            };
            shop_vouchers = Some(serde_json::to_value(shop_vouchers_data));
        });
    
        // Task 3
        s.spawn(|_| {
            let platform_vouchers_data = if let Some(platform) = platform_vouchers_target {
                vec![PlatformVoucher {
                    voucher_code: platform.voucher_code.clone(),
                    promotionid: platform.promotionid,
                }]
            } else {
                Vec::new()
            };
            platform_vouchers = Some(serde_json::to_value(&platform_vouchers_data));
        });
    
        // Task 4
        s.spawn(|_| {
            let free_shipping_voucher_info_data = if let Some(ref shipping_vc) = freeshipping_voucher {
                FreeShippingVoucherInfo {
                    free_shipping_voucher_id: shipping_vc.promotionid,
                    free_shipping_voucher_code: Some(shipping_vc.voucher_code.clone()),
                    disabled_reason: None,
                    disabled_reason_code: 0,
                    banner_info: Some(BannerInfo {
                        banner_type: 5,
                        learn_more_msg: "".to_string(),
                        msg: "Berhasil mendapatkan Gratis Ongkir".to_string(),
                    }),
                    required_be_channel_ids: Some(vec![]),
                    required_spm_channels: Some(vec![]),
                }
            } else {
                FreeShippingVoucherInfo {
                    free_shipping_voucher_id: 0,
                    free_shipping_voucher_code: None,
                    disabled_reason: Some("".to_string()),
                    disabled_reason_code: 0,
                    banner_info: None,
                    required_be_channel_ids: None,
                    required_spm_channels: None,
                }
            };
            free_shipping_voucher_info = Some(serde_json::to_value(&free_shipping_voucher_info_data));
        });
    
        // Task 5
        s.spawn(|_| {
            let fsv_selection_infos_data = if let Some(shipping_vca) = freeshipping_voucher.clone() {
                vec![FsvSelectionInfo {
                    fsv_id: shipping_vca.promotionid,
                    selected_shipping_ids: vec![1],
                    potentially_applied_shipping_ids: vec![1],
                }]
            } else {
                vec![]
            };
            fsv_selection_infos = Some(serde_json::to_value(&fsv_selection_infos_data));
        });
    });
    
    // Unwrap semua hasil setelah paralel selesai
    let selected_payment_channel_data = selected_payment_channel_data.unwrap()?;
    let shop_vouchers = shop_vouchers.unwrap()?;
    let platform_vouchers = platform_vouchers.unwrap()?;
    let free_shipping_voucher_info = free_shipping_voucher_info.unwrap()?;
    let fsv_selection_infos = fsv_selection_infos.unwrap()?;

    let body_json = BodyJson {
        timestamp: current_time.timestamp(),
        shoporders: vec![ShopOrder {
            shop: Shop {
                shopid: shop_id,
            },
            items: vec![Item {
                itemid: item_id,
                modelid: chosen_model.modelid,
                quantity: quantity,
                add_on_deal_id: 0,
                is_add_on_sub_item: false,
                item_group_id: None,
                insurances: vec![],
                channel_exclusive_info: ChannelExclusiveInfo {
                    source_id: 0,
                    token: "".to_string(),
                    is_live_stream: false,
                    is_short_video: false,
                },
                supports_free_returns: false,
            }],
            shipping_id: 1,
        }],
        selected_payment_channel_data,
        promotion_data: PromotionData {
            use_coins: true,
            free_shipping_voucher_info,
            platform_vouchers,
            shop_vouchers,
            check_shop_voucher_entrances: true,
            auto_apply_shop_voucher: false,
        },
        fsv_selection_infos,
        device_info: device_info.clone(),
        buyer_info: BuyerInfo {
            kyc_info: None,
            checkout_email: "".to_string(),
        },
        cart_type: 1,
        client_id: 5,
        tax_info: TaxInfo { tax_id: "".to_string() },
        client_event_info: ClientEventInfo { is_fsv_changed: false, is_platform_voucher_changed: false },
        add_to_cart_info: AddToCartInfo {},
        _cft: vec![469696383],
        dropshipping_info: DropshippingInfo {},
        shipping_orders: vec![ShippingOrder {
            sync: true,
            buyer_address_data: BuyerAddressData {
                addressid: addressid,
                address_type: 0,
                tax_address: "".to_string(),
            },
            selected_logistic_channelid: chosen_shipping.channelid,
            shipping_id: 1,
            shoporder_indexes: vec![0],
            selected_preferred_delivery_time_option_id: 0,
            prescription_info: PrescriptionInfo {
                images: vec![],
            },
            fulfillment_info: FulfillmentInfo {
                fulfillment_flag: 18,
                fulfillment_source: "IDE".to_string(),
                managed_by_sbs: false,
                order_fulfillment_type: 1,
                warehouse_address_id: 0,
                is_from_overseas: false,
            },
        }],
        order_update_info: OrderUpdateInfo {},
    };
	let mut headers = headers_checkout(&cookie_content);
	let data = crypt::random_hex_string(16);
	let current_time = Utc::now();
	headers.insert("af-ac-enc-dat", HeaderValue::from_str(&data).unwrap());
    // Convert struct to JSON
    let body_str = serde_json::to_string(&body_json)?;
	println!("Status: Start Checkout");
	//println!("{:?}", body_str);
	//println!("Request Headers:\n{:?}", headers);

	let url2 = format!("https://mall.shopee.co.id/api/v4/checkout/get");
	println!("{}", url2);
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
        .post(&url2)
        .header("Content-Type", "application/json")
        .headers(headers)
		.body(body_str)
        .version(Version::HTTP_2) 
        .send()
        .await?;

	println!("Status: Done Checkout");
    // Handle response as needed
	//println!("Request Headers:\n{:?}", headers);
	println!("Status: {}", response.status());
	let body_resp = response.text().await?;
	//println!("Body: {}", body_resp);
    let v: serde_json::Value = serde_json::from_str(&body_resp).unwrap();
	let keys = vec![
        "checkout_price_data",
        "order_update_info",
		"dropshipping_info",
        "promotion_data",
		"selected_payment_channel_data",
		"shoporders",
        "shipping_orders",
		"display_meta_data",
		"fsv_selection_infos",
		"buyer_info",
        "client_event_info",
		"buyer_txn_fee_info",
		"disabled_checkout_info",
		"buyer_service_fee_info",
		"iof_info",
    ];
	let results: HashMap<String, Option<Value>> = keys.par_iter()
		.map(|&key| {
			// Mengambil nilai untuk setiap kunci, jika tidak ditemukan return None
			(key.to_string(), v.get(key).cloned())
		})
		.collect();

    let selected_payment_channel_data_place_order = if chosen_payment.place_order.is_null(){
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
	let place_order = PlaceOrderBody {
		client_id: 5,
		cart_type: 1,
		timestamp: current_time.timestamp(),
		checkout_price_data: results.get("checkout_price_data").cloned().unwrap_or(None).unwrap_or(Value::Null),
		order_update_info: results.get("order_update_info").cloned().unwrap_or(None).unwrap_or(Value::Null),
		dropshipping_info: results.get("dropshipping_info").cloned().unwrap_or(None).unwrap_or(Value::Null),
		promotion_data: results.get("promotion_data").cloned().unwrap_or(None).unwrap_or(Value::Null),
		selected_payment_channel_data: selected_payment_channel_data_place_order,
		shoporders: results.get("shoporders").cloned().unwrap_or(None).unwrap_or(Value::Null),
		shipping_orders: results.get("shipping_orders").cloned().unwrap_or(None).unwrap_or(Value::Null),
		display_meta_data: results.get("display_meta_data").cloned().unwrap_or(None).unwrap_or(Value::Null),
		fsv_selection_infos: results.get("fsv_selection_infos").cloned().unwrap_or(None).unwrap_or(Value::Null),
		buyer_info: results.get("buyer_info").cloned().unwrap_or(None).unwrap_or(Value::Null),
		client_event_info: results.get("client_event_info").cloned().unwrap_or(None).unwrap_or(Value::Null),
		buyer_txn_fee_info: results.get("buyer_txn_fee_info").cloned().unwrap_or(None).unwrap_or(Value::Null),
		disabled_checkout_info: results.get("disabled_checkout_info").cloned().unwrap_or(None).unwrap_or(Value::Null),
		buyer_service_fee_info: results.get("buyer_service_fee_info").cloned().unwrap_or(None).unwrap_or(Value::Null),
		iof_info: results.get("iof_info").cloned().unwrap_or(None).unwrap_or(Value::Null),
		add_to_cart_info: HashMap::new(), // Empty HashMap for now
		ignored_errors: vec![0],
		can_checkout: true,
		ignore_warnings: false,
		captcha_id: "".to_string(),
		captcha_version: 1,
		captcha_signature: "".to_string(),
		extra_data: ExtraData {
			snack_click_id: None,
		},
		device_info,
		device_type: "mobile".to_string(),
		_cft: vec![4227792767, 24191],
	};

	Ok(place_order)
}