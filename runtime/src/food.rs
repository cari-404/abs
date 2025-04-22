use rquest as reqwest;
use reqwest::Version;
use reqwest::header::{HeaderValue, HeaderMap};
use chrono::{Utc, Timelike};
use anyhow::Result;
use serde_json::{Value};
use std::collections::HashMap;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};

use crate::prepare::{ModelInfo, ShippingInfo, PaymentInfo, ProductInfo, AddressInfo};
use crate::voucher::Vouchers;
use crate::crypt::{self, DeviceInfo};

#[derive(Debug, Deserialize)]
struct ResponseOptions {
    data: DataOptions,
}

#[derive(Debug, Deserialize)]
struct DataOptions {
    groups: Vec<GroupOptions>,
}

#[derive(Debug, Deserialize)]
struct GroupOptions {
    options: Vec<OptionItem>,
    group_id: String,
    group_name: String,
}

#[derive(Debug, Deserialize)]
struct OptionItem {
    id: i64,
    name: String,
    price: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: DiscountData,
}

#[derive(Debug, Deserialize)]
struct DiscountData {
    timeslot_dishes: Vec<TimeslotDish>,
}

#[derive(Debug, Deserialize)]
struct TimeslotDish {
    flash_sale_dishes: Vec<FlashSaleDish>,
}

#[derive(Debug, Deserialize)]
struct FlashSaleDish {
    discount: Discount,
}

#[derive(Debug, Deserialize)]
pub struct Discount {
    pub id: String,
    pub timeslot_id: String,
    pub store_id: String,
    pub dish_id: String,
    pub dish_name: String,
    pub flash_sale_dish_name: String,
    pub stock: i32,
    pub sold_num: i32,
    pub limit_per_user: i32,
    pub discount_price: String,
    pub discount_status: i32,
    pub discount_type: i32,
    pub discount_percentage: i32,
    pub operator: String,
    pub create_time: String,
    pub update_time: String,
    pub category_ids: String,
    pub self_pick_up_price: i64,
    pub pubcurrent_delivery_mode: i32,
    pub support_delivery_type: i32,
}

