use rquest as reqwest;
use reqwest::Version;
use std::sync::Arc;
use std::mem::drop;
use serde::Deserialize;

use crate::prepare::{self, CookieData, ModelInfo, ShippingInfo, PaymentInfo, ProductInfo, AddressInfo, FS_BASE_HEADER};
use crate::crypt::DeviceInfo;
use crate::task::{self};

#[derive(Deserialize, Debug)]
struct GetRespPdp {
    data: Option<PdpData>,
}
#[derive(Deserialize, Debug)]
struct PdpData {
    item: Option<ItemPdp>,
    shop: Option<ShopPdp>,
    product_shipping: Option<ShippingPdp>,
}
#[derive(Deserialize, Debug, Clone)]
struct ItemPdp {
    title: Option<String>,
    models: Option<Vec<ModelsPdp>>,
}
#[derive(Deserialize, Debug, Clone)]
struct ShopPdp {
    is_official_shop: Option<bool>,
}
#[derive(Deserialize, Debug, Clone)]
struct ModelsPdp {
    pub name: String,
    pub price: i64,
    pub stock: i64,
    pub model_id: i64,
}
#[derive(Deserialize, Debug)]
struct ShippingPdp {
    ungrouped_channel_infos: Option<Vec<ChannelInfo>>,
}
#[derive(Deserialize, Debug)]
struct ChannelInfo {
    channel_id: i64,
    name: String,
    price_before_discount: Option<PriceBD>,
}
#[derive(Deserialize, Debug)]
struct PriceBD {
    single_value: i64,
}

