use rquest as reqwest;
use reqwest::Version;
use reqwest::header::{HeaderValue, HeaderMap};
use chrono::{Utc, Timelike};
use anyhow::Result;
use serde_json::{Value};
use std::collections::HashMap;
use rayon::prelude::*;
use dashmap::DashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

use crate::prepare::{ModelInfo, ShippingInfo, PaymentInfo, AddressInfo};
use crate::voucher::Vouchers;
use crate::crypt::{self, DeviceInfo};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlaceOrderBody {
    client_id: i32,
    cart_type: i32,
    timestamp: i64,
    pub checkout_price_data: Option<Value>,
    order_update_info: OrderUpdateInfo,
    dropshipping_info: DropshippingInfo,
    promotion_data: PromotionData,
    selected_payment_channel_data: Option<Value>,
    pub shoporders: Vec<ShopOrder>,
    pub shipping_orders: Vec<ShippingOrder>,
    display_meta_data: Option<Value>,
    fsv_selection_infos: Vec<Option<FsvSelectionInfo>>,
    buyer_info: BuyerInfo,
    client_event_info: ClientEventInfo,
    captcha_id: String,
    buyer_txn_fee_info: Option<Value>,
    disabled_checkout_info: Option<Value>,
    can_checkout: bool,
    buyer_service_fee_info: Option<Value>,
    iof_info: Option<Value>,
    add_to_cart_info: AddToCartInfo,
    ignored_errors: Vec<i64>,
    ignore_warnings: bool,
    captcha_version: i64,
    captcha_signature: String,
    extra_data: ExtraData,
    checkout_session_id: String,
    device_info: DeviceInfo,
    device_type: String,
    _cft: Vec<i64>,
}

