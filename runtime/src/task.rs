#![recursion_limit = "1024"]
use rquest as reqwest;
use reqwest::impersonate::Impersonate;
use reqwest::{ClientBuilder, header::HeaderMap, Error, Response, Version, Body};
use reqwest::header::HeaderValue;
use chrono::{Utc};
use anyhow::Result;
use serde_json::json;
use crate::prepare::ShippingInfo;
use crate::prepare::ModelInfo;
use crate::prepare::PaymentInfo;

/*pub async fn place_order(cookie_content: &str, shop_id_str: &str, item_id_str: &str, quantity_str: &str, chosen_model: &ModelInfo, chosen_payment: &PaymentInfo, chosen_shipping: &ShippingInfo) -> Result<(), Box<dyn std::error::Error>> {
	let headers = headers_checkout(&cookie_content);
	let shop_id = shop_id_str.parse::<i64>().expect("Failed to parse shop_id");
	let item_id = item_id_str.parse::<i64>().expect("Failed to parse item_id");
	let quantity = quantity_str.parse::<i64>().expect("Failed to parse quantity");
	let channel_id: u64 = chosen_payment.channel_id.parse().expect("Failed to parse channel_id");
	let version: u64 = chosen_payment.version.parse().expect("Failed to parse version");
	let optioninfo: String = chosen_payment.option_info.clone();
	let current_time = Utc::now();
	
	let body_json = json!({
	  "headers": {},
	  "client_id": 5,
	  "cart_type": 1,
	  "timestamp": current_time.timestamp(),
	  "checkout_price_data": {
		"merchandise_subtotal": 2400000000, //response from get
		"shipping_subtotal_before_discount": 1600000000, //response from get
		"shipping_discount_subtotal": 0,
		"shipping_subtotal": 1600000000, //response from get
		"tax_payable": 0,
		"tax_exemption": 0,
		"import_tax_amount": 0,
		"icms_amount": 0,
		"iof_amount": 0,
		"custom_tax_subtotal": 0,
		"promocode_applied": null,
		"credit_card_promotion": null,
		"shopee_coins_redeemed": null,
		"group_buy_discount": 0,
		"bundle_deals_discount": null,
		"price_adjustment": null,
		"buyer_txn_fee": 0,
		"buyer_service_fee": 100000000, //response from get
		"insurance_subtotal": 0,
		"insurance_before_discount_subtotal": 0,
		"insurance_discount_subtotal": 0,
		"vat_subtotal": 0,
		"total_payable": 4100000000 //response from get
	  },
	  "order_update_info": {},
	  "dropshipping_info": {
		"enabled": false,
		"name": "",
		"phone_number": ""
	  },
	  "promotion_data": {
		"can_use_coins": true,
		"use_coins": false,
		"platform_vouchers": [
		  {
			"promotionid": 786674414714880, 
			"voucher_code": "DC50100RB01011",
			"applied_voucher_code": "",
			"invalid_message_code": 8,
			"reward_type": 0,
			"discount_value": 0,
			"coin_earned": 0,
			"required_be_channel_ids": [],
			"required_spm_channels": []
		  }
		],
		"free_shipping_voucher_info": {
		  "free_shipping_voucher_id": 783739022934016,
		  "free_shipping_voucher_code": "FSV-783739022934016",
		  "disabled_reason": "Voucher Gratis Ongkir tidak dapat dipakai pada jasa pengiriman yang dipilih.",
		  "disabled_reason_code": 2,
		  "banner_info": {
			"msg": "Voucher Gratis Ongkir tidak dapat dipakai pada jasa pengiriman yang dipilih.",
			"learn_more_msg": ""
		  },
		  "required_be_channel_ids": [],
		  "required_spm_channels": [
			{
			  "spm_channel_id": 8008500,
			  "spm_option_info": ""
			}
		  ]
		},
		"spl_voucher_info": null,
		"highlighted_platform_voucher_type": -1,
		"shop_voucher_entrances": [
		  {
			"shopid": 1093212352,
			"status": true
		  }
		],
		"applied_voucher_code": null,
		"voucher_code": "DC50100RB01011",
		"voucher_info": {
		  "coin_earned": 0,
		  "voucher_code": "DC50100RB01011",
		  "coin_percentage": 0,
		  "discount_percentage": 50,
		  "discount_value": 0,
		  "promotionid": 786674414714880,
		  "reward_type": 0,
		  "used_price": 0
		},
		"invalid_message": "Maaf, promosi dengan kode promo ini belum dimulai. Mohon coba kembali nanti.",
		"price_discount": 0,
		"coin_info": {
		  "coin_offset": 500000,
		  "coin_used": 5,
		  "coin_earn": 0,
		  "coin_earn_by_voucher": 0,
		  "coin_earn_by_maricredit": 0,
		  "coin_earn_rate_by_maricredit": 0
		},
		"card_promotion_id": null,
		"card_promotion_enabled": false
	  },
	  "selected_payment_channel_data": {
		"channel_id": 8005200,
		"channel_item_option_info": {
		  "option_info": "89052007"
		},
		"version": 2
	  },
	  "shoporders": [
		{
		  "shop": {
			"shopid": 1093212352,
			"shop_name": "YoyiC Official",
			"cb_option": false,
			"is_official_shop": false,
			"remark_type": 0,
			"support_ereceipt": false,
			"shop_ereceipt_type": 0,
			"seller_user_id": 1093471063,
			"shop_tag": 3
		  },
		  "items": [
			{
			  "itemid": 20285271157,
			  "modelid": 204654883278,
			  "quantity": 1,
			  "item_group_id": null,
			  "insurances": [],
			  "shopid": 1093212352,
			  "shippable": true,
			  "non_shippable_err": "",
			  "none_shippable_reason": "",
			  "none_shippable_full_reason": "",
			  "price": 2400000000, //response from get
			  "name": "6 Botol YoyiC Minuman Susu Fermentasi Rasa Original, Jeruk, Blueberry, Peach 130 ML*6",
			  "model_name": "Original",
			  "add_on_deal_id": 0,
			  "is_add_on_sub_item": false,
			  "is_pre_order": false,
			  "is_streaming_price": false,
			  "image": "id-11134207-7r98t-loqu9jy4in6zc4",
			  "checkout": true,
			  "categories": [
				{
				  "catids": [
					100629,
					100652,
					100838,
					101593
				  ]
				}
			  ],
			  "is_spl_zero_interest": false,
			  "is_prescription": false,
			  "channel_exclusive_info": {
				"source_id": 0,
				"token": "",
				"is_live_stream": false,
				"is_short_video": false
			  },
			  "offerid": 0,
			  "supports_free_returns": false
			}
		  ],
		  "tax_info": {
			"use_new_custom_tax_msg": true,
			"custom_tax_msg": "",
			"custom_tax_msg_short": "",
			"remove_custom_tax_hint": true,
			"help_center_url": ""
		  },
		  "tax_payable": 0,
		  "import_tax_amount": 0,
		  "icms_amount": 0,
		  "iof_amount": 0,
		  "shipping_id": 1,
		  "shipping_fee_discount": 0,
		  "shipping_fee": 1600000000, //response from get
		  "order_total_without_shipping": 2400000000, //response from get
		  "order_total": 4000000000, //response from get
		  "buyer_remark": null
		}
	  ],
	  "shipping_orders": [
		{
		  "shipping_id": 1,
		  "shoporder_indexes": [
			0
		  ],
		  "selected_logistic_channelid": 8003,
		  "selected_preferred_delivery_time_option_id": 0,
		  "selected_preferred_delivery_window": {},
		  "buyer_remark": null,
		  "buyer_address_data": {
			"addressid": 21685906,
			"address_type": 0,
			"tax_address": ""
		  },
		  "fulfillment_info": {
			"fulfillment_flag": 64,
			"fulfillment_source": "",
			"managed_by_sbs": false,
			"order_fulfillment_type": 2,
			"warehouse_address_id": 0,
			"is_from_overseas": false
		  },
		  "logistics": {
			"integrated_channelids": [
			  8000,
			  8001,
			  8002,
			  8003,
			  8005,
			  8006
			],
			"non_integrated_channelids": [],
			"voucher_wallet_checking_channel_ids": [],
			"logistic_channels": {
			  "8000": {
				"channel_data": {
				  "address_type": 0,
				  "channelid": 8000,
				  "cod_supported": false,
				  "enabled": false,
				  "is_mask_channel": 1,
				  "name": "Instant - 2 Jam",
				  "priority": 4,
				  "warning": "max_distance_limit_exceeded",
				  "warning_msg": "Melebihi batas jarak pengiriman",
				  "multi_address_validation_enabled": false,
				  "invalid_address_ids": null
				},
				"cod_data": {
				  "cod_available": false
				},
				"delivery_data": {
				  "delay_message": "",
				  "max_days": null,
				  "min_days": null
				},
				"shipping_fee_data": {
				  "chargeable_shipping_fee": 0,
				  "shipping_fee_before_discount": 0
				},
				"buyer_address_data": {
				  "addressid": 21685906,
				  "address_type": 0
				},
				"shippable_item_ids": [
				  20285271157
				]
			  },
			  "8001": {
				"channel_data": {
				  "address_type": 0,
				  "channelid": 8001,
				  "cod_supported": false,
				  "enabled": false,
				  "is_mask_channel": 1,
				  "name": "Same Day",
				  "priority": 3,
				  "warning": "max_distance_limit_exceeded",
				  "warning_msg": "Melebihi batas jarak pengiriman",
				  "multi_address_validation_enabled": false,
				  "invalid_address_ids": null
				},
				"cod_data": {
				  "cod_available": false
				},
				"delivery_data": {
				  "delay_message": "",
				  "max_days": null,
				  "min_days": null
				},
				"shipping_fee_data": {
				  "chargeable_shipping_fee": 0,
				  "shipping_fee_before_discount": 0
				},
				"buyer_address_data": {
				  "addressid": 21685906,
				  "address_type": 0
				},
				"shippable_item_ids": [
				  20285271157
				]
			  },
			  "8002": {
				"channel_data": {
				  "address_type": 0,
				  "channelid": 8002,
				  "cod_supported": false,
				  "enabled": true,
				  "is_mask_channel": 1,
				  "name": "Next Day",
				  "priority": 6,
				  "warning": "",
				  "warning_msg": "",
				  "multi_address_validation_enabled": false,
				  "invalid_address_ids": null
				},
				"cod_data": {
				  "cod_available": false
				},
				"delivery_data": {
				  "delay_message": "",
				  "detail_info": {
					"apt": 0.351528,
					"cdt_max": 2.9778,
					"cdt_min": 1.5463,
					"edt_max_dt": "2024-01-05",
					"edt_min_dt": "2024-01-02",
					"he_cdt": 0,
					"he_pt": 2
				  },
				  "display_mode": "edt_by_date",
				  "max_days": 5,
				  "min_days": 2,
				  "estimated_delivery_time_max": 5,
				  "estimated_delivery_time_min": 2,
				  "estimated_delivery_date_from": 1704198900,
				  "estimated_delivery_date_to": 1704458100,
				  "has_edt": true,
				  "is_cross_border": false,
				  "is_rapid_sla": false,
				  "is_shopee_24h": false
				},
				"shipping_fee_data": {
				  "chargeable_shipping_fee": 3700000000,
				  "shipping_fee_before_discount": 3700000000
				},
				"buyer_address_data": {
				  "addressid": 21685906,
				  "address_type": 0
				},
				"shippable_item_ids": [
				  20285271157
				]
			  },
			  "8003": {
				"channel_data": {
				  "address_type": 0,
				  "channelid": 8003,
				  "cod_supported": false,
				  "enabled": true,
				  "is_mask_channel": 1,
				  "name": "Reguler",
				  "priority": 1,
				  "warning": "",
				  "warning_msg": "",
				  "multi_address_validation_enabled": false,
				  "invalid_address_ids": null
				},
				"cod_data": {
				  "cod_available": true
				},
				"delivery_data": {
				  "delay_message": "",
				  "detail_info": {
					"apt": 0.351528,
					"cdt_max": 3.0433,
					"cdt_min": 1.8151,
					"edt_max_dt": "2024-01-05",
					"edt_min_dt": "2024-01-02",
					"he_cdt": 0,
					"he_pt": 2
				  },
				  "display_mode": "edt_by_date",
				  "max_days": 5,
				  "min_days": 2,
				  "estimated_delivery_time_max": 5,
				  "estimated_delivery_time_min": 2,
				  "estimated_delivery_date_from": 1704198900,
				  "estimated_delivery_date_to": 1704458100,
				  "has_edt": true,
				  "is_cross_border": false,
				  "is_rapid_sla": false,
				  "is_shopee_24h": false
				},
				"shipping_fee_data": {
				  "chargeable_shipping_fee": 1600000000,
				  "shipping_fee_before_discount": 1600000000
				},
				"buyer_address_data": {
				  "addressid": 21685906,
				  "address_type": 0
				},
				"shippable_item_ids": [
				  20285271157
				]
			  },
			  "8005": {
				"channel_data": {
				  "address_type": 0,
				  "channelid": 8005,
				  "cod_supported": false,
				  "enabled": true,
				  "is_mask_channel": 1,
				  "name": "Hemat",
				  "priority": 1,
				  "warning": "",
				  "warning_msg": "",
				  "multi_address_validation_enabled": false,
				  "invalid_address_ids": null
				},
				"cod_data": {
				  "cod_available": true
				},
				"delivery_data": {
				  "delay_message": "",
				  "detail_info": {
					"apt": 0.351528,
					"cdt_max": 3.0589,
					"cdt_min": 1.8484,
					"edt_max_dt": "2024-01-05",
					"edt_min_dt": "2024-01-02",
					"he_cdt": 0,
					"he_pt": 2
				  },
				  "display_mode": "edt_by_date",
				  "max_days": 5,
				  "min_days": 2,
				  "estimated_delivery_time_max": 5,
				  "estimated_delivery_time_min": 2,
				  "estimated_delivery_date_from": 1704198900,
				  "estimated_delivery_date_to": 1704458100,
				  "has_edt": true,
				  "is_cross_border": false,
				  "is_rapid_sla": false,
				  "is_shopee_24h": false
				},
				"shipping_fee_data": {
				  "chargeable_shipping_fee": 1500000000,
				  "shipping_fee_before_discount": 1500000000
				},
				"buyer_address_data": {
				  "addressid": 21685906,
				  "address_type": 0
				},
				"shippable_item_ids": [
				  20285271157
				]
			  },
			  "8006": {
				"channel_data": {
				  "address_type": 0,
				  "channelid": 8006,
				  "cod_supported": false,
				  "enabled": false,
				  "is_mask_channel": 1,
				  "name": "Kargo",
				  "priority": 2,
				  "warning": "shipping_option_unsupported",
				  "warning_msg": "Jasa kirim tidak didukung",
				  "multi_address_validation_enabled": false,
				  "invalid_address_ids": null
				},
				"cod_data": {
				  "cod_available": false
				},
				"delivery_data": {
				  "delay_message": "",
				  "max_days": null,
				  "min_days": null
				},
				"shipping_fee_data": {
				  "chargeable_shipping_fee": 0,
				  "shipping_fee_before_discount": 0
				},
				"buyer_address_data": {
				  "addressid": 21685906,
				  "address_type": 0
				},
				"shippable_item_ids": [
				  20285271157
				]
			  }
			},
			"logistic_service_types": {
			  "instant": {
				"channel_ids": [
				  8000
				],
				"enabled": false,
				"identifier": "instant",
				"max_cost": 0,
				"min_cost": 0,
				"name": "Instant",
				"priority": 1,
				"sla_msg": "Diterima 3 jam setelah paket diserahkan ke kurir"
			  },
			  "next_day": {
				"channel_ids": [
				  8002
				],
				"enabled": true,
				"identifier": "next_day",
				"max_cost": 3700000000,
				"min_cost": 3700000000,
				"name": "Next Day",
				"priority": 3,
				"sla_msg": "Diterima 1 hari setelah paket diserahkan ke kurir"
			  },
			  "regular": {
				"channel_ids": [
				  8003,
				  8005
				],
				"enabled": true,
				"identifier": "regular",
				"max_cost": 1600000000,
				"min_cost": 1500000000,
				"name": "Reguler",
				"priority": 4,
				"sla_msg": "Diterima 2-7 hari setelah paket diserahkan ke kurir"
			  },
			  "regular_cargo": {
				"channel_ids": [
				  8006
				],
				"enabled": false,
				"identifier": "regular_cargo",
				"max_cost": 0,
				"min_cost": 0,
				"name": "Hemat",
				"priority": 9,
				"sla_msg": "Diterima 2-7 hari setelah paket diserahkan ke kurir"
			  },
			  "same_day": {
				"channel_ids": [
				  8001
				],
				"enabled": false,
				"identifier": "same_day",
				"max_cost": 0,
				"min_cost": 0,
				"name": "",
				"priority": 2,
				"sla_msg": "Diterima 6 jam setelah paket diserahkan ke kurir"
			  }
			}
		  },
		  "order_total": 4000000000,
		  "order_total_without_shipping": 2400000000,
		  "selected_logistic_channelid_with_warning": null,
		  "shipping_fee": 1600000000,
		  "shipping_fee_discount": 0,
		  "shipping_group_description": "",
		  "shipping_group_icon": "",
		  "tax_payable": 0,
		  "is_fsv_applied": false,
		  "shipping_discount_type": 0,
		  "prescription_info": {
			"images": [],
			"required": false,
			"max_allowed_images": 5
		  },
		  "import_tax_amount": 0,
		  "icms_amount": 0,
		  "iof_amount": 0,
		  "is_ros_eligible": null,
		  "authorize_to_leave": 0
		}
	  ],
	  "fsv_selection_infos": [
		{
		  "fsv_id": 783739022934016,
		  "selected_shipping_ids": [
			1
		  ],
		  "potentially_applied_shipping_ids": [
			1
		  ]
		}
	  ],
	  "buyer_info": {
		"kyc_info": null,
		"checkout_email": "",
		"spl_activation_status": 1,
		"authorize_to_leave_preference": 0
	  },
	  "client_event_info": {
		"is_platform_voucher_changed": false,
		"is_fsv_changed": false,
		"recommend_payment_preselect_type": 0,
		"recommend_shipping_preselect": false
	  },
	  "buyer_txn_fee_info": {
		"title": "Biaya Penanganan",
		"description": "Biaya penanganan untuk transaksi ini adalah Rp0. Dapatkan bebas biaya penanganan dengan menggunakan metode pembayaran ShopeePay & SeaBank.",
		"learn_more_url": ""
	  },
	  "disabled_checkout_info": {},
	  "can_checkout": true,
	  "buyer_service_fee_info": {
		"learn_more_url": "https://shopee.co.id/m/biaya-layanan"
	  },
	  "iof_info": {},
	  "add_to_cart_info": {
		"is_added_to_cart": false
	  },
	  "ignored_errors": [
		0
	  ],
	  "ignore_warnings": false,
	  "captcha_version": 1,
	  "captcha_signature": "",
	  "device_info": {
		"device_id": "H1rF3LgiunFW1joqQtg1BnwzU7b4CTZCD380ATCK1Vk=",
		"device_fingerprint": "6738c5e2e8346dc3_unknow",
		"device_sz_fingerprint": "jari",
		"tongdun_blackbox": "td_disable_for_ID",
		"buyer_payment_info": {
		  "is_jko_app_installed": false
		},
		"gps_location_info": {}
	  },
	  "device_type": "mobile",
	  "_cft": [
		469696383
	  ]
	});
    // Convert struct to JSON
    let body_str = serde_json::to_string(&body_json).unwrap();
	let body = Body::from(body_str.clone());
	println!("{:?}", body_str);
	println!("Request Headers:\n{:?}", headers);

	let client = Client::new();
	let url2 = format!("https://mall.shopee.co.id/api/v4/checkout/place_order");
	println!("{}", url2);
	let mut request = http::Request::builder()
		.method("POST")
		.uri(&url2);
		
	for (name, value) in headers.iter() {
		request = request.header(name.clone(), value.clone());
	}

	let request = request.body(body)
		.unwrap();

	let result = client.send(request);

	println!("Status: get_courier");
    // Handle response as needed
	//println!("Request Headers:\n{:?}", headers);
    match result {
        Ok(response) => {
			println!("Status: {}", response.status());
			let body = response.body().as_bytes().expect("REASON").to_vec();
			println!("Body: {}", String::from_utf8_lossy(&body));
        }
        Err(error) => {
            println!("Error: {}", error);
        }
    } 
	Ok(())
}
*/
pub async fn checkout_get(cookie_content: &str, shop_id_str: &str, item_id_str: &str, addressid_str: &str, quantity_str: &str, chosen_model: &ModelInfo, chosen_payment: &PaymentInfo, chosen_shipping: &ShippingInfo, opt: bool) -> Result<(), Box<dyn std::error::Error>> {
	let headers = headers_checkout(&cookie_content);
	let shop_id = shop_id_str.parse::<i64>().expect("Failed to parse shop_id");
	let addressid = addressid_str.parse::<i64>().expect("Failed to parse addressid");
	let item_id = item_id_str.parse::<i64>().expect("Failed to parse item_id");
	let quantity = quantity_str.parse::<i64>().expect("Failed to parse quantity");
	let channel_id: u64 = chosen_payment.channel_id.parse().expect("Failed to parse channel_id");
	let version: u64 = chosen_payment.version.parse().expect("Failed to parse version");
	let optioninfo: String = chosen_payment.option_info.clone();
	let current_time = Utc::now();

// pre voucher


    let platform_vouchers = if opt == true{
        json!([{
			"voucher_code": "DC50100RB01011",
			"promotionid": 786674414714880 as i64
		  }])
    } else {
        json!([])
    };

	let body_json = json!({
	  "timestamp": current_time.timestamp(),
	  "shoporders": [
		{
		  "shop": {
			"shopid": shop_id
		  },
		  "items": [
			{
			  "itemid": item_id as i64,
			  "modelid": chosen_model.modelid as i64,
			  "quantity": quantity as i64,
			  "add_on_deal_id": 0,
			  "is_add_on_sub_item": false,
			  "item_group_id": null,
			  "insurances": [],
			  "channel_exclusive_info": {
				"source_id": 0,
				"token": "",
				"is_live_stream": false,
				"is_short_video": false
			  },
			  "supports_free_returns": false
			}
		  ],
		  "shipping_id": 1
		}
	  ],
	  "selected_payment_channel_data": {
		"page": "OPC_PAYMENT_SELECTION",
		"removed_vouchers": [],
		"channel_id": channel_id,
		"version": version,
		"group_id": 0,
		"channel_item_option_info": {
		  "option_info": optioninfo
		},
		"additional_info": {}
	  },
	  "promotion_data": {
		"use_coins": false,
		"free_shipping_voucher_info": {
		  "free_shipping_voucher_id": 783739022934016 as i64,
		  "free_shipping_voucher_code": "FSV-783739022934016",
		  "disabled_reason": null,
		  "disabled_reason_code": 0,
		  "banner_info": {
			"banner_type": 5,
			"learn_more_msg": "",
			"msg": "Berhasil mendapatkan Gratis Ongkir"
		  },
		  "required_be_channel_ids": [],
		  "required_spm_channels": []
		},
		"platform_vouchers": platform_vouchers,
		"shop_vouchers": [],
		"check_shop_voucher_entrances": true,
		"auto_apply_shop_voucher": false
	  },
	  "fsv_selection_infos": [
		{
		  "fsv_id": 783739022934016 as i64,
		  "selected_shipping_ids": [
			1
		  ],
		  "potentially_applied_shipping_ids": [
			1
		  ]
		}
	  ],
	  "device_info": {
		"device_id": "H1rF3LgiunFW1joqQtg1BnwzU7b4CTZCD380ATCK1Vk=",
		"device_fingerprint": "6738c5e2e8346dc3_unknow",
		"device_sz_fingerprint": "toV8/41L1Kp+vfBjcpEu3g==|0RUg667kZQ7oxtO4C61gcj9W3E0dMmsmsh9XhMNGoZDjUnI+EhnjwFy9k9HzhTrbwfzS7/eyvqxsDYbnzUm7cg==|zZ8vHxy7GGdEBtT1|08|1",
		"tongdun_blackbox": "td_disable_for_ID",
		"buyer_payment_info": {
		  "is_jko_app_installed": false
		},
		"gps_location_info": {}
	  },
	  "buyer_info": {
		"kyc_info": null,
		"checkout_email": ""
	  },
	  "cart_type": 1,
	  "client_id": 5,
	  "tax_info": {
		"tax_id": ""
	  },
	  "client_event_info": {},
	  "add_to_cart_info": {},
	  "_cft": [469696383],
	  "dropshipping_info": {},
	  "shipping_orders": [
		{
		  "sync": true,
		  "buyer_address_data": {
			"addressid": addressid as i64,
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
    // Convert struct to JSON
    let body_str = serde_json::to_string(&body_json).unwrap();
	let body = Body::from(body_str.clone());
	println!("{:?}", body_str);
	//println!("Request Headers:\n{:?}", headers);

	let url2 = format!("https://mall.shopee.co.id/api/v4/checkout/get");
	println!("{}", url2);
	// Buat klien HTTP
	let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .impersonate(Impersonate::Cronet)
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

	println!("Status: get_courier");
    // Handle response as needed
	//println!("Request Headers:\n{:?}", headers);
	println!("Status: {}", response.status());
	let body = response.text().await?;
	println!("Body: {}", body);
	Ok(())
}

fn headers_checkout(cookie_content: &str) -> HeaderMap {
    let csrftoken = extract_csrftoken(&cookie_content);
    let mut headers = reqwest::header::HeaderMap::new();
	headers.insert("x-api-source", HeaderValue::from_static("rn"));
	headers.insert("x-shopee-client-timezone", HeaderValue::from_static("Asia/Jakarta"));
	headers.insert("x-sap-access-f", HeaderValue::from_static(" "));
	headers.insert("x-requested-with", HeaderValue::from_static("XMLHttpRequest"));
	headers.insert("x-sap-access-t", HeaderValue::from_static(" "));
	headers.insert("af-ac-enc-dat", HeaderValue::from_static(" "));
	headers.insert("af-ac-enc-id", HeaderValue::from_static(" "));
	headers.insert("af-ac-enc-sz-token", HeaderValue::from_static(" "));
	headers.insert("if-none-match-", HeaderValue::from_static("55b03-97d86fe6888b54a9c5bfa268cf3d922d"));
	headers.insert("shopee_http_dns_mode", HeaderValue::from_static("1"));
	headers.insert("x-sap-access-s", HeaderValue::from_static(" "));
	headers.insert("x-csrftoken", HeaderValue::from_str(csrftoken).unwrap());
	headers.insert("user-agent", HeaderValue::from_static("Android app Shopee appver=31215 app_type=1"));
	headers.insert("referer", HeaderValue::from_static("https://mall.shopee.co.id/bridge_cmd?cmd=reactPath%3Ftab%3Dbuy%26path%3Dshopee%252FHOME_PAGE%253Fis_tab%253Dtrue%2526layout%253D%25255Bobject%252520Object%25255D%2526native_render%253Dsearch_prefills%25252Clanding_page_banners%25252Cwallet_bar%25252Cannouncement%25252Chome_squares%25252Cskinny_banners%25252Cnew_user_zone%25252Cearly_life_zone%25252Ccampaign_modules%25252Cflash_sales%25252Clive_streaming%25252Cvideo%25252Cdigital_products%25252Cdeals_nearby%25252Ccutline%25252Cdaily_discover%25252Cfood_order_status"));
	headers.insert("accept", HeaderValue::from_static("application/json"));
	headers.insert("content-type", HeaderValue::from_static("application/json"));
	headers.insert("cookie", HeaderValue::from_str(cookie_content).unwrap());
    // Return the created headers
    headers
}

fn extract_csrftoken(cookie_string: &str) -> &str {
    let mut csrftoken = "";

    if let Some(token_index) = cookie_string.find("csrftoken=") {
        let token_start = token_index + "csrftoken=".len();
        if let Some(token_end) = cookie_string[token_start..].find(';') {
            csrftoken = &cookie_string[token_start..token_start + token_end];
        }
    }

    csrftoken
}