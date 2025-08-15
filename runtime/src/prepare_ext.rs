use rquest as reqwest;
use std::sync::Arc;
use std::mem::drop;

use crate::prepare::{self, CookieData, ModelInfo, ShippingInfo, PaymentInfo, ProductInfo, AddressInfo};
use crate::crypt::DeviceInfo;
use crate::task::{self};

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
        prepare::ShippingInfo::kurir(client.clone(), headers.clone(), &product_info, &address_info),
        prepare::get_pdp(client.clone(), &product_info, &cookie_data),
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