pub struct GetFoodData {
    pub state: String,
    pub city: String,
    pub district: String,
    pub delivery_mode: i64,
}
pub static FOOD_HEADER_APP: Lazy<HeaderMap> = Lazy::new(|| {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("user-agent", HeaderValue::from_static("okhttp/3.12.4 app_type=1 platform=native_android os_ver=34 appver=34740"));
	headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("accept", HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert("content-type", HeaderValue::from_static("application/json;charset=utf-8"));
    headers.insert("shopee_http_dns_mode", HeaderValue::from_static("1"));
    headers.insert("x-shopee-client-timezone", HeaderValue::from_static("Asia/Jakarta"));
	headers.insert("af-ac-enc-sz-token", HeaderValue::from_static(""));
    headers
});

pub async fn get_food_data(client: Arc<reqwest::Client>, headers: Arc<HeaderMap>, device_info: &DeviceInfo, model_info: &ModelInfo, shipping_info: &ShippingInfo, payment_info: &PaymentInfo, product_info: &ProductInfo, address_info: &AddressInfo,) -> Result<Vec<Discount>> {
    let url = "https://foody.shopee.co.id/api/buyer/flash-sale/stores/1142540/dishes"; // Replace with actual URL
    let body  = GetFoodData {
        state: shipping_info.state.clone(),
        city: shipping_info.city.clone(),
        district: shipping_info.district.clone(),
        delivery_mode: 1,
    };
    
    let response = client
        .post(url)
        .headers((*headers).clone())
        .json(&body)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    if response.status().is_success() {
        let parsed: ApiResponse = response.json().await?;
        let discounts = parsed
            .data
            .timeslot_dishes
            .into_iter()
            .flat_map(|slot| slot.flash_sale_dishes.into_iter().map(|dish| dish.discount))
            .collect();
        Ok(discounts)
    } else {
        Err(anyhow::anyhow!("Failed to fetch data: {}", response.status()))
    }
}
pub async fn get_option(client: Arc<reqwest::Client>, headers: Arc<HeaderMap>, dish: &str) -> Result<Vec<(GroupInfo, Vec<OptionItem>)>>{
    let url = format!("https://foody.shopee.co.id/api/buyer/dishes/{}/option-groups?store_id=1142540", dish);
    let response = client
        .get(url)
        .headers((*headers).clone())
        .version(Version::HTTP_2) 
        .send()
        .await?;
    if response.status().is_success() {
        let json: ApiResponse = response.json().await?;
        let mut result = Vec::new();

        if let Some(data) = json.data {
            for group in data.groups {
                let info = GroupInfo {
                    group_id: group.group_id,
                    group_name: group.group_name,
                };
                result.push((info, group.options));
            }
        }
        Ok(result)
    } else {
        Err(anyhow::anyhow!("Failed to fetch data: {}", response.status()))
    }
}

#[derive(Debug)]
struct CartBody {
    quantity: i64,
    dish_id: i64,
    store_id: i64,
    option_groups: Vec<GroupCart>,
    remark: String,
    delivery_mode: i64,
    flash_sale_discount_id: i64,
    flash_sale_time_slot_id: i64,
    discount_type: i64,
    promotion_tool_control: i64,
}

#[derive(Debug)]
struct GroupCart {
    id: String,
    options: Vec<OptionCart>,
}

#[derive(Debug)]
struct OptionCart {
    id: String,
    price: String,
    quantity: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseCart {
    pub data: DataCart,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataCart {
    pub items: Vec<CartItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CartItem {
    pub id: String,
    pub detail: ItemDetail,
    pub quantity: i64,
    pub status: i64,
    pub remark: String,
    pub item_id: String,
    pub dish_id: String,
    pub create_time: String,
    pub primary_id: String,
    pub cart_item_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemDetail {
    pub dish: Dish,
    pub option_groups: Vec<OptionGroup>,
    pub flash_sale_discount: FlashSaleDiscount,
    pub user_prepaid_sku: Option<serde_json::Value>, // null
    pub discount_type: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dish {
    pub id: String,
    pub name: String,
    pub image: String,
    pub price: String,
    pub catalog_id: i64,
    pub list_price: String,
    pub limit_per_order: i64,
    pub flash_sale_limit: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionGroup {
    pub id: String,
    pub name: String,
    pub options: Vec<DishOption>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DishOption {
    pub id: String,
    pub name: String,
    pub price: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlashSaleDiscount {
    pub id: String,
    pub timeslot_id: String,
    pub discount_price: String,
}

pub async fn add_cart(client: Arc<reqwest::Client>, headers: Arc<HeaderMap>, dish: &str, selected_group: &[GroupInfo], selected_option: &[OptionItem], flash_sale_discount: &Discount) -> Result<Vec<CartItem>> {
    let url = format!("https://foody.shopee.co.id/api/buyer/cart/items");
    let body = CartBody{
        quantity: 1,
        dish_id: dish,
        store_id: 1142540,
        option_groups: vec![GroupCart{
            id: selected_option,
            options: vec![OptionCart{
                id: 1802644711595010,
                price: 0,
                quantity: 1
            }],
        }],
        remark: "".to_string(),
        delivery_mode: 1,
        flash_sale_discount_id: flash_sale_discount.id,
        flash_sale_time_slot_id: flash_sale_discount.timeslot_id,
        discount_type: 2,
        promotion_tool_control: 1 
    };
    
    let response = client
        .post(url)
        .headers((*headers).clone())
        .json(&body)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    if response.status().is_success() {
        let parsed:ResponseCart = response.json().await?;
        Ok(parsed.data.items)
    } else {
        Err(anyhow::anyhow!("Failed to fetch data"))
    }

}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckoutRequest {
    pub store_id: String,
    pub delivery_address: DeliveryAddress,
    pub cart_items: Vec<CartItem>,
    pub feature_flag: String,
    pub saver_delivery_enable: bool,
    pub coins: i64,
    pub voucher_identifiers: Vec<String>,
    pub payment_info: PaymentInfo,
    pub promotion_tool_control: i64,
    pub delivery_type: i64,
    pub checkout_request_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeliveryAddress {
    pub name: String,
    pub phone: String,
    pub location: Location,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
    pub city: String,
    pub state: String,
    pub district: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CartItem {
    pub id: String,
    pub detail: Detail,
    pub quantity: i64,
    pub status: i64,
    pub remark: String,
    pub item_id: String,
    pub dish_id: String,
    pub create_time: String,
    pub primary_id: String,
    pub cart_item_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Detail {
    pub dish: Dish,
    pub option_groups: Vec<OptionGroup>,
    pub flash_sale_discount: FlashSaleDiscount,
    pub user_prepaid_sku: Option<serde_json::Value>,
    pub discount_type: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dish {
    pub id: String,
    pub name: String,
    pub image: String,
    pub price: String,
    pub catalog_id: i64,
    pub list_price: String,
    pub limit_per_order: i64,
    pub flash_sale_limit: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionGroup {
    pub id: String,
    pub name: String,
    pub options: Vec<OptionItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionItem {
    pub id: String,
    pub name: String,
    pub price: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlashSaleDiscount {
    pub id: String,
    pub timeslot_id: String,
    pub discount_price: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentInfo {
}
pub async fn order_draft(client: Arc<reqwest::Client>, headers: Arc<HeaderMap>, cart_item: &[CartItem] flash_sale_discount: &Discount) -> Result<(String, String, String)> {
    let url = format!("https://foody.shopee.co.id/api/buyer/order-promotion-draft");
    let body = json!({
        
    })
}
pub async fn order_food(client: Arc<reqwest::Client>, headers: Arc<HeaderMap>, cart_item: &[CartItem] flash_sale_discount: &Discount) -> Result<(String, String, String)> {
    let url = format!("https://foody.shopee.co.id/api/buyer/orders");
    let body  = PlaceOrderFood {
        anti_fraud: AntiFraud{
            sps_fd: device_info.device_sz_fingerprint.clone(),
            device_location: DeviceLocation{
                precise_longitude: 112.7171911,
                precise_latitude: -7.2577994,
            },
        },
        order: Order{
            payment_method: 2,
            items: vec![OrderItem{
                id: cart_item.id,
                cart_item,
                unit_price: cart_item.detail.dish.price.clone(),
                flat_fee: "0".to_string(),
                subtotal: cart_item.detail.dish.price.clone(),
                remark: "".to_string(),
                unit_list_price: cart_item.detail.dish.list_price.clone(),
                status: 0,
                discount_type:2,
            }],
            delivery_method: 1,
            delivery_mode: 1,
            delivery_address: DeliveryAddress{
                name: "Faizal Rahman".to_string(),
                phone: "6287861700252".to_string(),
                location: Location{
                    latitude: -7.257941632231029,
                    longitude: 112.71724099293351,
                    address: "Jalan Pacuan Kuda No. 14, Petemon, Sawahan".to_string(),
                    city: "KOTA SURABAYA".to_string(),
                    state: "JAWA TIMUR".to_string(),
                    district: "SAWAHAN".to_string(),
                },
            },
            scheduled_delivery_start_time: 0,
            scheduled_delivery_end_time: 0,
            delivery_type: 1,
            store: Store{
                {
                id: "1142540".to_string(),
                name: "McDonald's - Basuki Rahmat Surabaya".to_string(),
                logo: "id-11134505-7rbkc-m8op7bcrdybf41".to_string(),
                partner_type: 2,
                is_use_wallet: 1,
                location: {
                    state: "JAWA TIMUR".to_string(),
                    city: "KOTA SURABAYA".to_string(),
                    district: "GENTENG".to_string(),
                    address: "Jl. Basuki Rachmat No. 21-23".to_string(),
                    latitude: -7.26397,
                    longitude: 112.74136,
                    precise_latitude: -7.26397,
                    precise_longitude: 112.74136
                },
            },
            amount: Amount{
                subtotal: String::new(),
                tax_amount: String::new(),
                merchant_service_fee: String::new(),
                platform_service_fee: String::new(),
                shipping_fee: String::new(),
                total_amount: String::new(),
                shipping_basic_fee: String::new(),
                shipping_surge_fee: String::new(),
                merchant_surchange_fee: String::new(),
                promotion: {
                    item_voucher_amount: String::new(),
                    item_discount_amount: String::new(),
                    shipping_voucher_amount: String::new(),
                    shipping_discount_amount: String::new(),
                    coins_redeemed_amount: String::new(),
                    coins_earning: String::new(),
                    coins_cashback: 0
                },
                small_order_fee: String::new(),
                shipping_fare_extra_fee: String::new(),
                shipping_fare_discount_amount: String::new(),
                parking_fee: String::new(),
                non_partner_fee: String::new(),
                bad_weather_fee: String::new(),
                holiday_fee: String::new(),
                late_night_fee: String::new(),
                shipping_fee_tax_amount: String::new(),
                shipping_fee_tax_rate: String::new(),
                shipping_fee_buyer_extra: 0,
                shipping_fee_buyer_discount: 0
                },
            },
            remark: String::new(),
            payment_info: PaymentInfo{
            },
        },
        promotion: Promotion{
            spend_coins: SpendCoin{
                coins: 0,
                discount: 0,
            },
            earn_coins: EarnCoin{
                type: 1,
                coins: 0,
            },
        },
        shipping_fee_request_id: String::new(),
        feature_flag: "14".to_string(),
        checkout_request_id: uuid::Uuid::new_v4().to_string(),
    };
    
    let response = client
        .post(url)
        .headers((*headers).clone())
        .json(&body)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    if response.status().is_success() {
        let v:Value = response.json().await?;

        Ok((body, String::new(), String::new()))
    } else {
        Err(anyhow::anyhow!("Failed to fetch data"))
    }
}


pub fn headers_food(cookie_content: &CookieData) -> HeaderMap {
    let mut headers = FOOD_HEADER_APP.clone();
	headers.insert("cookie", HeaderValue::from_str(&cookie_content.cookie_content).unwrap());
    headers
}