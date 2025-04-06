use runtime::prepare::{self, CookieData, ModelInfo, ShippingInfo, PaymentInfo, ProductInfo, AddressInfo};
use runtime::crypt::DeviceInfo;
use runtime::task::{self};
use std::sync::Arc;
use std::mem::drop;

use crate::get_user_input;
use crate::Opt;

pub fn choose_shipping(shippings: &[ShippingInfo], opt: &Opt) -> Option<ShippingInfo> {
    println!("shipping yang tersedia:");

    for (index, shipping) in shippings.iter().enumerate() {
        println!("{}. {} - Harga: {} - Id: {}", index + 1, shipping.channel_name, shipping.original_cost / 100000, shipping.channelid);
    }

    if let Some(kurir) = &opt.kurir {
        // If opt.kurir is present, find the shipping with a matching channel_name
        if let Some(selected_shipping) = shippings.iter().find(|shipping| shipping.channel_name == *kurir) {
            println!("{:?}", selected_shipping);
            return Some(selected_shipping.clone());
        } else {
            println!("Tidak ada shipping dengan nama '{}'", kurir);
            return None;
        }
    }

	let user_input = get_user_input("Pilih Shipping yang disediakan: ");

    // Convert user input to a number
    if let Ok(choice_index) = user_input.trim().parse::<usize>() {
        // Return the selected shipping based on the index
        println!("{:?}", shippings.get(choice_index - 1).cloned());
        return shippings.get(choice_index - 1).cloned();
    } else if user_input.trim().to_uppercase() == "N" {
        println!("Menampilkan lebih banyak pilihan...");
    }

    None
}

pub async fn get_shipping_data(cookie_data: &CookieData, device_info: &DeviceInfo, product_info: &ProductInfo, address_info: &AddressInfo, quantity: i32,  chosen_model: &ModelInfo, chosen_payment: &PaymentInfo, chosen_shipping: &ShippingInfo) -> Result<Vec<ShippingInfo>, Box<dyn std::error::Error>> {
    let get_body_ship = task::get_builder(&device_info, &product_info, &address_info, quantity, &chosen_model, &chosen_payment, &chosen_shipping, None, None, None).await?;
    let (shipping_info_result, shipping_orders_result) = tokio::join!(
        prepare::kurir(&cookie_data, &product_info, &address_info),
        task::checkout_get(&cookie_data, &get_body_ship)

    );
    let mut shipping_info = shipping_info_result?;
    let (_, _, _, _, _, _, shipping_orders, _, _, _, _, _, _, _, _) = shipping_orders_result?;

    let mut tasks = Vec::new();
    let device_info = Arc::new(device_info.clone());
    let product_info = Arc::new(product_info.clone());
    let address_info = Arc::new(address_info.clone());
    let cookie_data = Arc::new(cookie_data.clone());
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
            let product_info = Arc::clone(&product_info);
            let address_info = Arc::clone(&address_info);
            let cookie_data = Arc::clone(&cookie_data);
            let chosen_model = Arc::clone(&chosen_model);
            let chosen_payment = Arc::clone(&chosen_payment);
            let mut chosen_shipping = chosen_shipping.clone();
            
            let task = {
                let integrated = Arc::clone(&integrated);
                tokio::spawn(async move {
                    let mut shipping_info = Vec::new();
                    chosen_shipping.channelid = integrated.as_i64().unwrap_or(0);
                    println!("integrated_special: {:?}", chosen_shipping);  
                    let get_body_shipl = match task::get_builder(&device_info, &product_info, &address_info, quantity, &chosen_model, &chosen_payment, &chosen_shipping, None, None, None).await
                    {
                        Ok(body) => body,
                        Err(err) => {
                            eprintln!("Error in get_builder: {:?}", err);
                            return None;
                        }
                    };
                    let (_, _, _, _, _, _, shipping_ordersl, _, _, _, _, _, _, _, _) = match task::checkout_get(&cookie_data, &get_body_shipl).await
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