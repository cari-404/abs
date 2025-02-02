use rquest as reqwest;
use reqwest::tls::Impersonate;
use reqwest::{ClientBuilder, Version};
use reqwest::header::HeaderValue;
use chrono::{Utc};
use anyhow::Result;
use serde_json::{Value};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::prepare::{CookieData, ModelInfo, ShippingInfo, PaymentInfo, ProductInfo, AddressInfo};
use crate::voucher::Vouchers;
use crate::task::headers_checkout;
use crate::crypt::{self, DeviceInfo};

#[derive(Serialize, Deserialize, Debug)]
pub struct PlaceOrderBody {
    client_id: i64,
    cart_type: i64,
    timestamp: i64,
    checkout_price_data: Option<Value>, // Define more specific types if known
    order_update_info: Option<Value>,
    dropshipping_info: Option<Value>,
    promotion_data: Option<Value>,
    selected_payment_channel_data: Option<Value>,
    shoporders: Option<Value>,
    shipping_orders: Option<Value>,
    display_meta_data: Option<Value>,
    fsv_selection_infos: Option<Value>,
    buyer_info: Option<Value>,
    client_event_info: Option<Value>,
    captcha_id: String,
    buyer_txn_fee_info: Option<Value>,
    disabled_checkout_info: Option<Value>,
    can_checkout: bool,
    buyer_service_fee_info: Option<Value>,
    iof_info: Option<Value>,
    add_to_cart_info: HashMap<String, serde_json::Value>,
    ignored_errors: Vec<i64>,
    ignore_warnings: bool,
    captcha_version: i64,
    captcha_signature: String,
    extra_data: ExtraData,
    device_info: DeviceInfo,
    device_type: String,
    _cft: Vec<i64>,
}

impl PlaceOrderBody {
    fn new(device_info: &DeviceInfo) -> Self {
        let current_time = Utc::now();
        PlaceOrderBody {
            client_id: 5,
            cart_type: 1,
            timestamp: current_time.timestamp(),
            checkout_price_data: None,
            order_update_info: None,
            dropshipping_info: None,
            promotion_data: None,
            selected_payment_channel_data: None,
            shoporders: None,
            shipping_orders: None,
            display_meta_data: None,
            fsv_selection_infos: None,
            buyer_info: None,
            client_event_info: None,
            buyer_txn_fee_info: None,
            disabled_checkout_info: None,
            buyer_service_fee_info: None,
            iof_info: None,
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
            device_info: device_info.clone(),
            device_type: "mobile".to_string(),
            _cft: vec![4227792767, 24191],
        }
    }

    fn insert(&mut self, key: &str, value: Option<Value>) {
        match key {
            "checkout_price_data" => self.checkout_price_data = value,
            "order_update_info" => self.order_update_info = value,
            "dropshipping_info" => self.dropshipping_info = value,
            "promotion_data" => self.promotion_data = value,
            "selected_payment_channel_data" => self.selected_payment_channel_data = value,
            "shoporders" => self.shoporders = value,
            "shipping_orders" => self.shipping_orders = value,
            "display_meta_data" => self.display_meta_data = value,
            "fsv_selection_infos" => self.fsv_selection_infos = value,
            "buyer_info" => self.buyer_info = value,
            "client_event_info" => self.client_event_info = value,
            "buyer_txn_fee_info" => self.buyer_txn_fee_info = value,
            "disabled_checkout_info" => self.disabled_checkout_info = value,
            "buyer_service_fee_info" => self.buyer_service_fee_info = value,
            "iof_info" => self.iof_info = value,
            _ => {}
        }
    }
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
    fsv_selection_infos: Vec<Option<FsvSelectionInfo>>,
    device_info: DeviceInfo,
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
    quantity: i32,
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
    free_shipping_voucher_info: FreeShippingVoucherInfo,
    platform_vouchers: Vec<Option<PlatformVoucher>>,
    shop_vouchers: Vec<Option<ShopVoucher>>,
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

#[derive(Serialize, Deserialize)]
struct ShopVoucher {
    shopid: i64,
    promotionid: i64,
    voucher_code: String,
    applied_voucher_code: String,
    invalid_message_code: i32,
    reward_type: i32,
    shipping_order_distributions: Vec<()>, // Jika tipe ini kosong, bisa gunakan Vec<()>, atau tipe lain jika ada data
}

#[derive(Serialize, Deserialize)]
struct PlatformVoucher {
    voucher_code: String,
    promotionid: i64,
}

#[derive(Serialize, Deserialize)]
struct FreeShippingVoucherInfo {
    free_shipping_voucher_id: i64,
    free_shipping_voucher_code: Option<String>,
    disabled_reason: Option<String>,
    disabled_reason_code: i64,
    banner_info: Option<BannerInfo>,  // Optional, will be included only if Some
    required_be_channel_ids: Option<Vec<String>>,  // Optional, will be included only if Some
    required_spm_channels: Option<Vec<String>>,  // Optional, will be included only if Some
}

#[derive(Serialize, Deserialize)]
struct BannerInfo {
    banner_type: i64,
    learn_more_msg: String,
    msg: String,
}

#[derive(Serialize, Deserialize)]
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

pub async fn place_order_ng(cookie_content: &CookieData, place_body: &PlaceOrderBody) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
	let mut headers = headers_checkout(&cookie_content);
	let data = crypt::random_hex_string(16);
	headers.insert("af-ac-enc-dat", HeaderValue::from_str(&data).unwrap());
    // Convert struct to JSON
	//let body_str = serde_json::to_string(&place_body)?;
	println!("Status: Start Place_Order");
	//println!("{:?}", body_str);
	//println!("Request Headers:\n{:?}", headers);