pub async fn get_shipping_data(client: Arc<reqwest::Client>, headers: Arc<reqwest::header::HeaderMap>, shared_headers: Arc<reqwest::header::HeaderMap>, device_info: &DeviceInfo, product_info: Option<&ProductInfo>, address_info: &AddressInfo, chosen_model: &ModelInfo, chosen_payment: &PaymentInfo, chosen_shipping: &ShippingInfo) -> anyhow::Result<Vec<ShippingInfo>> {
    let product_info = match product_info {
        Some(info) => info.clone(),
        None => {
            ProductInfo::from(chosen_model)
        }
    };
    let cookie_data = CookieData::from_headers(&shared_headers.clone());
    let get_body_ship = task::get_builder(&device_info, &address_info, &[chosen_model.clone()], &chosen_payment, &chosen_shipping, None, None, None).await?;
    let (shipping_info_result, shipping_info_result2, shipping_orders_result) = tokio::join!(
        prepare::kurir(client.clone(), headers.clone(), &product_info, &address_info),
        get_pdp(client.clone(), &product_info, &cookie_data),
        task::checkout_get(client.clone(), shared_headers.clone(), &get_body_ship)

    );
    let mut shipping_info = shipping_info_result?;
    let (_, _, _, mut shipping_info2, _) = shipping_info_result2?;
    shipping_info.append(&mut shipping_info2);
    let (_, _, _, _, _, _, shipping_orders, _, _, _, _, _, _, _, _) = shipping_orders_result?;

    let mut tasks = Vec::new();
    let device_info = Arc::new(device_info.clone());
    let address_info = Arc::new(address_info.clone());
    let chosen_model = Arc::new(chosen_model.clone());
    let chosen_payment = Arc::new(chosen_payment.clone());

    println!("{}", shipping_orders[0]["selected_logistic_channelid"]);
    if let Some(integrated_channelids) = shipping_orders[0]["logistics"]["integrated_channelids"].as_array() {
        for integrated in integrated_channelids {
            shipping_info.push(ShippingInfo {
                original_cost: shipping_orders[0]["logistics"]["logistic_channels"][integrated.to_string()]["shipping_fee_data"]["shipping_fee_before_discount"].as_i64().unwrap_or(0),
                channelid: integrated.as_i64().unwrap_or(0),
                channelidroot: integrated.as_i64().unwrap_or(0),
                channel_name: shipping_orders[0]["logistics"]["logistic_channels"][integrated.to_string()]["channel_data"]["name"].to_string(),
            });
            let integrated = Arc::new(integrated.clone());
            let device_info = Arc::clone(&device_info);
            let address_info = Arc::clone(&address_info);
            let chosen_model = Arc::clone(&chosen_model);
            let chosen_payment = Arc::clone(&chosen_payment);
            let client_clone = Arc::clone(&client);
            let shared_headers_clone = Arc::clone(&shared_headers);
            let mut chosen_shipping = chosen_shipping.clone();
            
            let task = {
                let integrated = Arc::clone(&integrated);
                tokio::spawn(async move {
                    let mut shipping_info = Vec::new();
                    chosen_shipping.channelid = integrated.as_i64().unwrap_or(0);
                    println!("integrated_special: {:?}", chosen_shipping);  
                    let get_body_shipl = match task::get_builder(&device_info, &address_info, &[(*chosen_model).clone()], &chosen_payment, &chosen_shipping, None, None, None).await
                    {
                        Ok(body) => body,
                        Err(err) => {
                            eprintln!("Error in get_builder: {:?}", err);
                            return None;
                        }
                    };
                    let (_, _, _, _, _, _, shipping_ordersl, _, _, _, _, _, _, _, _) = match task::checkout_get(client_clone, shared_headers_clone, &get_body_shipl).await
                    {
                        Ok(body) => body,
                        Err(err) => {
                            eprintln!("Error in get_builder: {:?}", err);
                            return None;
                        }
                    };
                    if let Some(specific_channel_ids) = shipping_ordersl[0]["logistics"]["specific_channel_mappings"][integrated.to_string()]["specific_channel_ids"].as_array() {
                        for specific in specific_channel_ids {
                            println!("specific_channelid: {}", specific);
                            shipping_info.push(ShippingInfo {
                                original_cost: shipping_ordersl[0]["logistics"]["logistic_channels"][specific.to_string()]["shipping_fee_data"]["shipping_fee_before_discount"].as_i64().unwrap_or(0),
                                channelid: specific.as_i64().unwrap_or(0),
                                channelidroot: integrated.as_i64().unwrap_or(0),
                                channel_name: shipping_ordersl[0]["logistics"]["logistic_channels"][specific.to_string()]["channel_data"]["name"].as_str().unwrap_or("").to_string(),
                            });
                        }
                    } else {
                        eprintln!("specific_channel_ids not found or is not an array for integrated_channelid: {}", integrated);
                    }
                    drop(shipping_ordersl);
                    Some(shipping_info)
                })
            };
            tasks.push(task);
        }
    }
    let results = futures::future::join_all(tasks).await;
    for result in results {
        if let Ok(Some(mut info)) = result {
            shipping_info.append(&mut info);
        }
    }
    drop(shipping_orders);
    println!("{:?}", shipping_info);
    Ok(shipping_info)
}
pub async fn get_pdp(client: Arc<reqwest::Client>, product_info: &ProductInfo, cookie_content: &CookieData) -> Result<(String, Vec<ModelInfo>, bool, Vec<ShippingInfo>, String), anyhow::Error> {
    let url2 = format!("https://mall.shopee.co.id/api/v4/pdp/get?item_id={}&shop_id={}", product_info.item_id, product_info.shop_id);
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
    let hasil: GetRespPdp = response.json().await?;
    //println!("Body: {}", &body);

    let (name, raw_models, is_official_shop, raw_shipping_info) = if let Some(data) = hasil.data {
        (
            data.item.clone().unwrap().title.unwrap_or_else(|| "Unknown".into()),
            data.item.clone().unwrap().models.unwrap_or_default(),          // bisa kosong
            data.shop.clone().unwrap().is_official_shop.unwrap_or(false),
            data.product_shipping.unwrap().ungrouped_channel_infos.unwrap_or_default(),
        )
    } else {
        println!("Status: {}", status_code);
        ("INVALID".to_string(), Vec::new(), false, Vec::new())
    };
    let models_info: Vec<ModelInfo> = raw_models
        .into_iter()
        .map(|m| ModelInfo {
            name: m.name,
            product_name: name.clone(),
            price: m.price,
            stock: m.stock,
            modelid: m.model_id,
            promotionid: 0,
            shop_id:   product_info.shop_id,   // override di *sini*
            item_id:   product_info.item_id,
            quantity:  1,
            voucher_code: None,
        })
        .collect();
    let shipping_info: Vec<ShippingInfo> = raw_shipping_info
        .into_iter()
        .map(|info| ShippingInfo {
            original_cost: info.price_before_discount.and_then(|p| Some(p.single_value)).unwrap_or(0),
            channelid: info.channel_id,
            channelidroot: info.channel_id,
            channel_name: info.name,
        })
        .collect();
    Ok((name, models_info, is_official_shop, shipping_info, status_code))
}