impl PlaceOrderBody {
    fn insert(&mut self, key: &str, value: Option<Value>) {
        match key {
            "checkout_price_data" => self.checkout_price_data = value,
            "selected_payment_channel_data" => self.selected_payment_channel_data = value,
            "display_meta_data" => self.display_meta_data = value,
            "buyer_txn_fee_info" => self.buyer_txn_fee_info = value,
            "disabled_checkout_info" => self.disabled_checkout_info = value,
            "buyer_service_fee_info" => self.buyer_service_fee_info = value,
            "iof_info" => self.iof_info = value,
            _ => {}
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ExtraData {
    snack_click_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetBodyJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<i64>,
    shoporders: Vec<ShopOrder>,
    selected_payment_channel_data: serde_json::Value,
    promotion_data: PromotionData,
    fsv_selection_infos: Vec<Option<FsvSelectionInfo>>,
    device_info: DeviceInfo,
    buyer_info: BuyerInfo,
    cart_type: i32,
    client_id: i32,
    tax_info: TaxInfo,
    client_event_info: ClientEventInfo,
    add_to_cart_info: AddToCartInfo,
    _cft: Vec<i64>,
    checkout_session_id: String,
    dropshipping_info: DropshippingInfo,
    shipping_orders: Vec<ShippingOrder>,
    order_update_info: OrderUpdateInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShopOrder {
    pub shop: Shop,
    pub items: Vec<Item>,
    pub shipping_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Shop {
    pub shopid: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub itemid: i64,
    pub modelid: i64,
    pub quantity: i32,
    pub add_on_deal_id: i64,
    pub is_add_on_sub_item: bool,
    pub item_group_id: Option<String>,
    pub insurances: Vec<serde_json::Value>,
    pub channel_exclusive_info: ChannelExclusiveInfo,
    pub supports_free_returns: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelExclusiveInfo {
    pub source_id: i32,
    pub token: String,
    pub is_live_stream: bool,
    pub is_short_video: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PromotionData {
    use_coins: bool,
    free_shipping_voucher_info: FreeShippingVoucherInfo,
    platform_vouchers: Vec<Option<PlatformVoucher>>,
    shop_vouchers: Vec<Option<ShopVoucher>>,
    check_shop_voucher_entrances: bool,
    auto_apply_shop_voucher: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BuyerInfo {
    kyc_info: Option<String>,
    checkout_email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TaxInfo {
    tax_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ClientEventInfo {
    is_fsv_changed: bool,
    is_platform_voucher_changed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AddToCartInfo {}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DropshippingInfo {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShippingOrder {
    #[serde(default = "default_sync")]
    pub sync: bool,
    pub buyer_address_data: BuyerAddressData,
    pub selected_logistic_channelid: i64,
    pub shipping_id: i32,
    pub shoporder_indexes: Vec<i32>,
    #[serde(default = "selected_preferred_delivery_time_option_id")]
    pub selected_preferred_delivery_time_option_id: i32,
    pub prescription_info: PrescriptionInfo,
    pub fulfillment_info: FulfillmentInfo,
    pub selected_logistic_channel_data: SelectedLogisticChannelData,
}
fn selected_preferred_delivery_time_option_id() -> i32 {
    0
}
fn default_sync() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuyerAddressData {
    pub addressid: i64,
    pub address_type: i32,
    pub tax_address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrescriptionInfo {
    pub images: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FulfillmentInfo {
    pub fulfillment_flag: i32,
    pub fulfillment_source: String,
    pub managed_by_sbs: bool,
    pub order_fulfillment_type: i32,
    pub warehouse_address_id: i64,
    pub is_from_overseas: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SelectedLogisticChannelData {
    pub support_advance_booking: bool,
    pub selected_from: i32,
    pub fulfillment_shipping_order_channel_data: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OrderUpdateInfo {}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ShopVoucher {
    shopid: i64,
    promotionid: i64,
    voucher_code: String,
    applied_voucher_code: String,
    invalid_message_code: i32,
    reward_type: i32,
    shipping_order_distributions: Vec<()>, // Jika tipe ini kosong, bisa gunakan Vec<()>, atau tipe lain jika ada data
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PlatformVoucher {
    voucher_code: String,
    promotionid: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FreeShippingVoucherInfo {
    free_shipping_voucher_id: i64,
    free_shipping_voucher_code: Option<String>,
    disabled_reason: Option<String>,
    disabled_reason_code: i64,
    banner_info: Option<BannerInfo>,  // Optional, will be included only if Some
    required_be_channel_ids: Option<Vec<String>>,  // Optional, will be included only if Some
    required_spm_channels: Option<Vec<String>>,  // Optional, will be included only if Some
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BannerInfo {
    banner_type: i64,
    learn_more_msg: String,
    msg: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FsvSelectionInfo {
    fsv_id: i64,
    selected_shipping_ids: Vec<i64>,
    potentially_applied_shipping_ids: Vec<i64>,
}

#[derive(Serialize, Clone)]
pub struct ChannelItemOptionInfo {
    pub option_info: String,
}

#[derive(Serialize, Clone)]
pub struct SelectedGet {
    pub page: String,
    pub removed_vouchers: Vec<String>,
    pub channel_id: i64,
    pub version: i64,
    pub group_id: u64,
    pub channel_item_option_info: ChannelItemOptionInfo,
    pub additional_info: serde_json::Value,
}
#[derive(Serialize, Clone)]
pub struct SelectedPlaceOrder {
    pub channel_id: i64,
    pub channel_item_option_info: ChannelItemOptionInfo,
    pub version: i64,
}
impl From<GetBodyJson> for PlaceOrderBody {
    fn from(get: GetBodyJson) -> Self {
        PlaceOrderBody {
            client_id: get.client_id,
            cart_type: get.cart_type,
            timestamp: get.timestamp.unwrap_or(0),
            checkout_price_data: None,
            order_update_info: get.order_update_info,
            dropshipping_info: get.dropshipping_info,
            promotion_data: get.promotion_data,
            selected_payment_channel_data: Some(get.selected_payment_channel_data), // ubah tipe di struct jika perlu
            shoporders: get.shoporders,
            shipping_orders: get.shipping_orders,
            display_meta_data: None,
            fsv_selection_infos: get.fsv_selection_infos,
            buyer_info: get.buyer_info,
            client_event_info: get.client_event_info,
            captcha_id: String::new(),
            buyer_txn_fee_info: None,
            disabled_checkout_info: None,
            can_checkout: true,
            buyer_service_fee_info: None,
            iof_info: None,
            add_to_cart_info: get.add_to_cart_info,
            ignored_errors: vec![],
            ignore_warnings: false,
            captcha_version: 1,
            captcha_signature: String::new(),
            extra_data:  ExtraData {
                snack_click_id: None,
            },
            checkout_session_id: get.checkout_session_id,
            device_info: get.device_info,
            device_type: "mobile".into(),
            _cft: get._cft,
        }
    }
}

pub async fn place_order_ng(client: Arc<reqwest::Client>, base_headers: Arc<HeaderMap>, place_body: &PlaceOrderBody) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut headers = (*base_headers).clone();
    headers.insert("af-ac-enc-dat", HeaderValue::from_str(&crypt::random_hex_string(16)).unwrap());
    // Convert struct to JSON
	//let body_str = serde_json::to_string(&place_body)?;
	//println!("Status: Start Place_Order");
	//println!("{:?}", body_str);
	//println!("Request Headers:\n{:?}", headers);

	let url2 = format!("https://mall.shopee.co.id/api/v4/checkout/place_order");
	println!("{}", url2);
    // Buat permintaan HTTP POST
    let response = (*client)
        .post(&url2)
        .headers(headers)
		.json(&place_body)
        .version(Version::HTTP_2) 
        .send()
        .await?;
	
	//println!("Status: Done Place_Order");
	//println!("Status: {}", response.status());
    let v: Value = response.json().await?;
	println!("Body: {}", v);
	Ok(v)
}
pub async fn get_body_builder(device_info: &DeviceInfo, 
    chosen_payment: &PaymentInfo, 
    freeshipping_voucher: Arc<Option<Vouchers>>, 
    platform_vouchers_target: Arc<Option<Vouchers>>, shop_vouchers_target: Arc<Option<Vouchers>>, use_coins: bool,
    place_order: &mut PlaceOrderBody) -> Result<(GetBodyJson, PlaceOrderBody), Box<dyn std::error::Error>> {
	let current_time = Utc::now();
    let timestamp_millis = current_time.timestamp_millis();
    let timestamp_specific = format!("{:.16}", current_time.nanosecond() as f64 / 1_000_000_000.0);
    let checkout_session_id = format!(
        "{}:{}:{}{}",
        device_info.device_id, timestamp_millis, timestamp_millis, timestamp_specific
    );

    let free_shipping_thread = {
        let freeshipping_voucher = Arc::clone(&freeshipping_voucher);
        tokio::spawn(async move{
            match &*freeshipping_voucher {
                Some(shipping_vc) => FreeShippingVoucherInfo {
                    free_shipping_voucher_id: shipping_vc.promotionid,
                    free_shipping_voucher_code: Some(shipping_vc.voucher_code.clone()),
                    disabled_reason: None,
                    disabled_reason_code: 0,
                    banner_info: Some(BannerInfo {
                        banner_type: 5,
                        learn_more_msg: "".to_owned(),
                        msg: "Berhasil mendapatkan Gratis Ongkir".to_owned(),
                    }),
                    required_be_channel_ids: Some(vec![]),
                    required_spm_channels: Some(vec![]),
                },
                None => FreeShippingVoucherInfo {
                    free_shipping_voucher_id: 0,
                    free_shipping_voucher_code: None,
                    disabled_reason: Some("".to_owned()),
                    disabled_reason_code: 0,
                    banner_info: None,
                    required_be_channel_ids: None,
                    required_spm_channels: None,
                },
            }
        })
    };

    let shop_vouchers_thread = {
        let shop_vouchers_target = Arc::clone(&shop_vouchers_target);
        tokio::spawn(async move{
            match &*shop_vouchers_target {
                Some(shop) => vec![Some(ShopVoucher {
                    shopid: shop.shop_id.unwrap_or(0),
                    promotionid: shop.promotionid,
                    voucher_code: shop.voucher_code.clone(),
                    applied_voucher_code: shop.voucher_code.clone(),
                    invalid_message_code: 0,
                    reward_type: 0,
                    shipping_order_distributions: vec![],
                })],
                None => vec![],
            }
        })
    };

    let platform_vouchers_thread = {
        let platform_vouchers_target = Arc::clone(&platform_vouchers_target);
        tokio::spawn(async move{
            match &*platform_vouchers_target {
                Some(platform) => vec![Some(PlatformVoucher {
                    voucher_code: platform.voucher_code.clone(),
                    promotionid: platform.promotionid,
                })],
                None => vec![],
            }
        })
    };

    let fsv_selection_thread = {
        let freeshipping_voucher = Arc::clone(&freeshipping_voucher);
        tokio::spawn(async move{
            match &*freeshipping_voucher {
                Some(shipping_vca) => vec![Some(FsvSelectionInfo {
                    fsv_id: shipping_vca.promotionid,
                    selected_shipping_ids: vec![1],
                    potentially_applied_shipping_ids: vec![1],
                })],
                None => vec![],
            }
        })
    };

    let free_shipping_voucher_info = free_shipping_thread.await?;
    let shop_vouchers = shop_vouchers_thread.await?;
    let platform_vouchers = platform_vouchers_thread.await?;
    let fsv_selection_infos = fsv_selection_thread.await?;

    let body_json = GetBodyJson {
        timestamp: Some(current_time.timestamp()),
        shoporders: place_order.shoporders.to_vec(),
        selected_payment_channel_data: chosen_payment.selected_get.clone(),
        promotion_data: PromotionData {
            use_coins,
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
            checkout_email: "".to_owned(),
        },
        cart_type: 1,
        client_id: 5,
        tax_info: TaxInfo { tax_id: "".to_owned() },
        client_event_info: ClientEventInfo { is_fsv_changed: false, is_platform_voucher_changed: false },
        add_to_cart_info: AddToCartInfo {},
        _cft: vec![4227792767, 36961919],
        checkout_session_id,
        dropshipping_info: DropshippingInfo {},
        shipping_orders: place_order.shipping_orders.to_vec(),
        order_update_info: OrderUpdateInfo {},
    };
    *place_order = body_json.clone().into();
    Ok((body_json, (*place_order).clone()))
}
pub async fn get_ng(client: Arc<reqwest::Client>, base_headers: Arc<HeaderMap>, body_json: &GetBodyJson, chosen_payment: &PaymentInfo, mut place_order_body: PlaceOrderBody, adjusted_max_price: Option<i64>) -> Result<PlaceOrderBody, Box<dyn std::error::Error>> {
    let mut headers = (*base_headers).clone();
    headers.insert("af-ac-enc-dat", HeaderValue::from_str(&crypt::random_hex_string(16)).unwrap());
    loop{
        //println!("Status: Start Checkout");
        //let t1 = std::time::Instant::now();
        let url2 = format!("https://mall.shopee.co.id/api/v4/checkout/get");
        //let body_str = serde_json::to_string(&body_json)?;
        //println!("{}", body_str);
        println!("{}", url2);

        let response = (*client)
            .post(&url2)
            .headers(headers.clone())
            .version(Version::HTTP_2)
            .json(&body_json)
            .send()
            .await?;
        //println!("[{:?}] Setelah .send()", t1.elapsed());
        //println!("Status: Done Checkout");
        let status = response.status();
        if status == reqwest::StatusCode::OK {
            let v: Value = response.json().await?;
            //println!("[{:?}] setelah .json()", t1.elapsed());
            let v = Arc::new(v);
            if let Some(limit) = adjusted_max_price {
                let price_opt = v.get("checkout_price_data")
                    .and_then(|d| d.get("merchandise_subtotal"))
                    .and_then(|v| v.as_i64());

                match price_opt {
                    Some(price) if price > limit => {
                        println!(
                            "[{}]Harga terlalu tinggi ({} > {}). Coba ulang...",
                            chrono::Local::now().format("%H:%M:%S.%3f"),
                            price, limit
                        );
                        // Retry
                        continue;
                    }
                    Some(price) => {
                        println!(
                            "[{}]Harga cocok ({} <= {}).",
                            chrono::Local::now().format("%H:%M:%S.%3f"),
                            price, limit
                        );
                        // Lanjut proses
                    }
                    None => {
                        println!(
                            "[{}]Gagal membaca merchandise_subtotal, ulangi...",
                            chrono::Local::now().format("%H:%M:%S.%3f")
                        );
                        continue; // Retry
                    }
                }
            }

            let keys = [
                "checkout_price_data",
                "display_meta_data",
                "buyer_txn_fee_info",
                "disabled_checkout_info",
                "buyer_service_fee_info",
                "iof_info",
            ];

            let dash_body: Arc<DashMap<String, Option<Value>>> = Arc::new(DashMap::new());

            keys.par_iter().for_each(|&key| {
                let value = v.get(key).cloned();
                dash_body.insert(key.to_string(), value);
            });

            dash_body.insert(
                "selected_payment_channel_data".to_string(),
                Some(chosen_payment.place_order.clone()),
            );

            // Langkah 5: Transfer nilai dari DashMap ke PlaceOrderBody
            for entry in dash_body.iter() {
                match entry.key().as_str() {
                    "checkout_price_data" => place_order_body.checkout_price_data = entry.value().clone(),
                    "display_meta_data" => place_order_body.display_meta_data = entry.value().clone(),
                    "buyer_txn_fee_info" => place_order_body.buyer_txn_fee_info = entry.value().clone(),
                    "disabled_checkout_info" => place_order_body.disabled_checkout_info = entry.value().clone(),
                    "buyer_service_fee_info" => place_order_body.buyer_service_fee_info = entry.value().clone(),
                    "iof_info" => place_order_body.iof_info = entry.value().clone(),
                    "selected_payment_channel_data" => place_order_body.selected_payment_channel_data = entry.value().clone(),
                    _ => {}
                }
            }
            return Ok(place_order_body)
        } else {
            eprintln!("Failed to get checkout data: {}", status);
            return Err("Failed to get checkout data".into());
        };
    }
}
pub async fn get_builder(client: Arc<reqwest::Client>, base_headers: Arc<HeaderMap>,
    device_info: &DeviceInfo, 
    address_info: &AddressInfo,
    chosen_model: &[ModelInfo], chosen_payment: &PaymentInfo, 
    chosen_shipping: &ShippingInfo, freeshipping_voucher: &Option<Vouchers>, 
    platform_vouchers_target: &Option<Vouchers>, shop_vouchers_target: &Option<Vouchers>, use_coins: bool) -> Result<PlaceOrderBody, Box<dyn std::error::Error>> {
    let shoporders = multi_product(&chosen_model);
    let current_time = Utc::now();
    let timestamp_millis = current_time.timestamp_millis();
    let timestamp_specific = format!("{:.16}", current_time.nanosecond() as f64 / 1_000_000_000.0);
    let checkout_session_id = format!(
        "{}:{}:{}{}",
        device_info.device_id, timestamp_millis, timestamp_millis, timestamp_specific
    );
    let freeshipping_voucher_clone = freeshipping_voucher.clone();

    let free_shipping_thread = tokio::spawn(async move{
        match freeshipping_voucher_clone {
            Some(shipping_vc) => FreeShippingVoucherInfo {
                free_shipping_voucher_id: shipping_vc.promotionid,
                free_shipping_voucher_code: Some(shipping_vc.voucher_code.clone()),
                disabled_reason: None,
                disabled_reason_code: 0,
                banner_info: Some(BannerInfo {
                    banner_type: 5,
                    learn_more_msg: "".to_owned(),
                    msg: "Berhasil mendapatkan Gratis Ongkir".to_owned(),
                }),
                required_be_channel_ids: Some(vec![]),
                required_spm_channels: Some(vec![]),
            },
            None => FreeShippingVoucherInfo {
                free_shipping_voucher_id: 0,
                free_shipping_voucher_code: None,
                disabled_reason: Some("".to_owned()),
                disabled_reason_code: 0,
                banner_info: None,
                required_be_channel_ids: None,
                required_spm_channels: None,
            },
        }
    });

    let shop_vouchers_target_clone = shop_vouchers_target.clone();
    let shop_vouchers_thread = tokio::spawn(async move{
        match shop_vouchers_target_clone {
            Some(shop) => vec![Some(ShopVoucher {
                shopid: shop.shop_id.unwrap_or(0),
                promotionid: shop.promotionid,
                voucher_code: shop.voucher_code.clone(),
                applied_voucher_code: shop.voucher_code.clone(),
                invalid_message_code: 0,
                reward_type: 0,
                shipping_order_distributions: vec![],
            })],
            None => vec![],
        }
    });

    let platform_vouchers_target_clone = platform_vouchers_target.clone();
    let platform_vouchers_thread = tokio::spawn(async move{
        match platform_vouchers_target_clone {
            Some(platform) => vec![Some(PlatformVoucher {
                voucher_code: platform.voucher_code.clone(),
                promotionid: platform.promotionid,
            })],
            None => vec![],
        }
    });

    let freeshipping_voucher_clone = freeshipping_voucher.clone();
    let fsv_selection_thread = tokio::spawn(async move{
        match freeshipping_voucher_clone {
            Some(shipping_vca) => vec![Some(FsvSelectionInfo {
                fsv_id: shipping_vca.promotionid,
                selected_shipping_ids: vec![1],
                potentially_applied_shipping_ids: vec![1],
            })],
            None => vec![],
        }
    });

    let free_shipping_voucher_info = free_shipping_thread.await?;
    let shop_vouchers = shop_vouchers_thread.await?;
    let platform_vouchers = platform_vouchers_thread.await?;
    let fsv_selection_infos = fsv_selection_thread.await?;
    let shipping_orders = if chosen_shipping.channelid == 0 {
        vec![]
    } else {
        vec![ShippingOrder {
            sync: true,
            buyer_address_data: BuyerAddressData {
                addressid: address_info.id,
                address_type: 0,
                tax_address: "".to_owned(),
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
                fulfillment_source: "IDE".to_owned(),
                managed_by_sbs: false,
                order_fulfillment_type: 1,
                warehouse_address_id: 0,
                is_from_overseas: false,
            },
            selected_logistic_channel_data: SelectedLogisticChannelData{
                support_advance_booking: false,
                selected_from: 1,
                fulfillment_shipping_order_channel_data: None,
            },
        }]
    };

    let body_json = GetBodyJson {
        timestamp: None,
        shoporders,
        selected_payment_channel_data: chosen_payment.selected_get.clone(),
        promotion_data: PromotionData {
            use_coins,
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
            checkout_email: "".to_owned(),
        },
        cart_type: 1,
        client_id: 5,
        tax_info: TaxInfo { tax_id: "".to_owned() },
        client_event_info: ClientEventInfo { is_fsv_changed: false, is_platform_voucher_changed: false },
        add_to_cart_info: AddToCartInfo {},
        _cft: vec![4227792767, 36961919],
        checkout_session_id,
        dropshipping_info: DropshippingInfo {},
        shipping_orders,
        order_update_info: OrderUpdateInfo {},
    };
	let mut headers = (*base_headers).clone();
    headers.insert("af-ac-enc-dat", HeaderValue::from_str(&crypt::random_hex_string(16)).unwrap());
	println!("Status: Start Checkout");

	let url2 = format!("https://mall.shopee.co.id/api/v4/checkout/get");
	println!("{}", url2);
    let response = client
        .post(&url2)
        .headers(headers)
		.json(&body_json)
        .version(Version::HTTP_2) 
        .send()
        .await?;

	println!("Status: Done Checkout");
	println!("Status: {}", response.status());
	let v: Value = response.json().await?;
    let keys = vec![
        "checkout_price_data",
        "display_meta_data",
        "buyer_txn_fee_info",
        "disabled_checkout_info",
        "buyer_service_fee_info",
        "iof_info",
    ];

    //println!("Copying shoporders");
    let mut place_order_body: PlaceOrderBody = body_json.clone().into();
    if let Some(shoporders) = v.get("shoporders") {
        place_order_body.shoporders = serde_json::from_value(shoporders.clone())?;
    }
    //println!("Copying shipping_orders");
    if let Some(shipping_orders) = v.get("shipping_orders") {
        place_order_body.shipping_orders = serde_json::from_value(shipping_orders.clone())?;
    }
    place_order_body.selected_payment_channel_data = Some(chosen_payment.place_order.clone()); 
    //println!("Copying essential keys");
    let handles = keys.iter().map(|key| {
        let v_clone = v.clone();
        let key = key.to_string();
        tokio::spawn(async move {
            let value = v_clone.get(&key).cloned();
            (key, value)
        })
    }).collect::<Vec<_>>();
    let results = futures::future::join_all(handles).await;
    for result in results {
        match result {
            Ok((key, value)) => {
                place_order_body.insert(&key, value);
            }
            Err(e) => {
                eprintln!("Error joining thread: {:?}", e);
            }
        }
    }

    place_order_body.insert("selected_payment_channel_data", Some(chosen_payment.place_order.clone()));

	Ok(place_order_body)
}

pub fn force_deselect_insurance(shoporders: &mut [ShopOrder]) {
    for item in shoporders.iter_mut().flat_map(|s| &mut s.items) {
        for insurance in &mut item.insurances {
            if let Some(obj) = insurance.as_object_mut() {
                obj.insert("selected".into(), serde_json::Value::Bool(false));
            }
        }
    }
}

pub fn multi_product(products: &[ModelInfo]) -> Vec<ShopOrder> {
    let mut grouped: HashMap<i64, Vec<&ModelInfo>> = HashMap::new();

    // Kelompokkan berdasarkan shop_id
    for product in products {
        grouped.entry(product.shop_id).or_default().push(product);
    }

    // Sortir shop_id biar urut
    let mut shop_ids: Vec<_> = grouped.keys().cloned().collect();
    shop_ids.sort_unstable();

    // Buat ShopOrder dengan shipping_id urut berdasarkan index
    shop_ids
        .into_iter()
        .enumerate()
        .map(|(idx, shop_id)| {
            let items = grouped.remove(&shop_id).unwrap_or_default();

            ShopOrder {
                shop: Shop { shopid: shop_id },
                items: items
                    .iter()
                    .map(|p| Item {
                        itemid: p.item_id,
                        modelid: p.modelid,
                        quantity: p.quantity,
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
                    })
                    .collect(),
                shipping_id: (idx + 1) as i32,
            }
        })
        .collect()
}
//pub fn precognition_value()