	let url2 = format!("https://mall.shopee.co.id/api/v4/checkout/place_order");
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
        .headers(headers)
		.json(&place_body)
        .version(Version::HTTP_2) 
        .send()
        .await?;
	
	println!("Status: Done Place_Order");
	//println!("Status: {}", response.status());
    let v: Value = response.json().await?;
	println!("Body: {}", v);
	Ok(v)
}

pub async fn get_builder(cookie_content: &CookieData, 
    device_info: &DeviceInfo, 
    product_info: &ProductInfo, 
    address_info: &AddressInfo, quantity: i32, 
    chosen_model: &ModelInfo, chosen_payment: &PaymentInfo, 
    chosen_shipping: &ShippingInfo, freeshipping_voucher: Option<Vouchers>, 
    platform_vouchers_target: Option<Vouchers>, shop_vouchers_target: Option<Vouchers>) -> Result<PlaceOrderBody, Box<dyn std::error::Error>> {
	let current_time = Utc::now();
    let freeshipping_voucher_clone = freeshipping_voucher.clone();
    let shop_id = product_info.shop_id;

    let free_shipping_thread = tokio::spawn(async move{
        match freeshipping_voucher {
            Some(shipping_vc) => FreeShippingVoucherInfo {
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
            },
            None => FreeShippingVoucherInfo {
                free_shipping_voucher_id: 0,
                free_shipping_voucher_code: None,
                disabled_reason: Some("".to_string()),
                disabled_reason_code: 0,
                banner_info: None,
                required_be_channel_ids: None,
                required_spm_channels: None,
            },
        }
    });

    let shop_vouchers_thread = tokio::spawn(async move{
        match shop_vouchers_target {
            Some(shop) => vec![Some(ShopVoucher {
                shopid: shop_id,
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

    let platform_vouchers_thread = tokio::spawn(async move{
        match platform_vouchers_target {
            Some(platform) => vec![Some(PlatformVoucher {
                voucher_code: platform.voucher_code.clone(),
                promotionid: platform.promotionid,
            })],
            None => vec![],
        }
    });

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

    let body_json = BodyJson {
        timestamp: current_time.timestamp(),
        shoporders: vec![ShopOrder {
            shop: Shop {
                shopid: product_info.shop_id,
            },
            items: vec![Item {
                itemid: product_info.item_id,
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
        selected_payment_channel_data: chosen_payment.selected_get.clone(),
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
                addressid: address_info.id,
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
	headers.insert("af-ac-enc-dat", HeaderValue::from_str(&data).unwrap());
    // Convert struct to JSON
    //let body_str = serde_json::to_string(&body_json)?;
	println!("Status: Start Checkout");
	//println!("{:?}", body_str);
	//println!("Request Headers:\n{:?}", headers);

	let url2 = format!("https://mall.shopee.co.id/api/v4/checkout/get");
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
        .headers(headers)
		.json(&body_json)
        .version(Version::HTTP_2) 
        .send()
        .await?;

	println!("Status: Done Checkout");
    // Handle response as needed
	//println!("Request Headers:\n{:?}", headers);
	println!("Status: {}", response.status());
	let v: Value = response.json().await?;
	//println!("Body: {}", body_resp);
    let keys = vec![
        "checkout_price_data",
        "order_update_info",
        "dropshipping_info",
        "promotion_data",
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

    // Menyimpan hasil dalam struct PlaceOrderBody
    let mut place_order_body = PlaceOrderBody::new(&device_info);

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