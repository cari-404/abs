use crate::prepare::{ModelInfo, ShippingInfo, PaymentInfo, ProductInfo, AddressInfo};
use crate::voucher::Vouchers;
use crate::crypt::{self, DeviceInfo};
use crate::task::{self, GetBodyJson, PlaceOrderBody, FreeShippingVoucherInfo, BannerInfo, PromotionData, ShopVoucher, PlatformVoucher, FsvSelectionInfo, BuyerInfo, TaxInfo, ClientEventInfo, AddToCartInfo, DropshippingInfo, OrderUpdateInfo};

pub async fn get_body_builder(device_info: &DeviceInfo, 
    product_info: &ProductInfo, 
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
    let shop_id = product_info.shop_id;

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

pub async fn get_multi_body_builder(device_info: &DeviceInfo, 
    product_info: &ProductInfo, 
    chosen_payment: &PaymentInfo, 
    freeshipping_voucher: Arc<Option<Vouchers>>, 
    platform_vouchers_target: Arc<Option<Vouchers>>, shop_vouchers_target: Arc<Option<Vouchers>>, use_coins: bool,
    place_order: &mut PlaceOrderBody) -> Result<(GetBodyJson, PlaceOrderBody), Box<dyn std::error::Error>> {
	let current_time = Utc::now();
    let timestamp_millis = current_time.timestamp_millis();
    let timestamp_specific = format!("{:.16}", current_time.nanosecond() a====fcdfggdfghfhDSDSSSSSs f64 / 1_000_000_000.0);
    let checkout_session_id = format!(
        "{}:{}:{}{}",
        device_info.device_id, timestamp_millis, timestamp_millis, timestamp_specific
    );
    let shop_id = product_info.shop_id;

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