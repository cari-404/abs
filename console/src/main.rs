/*This Is a Auto Buy Shopee
Whats new in 1.1.7 :
    add new features to multi
    -bypass, dump, and bypass breaker
Whats new in 1.1.6 :
    add bypass breaker
Whats new in 1.1.5 :
    add logging for request and response for diagnostic
*/
use runtime::prepare::{self, ModelInfo, ShippingInfo, PaymentInfo};
use runtime::task_ng::{SelectedGet, SelectedPlaceOrder, ChannelItemOptionInfo};
use runtime::task::{self};
use runtime::task_ng::{self};
use runtime::voucher::{self};
use runtime::crypt::{self};
use runtime::telegram::{self};
use runtime::telemetry;
use chrono::{Local, Duration, NaiveDateTime, DateTime, Timelike, Utc};
use std::io::{self, Write, Read};
use std::process;
use anyhow::Result;
use std::fs::File;
use std::borrow::Cow;
use structopt::StructOpt;
use num_cpus;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

mod collective;

#[derive(Debug, StructOpt)]
#[structopt(name = "Auto Buy Shopee", about = "Make fast buy from shopee.co.id")]
struct Opt {
    #[structopt(short, long, help = "URL product")]
    url: Option<String>,  
    #[structopt(short, long, help = "selected file cookie")]
    file: Option<String>,    
    #[structopt(short, long, help = "time to run checkout")]
    time: Option<String>,    
    #[structopt(long, help = "Select Product")]
    product: Option<String>,    
    #[structopt(short, long, help = "Select Courier")]
    kurir: Option<String>,    
    #[structopt(long, help = "Select Payment Method")]
    payment: Option<String>,    
	#[structopt(short, long, help = "Set Harga MAX")]
    harga: Option<String>,	
	#[structopt(short, long, help = "Set quantity")]
    quantity: Option<i32>,
	#[structopt(short, long, help = "Set token media")]
    token: Option<String>,
	
	/*#[structopt(short, long, help = "Apply token media")]
    media: bool,*/ // Confused?
	#[structopt(short, long, help = "Apply freeshipping voucher only")]
    fsv_only: bool,
    #[structopt(short, long, help = "No freeshipping voucher")]
    no_fsv: bool,
	#[structopt(short, long, help = "Apply platform Voucher")]
    platform_vouchers: bool,
	#[structopt(short, long, help = "Apply shop Voucher(test)")]
    shop_vouchers: bool,
	#[structopt(short, long, help = "Apply voucher from collections")]
    collection_vouchers: bool,
	#[structopt(short, long, help = "Apply Platform Voucher klaim(claim_platform_voucher) enable pro_id&sign ")]
    claim_platform_vouchers: bool,
	#[structopt(short, long, help = "Alternative Platform Voucher without claim Required pro_id&sign")]
    no_claim_platform_vouchers: bool,
	#[structopt(short, long, help = "Set Platform kode voucher")]
    code_platform: Option<String>,
	#[structopt(short, long, help = "Set shop kode Voucher")]
    code_shop: Option<String>,
	#[structopt(short, long, help = "Set promotionid(need claim_platform_vouchers)")]
    pro_id: Option<String>,
	#[structopt(short, long, help = "Set signature(need claim_platform_vouchers)")]
    sign: Option<String>,
	#[structopt(short, long, help = "Set Voucher from collection_id")]
    collectionid: Option<String>,
    #[structopt(short, long, help = "Set Custom Threads")]
    job: Option<String>,
    #[structopt(short, long, help = "Set No Coins used")]
    no_coins: bool,
    #[structopt(short, long, help = "Test mode")]
    test: bool,
    #[structopt(short, long, help = "Collect logs")]
    dump: bool,
    #[structopt(short, long, help = "Bypass mode(Calculate Without server).!!!UNSTABLE!!!")]
    bypass: bool,
}

#[cfg(windows)]
fn clear_screen() {
    use std::process::Command;
    // Use the 'cls' command to clear the screen on Windows
    if Command::new("cmd")
        .args(&["/c", "cls"])
        .status()
        .expect("Failed to execute command")
        .success()
    {
        // Clearing was successful
    } else {
        // Handle the case where clearing the screen failed
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
    }
}
#[cfg(not(windows))]
fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

async fn heading_app(promotionid: &str, signature: &str, voucher_code_platform: &str, voucher_code_shop: &str, voucher_collectionid: &str, opt: &Opt, target_url: &str, task_time_str: &str, selected_file: &str, username: &str, max_price: &str, chosen_model: &ModelInfo, chosen_shipping: &ShippingInfo, chosen_payment: &PaymentInfo) {
    let padding = 15;
    let version_info = env!("CARGO_PKG_VERSION");
	println!("---------------------------------------------------------------");
    println!("              Auto Buy Shopee [Version {}]              ", version_info);
    println!("{:<padding$}: {}", "Cookie file", selected_file, padding = padding);
    println!("{:<padding$}: {}", "Username", username, padding = padding);
    println!("{:<padding$}: {}", "URL", target_url, padding = padding);
    println!("{:<padding$}: {}", "Time", task_time_str, padding = padding);
    println!("{:<padding$}: {}", "Product", chosen_model.product_name, padding = padding);
    println!("{:<padding$}: {}", "Variant", chosen_model.name, padding = padding);
    println!("{:<padding$}: {}", "Model Id", chosen_model.modelid, padding = padding);
    println!("{:<padding$}: {}", "Kurir", chosen_shipping.channel_name, padding = padding);
    if !max_price.is_empty() {
        println!("{:<padding$}: {} {}", "Max Price", max_price, if opt.bypass {"(Bypass)"} else { "" }, padding = padding);
    }
    println!("{:<padding$}: {}", "Payment", chosen_payment.name, padding = padding);
    if opt.claim_platform_vouchers {
        println!("{:<padding$}: {}", "Mode", "Klaim Platform Voucher", padding = padding);
        println!("{:<padding$}: {}", "Promotion_Id", opt.pro_id.as_deref().unwrap_or(promotionid), padding = padding);
        println!("{:<padding$}: {}", "Signature", opt.sign.as_deref().unwrap_or(signature), padding = padding);
    } else if opt.platform_vouchers {
        println!("{:<padding$}: {}", "Mode", "Code Platform Voucher", padding = padding);
        println!("{:<padding$}: {}", "Code", opt.code_platform.as_deref().unwrap_or(voucher_code_platform), padding = padding);
    } else if opt.collection_vouchers {
        println!("{:<padding$}: {}", "Mode", "Voucher Collection", padding = padding);
        println!("{:<padding$}: {}", "Collection", opt.collectionid.as_deref().unwrap_or(voucher_collectionid), padding = padding);
    } 
    if opt.shop_vouchers {
        println!("{:<padding$}: {}", "Mode", "Code Shop Voucher", padding = padding);
        println!("{:<padding$}: {}", "Code", opt.code_shop.as_deref().unwrap_or(voucher_code_shop), padding = padding);
    }
    println!("---------------------------------------------------------------");
    println!("");
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(prepare::universal_client_skip_headers().await);
    let mut chosen_model = ModelInfo {
        name: String::from("NOT SET"),
        product_name: String::from("NOT SET"),
        price: 0,
        stock: 0,
        modelid: 0,
        promotionid: 0,
        shop_id: 0,
        item_id: 0,
        quantity: 0,
        voucher_code: None,
    };
    let mut chosen_shipping = ShippingInfo::default();
    let mut chosen_payment = PaymentInfo {
		name: String::from("NOT SET"),
		channel_id: 0,
		option_info: String::from("NOT SET"),
		version: 0,
		txn_fee: 0,
        selected_get: serde_json::Value::Null,
        place_order: serde_json::Value::Null,
	};
	let version_info = env!("CARGO_PKG_VERSION");
	let opt = Opt::from_args();
    let max_threads = {
        if let Some(job_str) = &opt.job {
            match job_str.parse::<usize>() { 
                Ok(value) => value,
                Err(_) => {
                    eprintln!("Invalid thread count '{}'. Using default.", job_str);
                    if num_cpus::get() > 4 {
                        num_cpus::get()
                    } else {
                        4
                    }
                }
            }
        } else {
            if num_cpus::get() > 4 {
                num_cpus::get()
            } else {
                4
            }
        }
    };
    let use_coins = if opt.no_coins{
        false
    }else{
        true
    };
    let config = match telegram::open_config_file().await {
        Ok(config_content) => {
            match telegram::get_config(&config_content).await {
                Ok(tele_info) => tele_info,
                Err(e) => {
                    eprintln!("Failed to parse config file: {}. Using default data.", e);
                    telegram::get_data("a", "a")
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to open config file: {}. Using default data.", e);
            telegram::get_data("a", "a")
        }
    };
    println!("Telegram Notification data: {:?}", config);
    let telmet = if opt.dump {
        telemetry::Telemetry::new(env!("CARGO_PKG_NAME"))
    } else {
        telemetry::Telemetry::default()
    };
    telmet.write(&format!("Auto Buy Shopee [Version {}]", version_info));
    args_checking(&opt);
    clear_screen();
    // Welcome Header
    println!("Auto Buy Shopee [Version {}]", version_info);
    println!("Logical CPUs: {}", num_cpus::get());
    if config.telegram_notif {
        println!("Telegram Notification: Enabled");
    }
    println!("");

    // Get account details
    let selected_file = opt.file.clone().unwrap_or_else(|| select_cookie_file().expect("Folder akun dan file cookie tidak ada\n"));
    let cookie_data = prepare::create_cookie(&prepare::read_cookie_file(&selected_file));
    println!("csrftoken: {}", cookie_data.csrftoken);
    let base_headers = Arc::new(prepare::create_headers(&cookie_data));

    let fp_folder = format!("./header/{}/af-ac-enc-sz-token.txt", selected_file);
	
	// Membuat folder header jika belum ada
	tokio::fs::create_dir_all(&format!("./header/{}", selected_file)).await?;

	// Membuat file header jika belum ada
	if !File::open(&fp_folder).is_ok() {
		let mut header_file = File::create(&fp_folder)?;
		// Isi file header dengan konten default atau kosong sesuai kebutuhan
		header_file.write_all(b"ganti kode ini dengan sz-token valid")?;
	}

	// Baca isi file untuk header af-ac-enc-sz-token
	let mut sz_token_content = String::new();
	File::open(&fp_folder)?.read_to_string(&mut sz_token_content)?;
	println!("sz-token:{}", sz_token_content);

    let device_info = crypt::create_devices(&sz_token_content);
	
    if opt.test {
        println!("Test mode enabled");
        //let _ = test().await;
        process::exit(1);
    }
    // Get target URL
	let mut target_url = get_or_prompt(opt.url.as_deref(), "Masukkan URL:");
    let mut product_info = prepare::process_url(&target_url.trim());
    if product_info.shop_id == 0 && product_info.item_id == 0 {
        println!("Cek apakah redirect?.");
        target_url = prepare::get_redirect_url(&target_url).await?.into();
        product_info = prepare::process_url(&target_url);
    }
	
	let mut promotionid = String::new();
	let mut signature = String::new();
	let mut voucher_code_platform = String::new();
	let mut voucher_code_shop = String::new();
	let mut voucher_collectionid = String::new();

	if opt.platform_vouchers {
		println!("voucher code platform enable");
		voucher_code_platform = opt.code_platform.clone().unwrap_or_else(|| get_user_input("Masukan voucher code platform: "));
	}
    if opt.collection_vouchers {
		println!("voucher collection enable");
		voucher_collectionid = opt.collectionid.clone().unwrap_or_else(|| get_user_input("Masukan collection_id: "));
	}
	if opt.claim_platform_vouchers {
		println!("voucher claim enable");
		promotionid = opt.pro_id.clone().unwrap_or_else(|| get_user_input("Masukan Promotion_Id: "));
		signature = opt.sign.clone().unwrap_or_else(|| get_user_input("Masukan Signature: "));	
	}
	if !promotionid.is_empty() && !signature.is_empty(){
		println!("promotionid: {}", promotionid);
		println!("signature: {}", signature);		
	}

    // Get task_time from user input
	let mut task_time_str = get_or_prompt(opt.time.as_deref(), "Enter task time (HH:MM:SS.NNNNNNNNN): ");
    if task_time_str.trim().is_empty() {
        println!("Task time is empty, using default time.");
        let local: DateTime<Local> = Local::now();
        let hour = local.hour();
        let rounded_minute = match local.minute() {
            m if m <= 14 => 14,
            m if m <= 29 => 29,
            m if m <= 44 => 44,
            _ => 59,
        };
        task_time_str = format!("{:02}:{:02}:59.900", hour, rounded_minute).into();
    }
    let task_time_dt = parse_task_time(&task_time_str)?;
    telmet.write(&format!("Url: {}", target_url));
    telmet.write(&format!("Task time set to: {}", task_time_dt));

    clear_screen();
    heading_app(&promotionid, &signature, &voucher_code_platform, &voucher_code_shop, &voucher_collectionid, &opt, &target_url, &task_time_str, &selected_file, "", "", &chosen_model, &chosen_shipping, &chosen_payment).await;
    // Perform the main task
    let (info_result, address_result, product_result) = tokio::join!(
        prepare::info_akun(client.clone(), base_headers.clone()),
        prepare::address(client.clone(), base_headers.clone()),
        prepare::get_product(client.clone(), &product_info, &cookie_data)
    );
    let userdata = info_result?;
    let address_info = address_result?;
    let (name, model_info, is_official_shop, fs_info, status_code) = product_result?;
	println!("Username  : {}", userdata.username);
	println!("Email     : {}", userdata.email);
	println!("Phone     : {}", userdata.phone);
	println!("State     : {}", address_info.state);
	println!("City      : {}", address_info.city);
	println!("District  : {}", address_info.district);
    telmet.write(&format!(
        "User info: username={}, email={}, phone={}, state={}, city={}, district={}",
        userdata.username,
        userdata.email,
        userdata.phone,
        address_info.state,
        address_info.city,
        address_info.district
    ));
	//std::thread::sleep(std::time::Duration::from_secs(2));
	println!("shop_id: {}", product_info.shop_id);
    println!("item_id: {}", product_info.item_id);

    if status_code != "200 OK"{
        println!("Status: {}", status_code);
        println!("Harap Ganti akun");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Gagal membaca baris");
        process::exit(1);
    }
    clear_screen();
    heading_app(&promotionid, &signature, &voucher_code_platform, &voucher_code_shop, &voucher_collectionid, &opt, &target_url, &task_time_str, &selected_file, &userdata.username, "", &chosen_model, &chosen_shipping, &chosen_payment).await;
    println!("addressid  : {}", address_info.id);
	println!("name             : {}", name);
    // println!("models           : \n{:#?}", model_info);
    println!("Official Shop ?  : {}", is_official_shop);
    let fs_item = if fs_info.promotionid != 0 {
        println!("promotionid  : {}", fs_info.promotionid);
        println!("start_time   : {}", human_readable_time(fs_info.start_time));
        println!("end_time     : {}", human_readable_time(fs_info.end_time));
        let fs_items = prepare::get_flash_sale_batch_get_items(client.clone(), &cookie_data, &[product_info.clone()], &fs_info).await?;
        fs_items
    }else {
        Vec::new()
    };
	
	//std::thread::sleep(std::time::Duration::from_secs(2));
    if let Some(mut model) = collective::choose_model(&model_info, &opt, &fs_item) {
        let quantity = opt.quantity.unwrap_or_else(|| {
            loop {
                let input = get_user_input("Kuantiti: ");
                match input.parse::<i32>() {
                    Ok(value) => break value, // Kembalikan nilai yang valid
                    Err(_) => {
                        println!("Input tidak valid, coba lagi.");
                    }
                }
            }
        });
        model.quantity = quantity;
        if opt.shop_vouchers {
            println!("voucher code shop enable");
            voucher_code_shop = opt.code_shop.clone().unwrap_or_else(|| get_user_input("Masukan voucher code shop: "));
            if !voucher_code_shop.trim().is_empty() {
                model.voucher_code = Some(voucher_code_shop.clone());
            }
        }
        chosen_model = model;
        println!("Anda memilih model: {:#?}", chosen_model);
        // Lanjutkan dengan logika berikutnya
    } else {
        println!("Model tidak valid.");
		process::exit(1);
        // Handle jika model tidak valid
    }
    let quantity = chosen_model.quantity;

    println!("Anda memilih model: {}", chosen_model.name);
    let shared_headers = Arc::new(task::headers_checkout(&cookie_data));
    let shipping_info = runtime::prepare_ext::get_shipping_data(client.clone(), base_headers.clone(), shared_headers.clone(), &device_info, None, &address_info, &chosen_model, &chosen_payment, &chosen_shipping).await?;

    clear_screen();
    heading_app(&promotionid, &signature, &voucher_code_platform, &voucher_code_shop, &voucher_collectionid, &opt, &target_url, &task_time_str, &selected_file, &userdata.username, "",&chosen_model, &chosen_shipping, &chosen_payment).await;

	if let Some(shipping) = collective::choose_shipping(&shipping_info, &opt) {
		chosen_shipping = shipping;
		println!("Anda memilih shipping: {:#?}", chosen_shipping);
		// Continue with the next logic
	} else {
		println!("shipping tidak valid.");
		process::exit(1);
		// Handle if the shipping is not valid
	}
	println!("{:?}", chosen_shipping);
	clear_screen();
	heading_app(&promotionid, &signature, &voucher_code_platform, &voucher_code_shop, &voucher_collectionid, &opt, &target_url, &task_time_str, &selected_file, &userdata.username, "", &chosen_model, &chosen_shipping, &chosen_payment).await;
	let max_price = opt.harga.clone().unwrap_or_else(|| get_user_input("Harga MAX:")).trim().to_string();
    let token = get_or_prompt(opt.token.as_deref(), "Token Media: ");
	
	let payment_info = prepare::get_payment(&prepare::open_payment_file().await?)?;

	if let Some(payment) = collective::choose_payment(&payment_info, &opt) {
		chosen_payment = payment;
		println!("Anda memilih payment: {:#?}", chosen_payment);
		// Continue with the next logic
	} else {
		println!("payment tidak valid.");
		process::exit(1);
		// Handle if the payment is not valid
	}

    if chosen_payment.selected_get.is_null() {
        chosen_payment.selected_get = serde_json::to_value(SelectedGet {
            page: "OPC_PAYMENT_SELECTION".to_string(),
            removed_vouchers: vec![],
            channel_id: chosen_payment.channel_id,
            version: chosen_payment.version,
            group_id: 0,
            channel_item_option_info: ChannelItemOptionInfo {
                option_info: chosen_payment.option_info.clone(),
            },
            additional_info: json!({}),
        })
        .unwrap();
    };
    if chosen_payment.place_order.is_null(){
        chosen_payment.place_order = serde_json::to_value(SelectedPlaceOrder {
            channel_id: chosen_payment.channel_id,
            channel_item_option_info: ChannelItemOptionInfo {
                option_info: chosen_payment.option_info.clone(),
            },
            version: chosen_payment.version,
        })
        .unwrap();
    };
    let adjusted_max_price = if !max_price.is_empty() {
        match max_price.replace(",", "").trim().parse::<i64>() {
            Ok(val) => Some(val * 100_000),
            Err(_) => {
                println!("Gagal mengonversi max_price menjadi angka.");
                None
            }
        }
    } else {
        None
    };

	println!("{:?}", chosen_payment);
	clear_screen();
	heading_app(&promotionid, &signature, &voucher_code_platform, &voucher_code_shop, &voucher_collectionid, &opt, &target_url, &task_time_str, &selected_file, &userdata.username, &max_price, &chosen_model, &chosen_shipping, &chosen_payment).await;
    println!("{:?}", chosen_payment);
    let vc_header = Arc::new(voucher::headers_checkout(&cookie_data));
    let cookie_data = Arc::new(cookie_data);
    let device_info = Arc::new(device_info);
    let product_info = Arc::new(product_info);
    let address_info = Arc::new(address_info);
    let chosen_payment = Arc::new(chosen_payment);
    let chosen_shipping = Arc::new(chosen_shipping);
    let mut temp = task_ng::get_builder(client.clone(), shared_headers.clone(), &device_info, &address_info, &[chosen_model.clone()], &chosen_payment, &chosen_shipping, &None, &None, &None, use_coins).await?;
    task_ng::force_deselect_insurance(&mut temp.shoporders);
    let chosen_model = Arc::new(chosen_model);
    println!("asuransi: {:?}", temp.shoporders);
    let mut selected_platform_voucher = None;
    /*if opt.bypass {
        println!("Bypass mode enabled. Skipping voucher claim and checkout.");
        println!("You can use this mode to calculate without server interaction.");
        println!("Please ensure you have the correct data for the calculation.");
        if let Some(ref checkout_price_data) = temp.checkout_price_data {
            let detail_payment = task::CheckoutPriceData::parse_checkout_price_data(checkout_price_data);
            match detail_payment {
                Ok(detail_payment) => {
                    println!("Checkout Price Data: {:#?}", detail_payment);
                    println!("Subtotal Pesanan: {}", detail_payment.merchandise_subtotal / 100_000);
                    println!("Total Proteksi Produk: {}", detail_payment.insurance_subtotal / 100_000);
                    println!("Subtotal Pengiriman: {}", detail_payment.shipping_subtotal_before_discount / 100_000);
                    println!("Biaya Layanan: {}", detail_payment.buyer_service_fee / 100_000);
                    if detail_payment.shipping_discount_subtotal > 0 {
                        println!("Total Diskon Pengiriman: -{}", detail_payment.shipping_discount_subtotal / 100_000);
                    }
                    if detail_payment.promocode_applied.is_some() {
                        println!("Voucher Diskon: -{}", detail_payment.promocode_applied.as_ref().and_then(|v| v.as_i64()).map(|v| v / 100_000).unwrap_or(0));
                    }
                    if detail_payment.shopee_coins_redeemed.is_some() {
                        println!("Koin Shopee Telah Digunakan: -{}", detail_payment.shopee_coins_redeemed.as_ref().and_then(|v| v.as_i64()).map(|v| v / 100_000).unwrap_or(0));
                    }
                    println!("---------------------------------------------------------------------");
                    println!("Total Harga: {}", detail_payment.total_payable / 100_000);
                }
                Err(e) => {
                    println!("Gagal parsing Checkout Price Data: {e}");
                }
            }
        } else {
            println!("Checkout Price Data: None");
        }
        process::exit(0);
    }*/
    if opt.claim_platform_vouchers{
        if !promotionid.is_empty() && !signature.is_empty() {
            selected_platform_voucher = voucher::get_voucher_data(client.clone(), &promotionid, &signature, vc_header.clone()).await?;
        } else {
            selected_platform_voucher = None;
        }
    }
    println!("selected_platform_voucher: {:?}", selected_platform_voucher);
    countdown_to_task(&task_time_dt).await;

    if opt.claim_platform_vouchers || opt.platform_vouchers || opt.collection_vouchers || opt.fsv_only || opt.shop_vouchers {
        let selected_platform_voucher_fut = async {
            if !voucher_code_platform.is_empty() {
                selected_platform_voucher = voucher::save_platform_voucher_by_voucher_code(
                    client.clone(),
                    &voucher_code_platform,
                    vc_header.clone(),
                ).await?;
            } else if !voucher_collectionid.is_empty() {
                let (promo_id, sig) = voucher::some_function(client.clone(), &voucher_collectionid, &cookie_data).await?;
                promotionid = promo_id;
                signature = sig;
                selected_platform_voucher = voucher::save_voucher(client.clone(), &promotionid, &signature, vc_header.clone()).await?;
            } 
            Ok(())
        };
        let selected_shop_voucher_fut = async {
            if !voucher_code_shop.is_empty() {
                voucher::save_shop_voucher_by_voucher_code(
                    client.clone(),
                    &voucher_code_shop,
                    vc_header.clone(),
                    product_info.as_ref().clone(),
                ).await
            } else {
                Ok(None)
            }
        };
        let recommend_voucher_fut = async {
            if !opt.no_fsv {
                voucher::get_recommend_platform_vouchers(
                    adjusted_max_price,
                    &address_info,
                    client.clone(),
                    vc_header.clone(),
                    &product_info,
                    quantity,
                    &chosen_model,
                    &chosen_payment,
                    &chosen_shipping,
                ).await
            } else {
                Ok((None, None))
            }
        };

        let (selected_shop_voucher, (freeshipping_voucher, platform_vouchers_target),_) =
            tokio::try_join!(
                selected_shop_voucher_fut,
                recommend_voucher_fut,
                selected_platform_voucher_fut,
            )?;

        let final_voucher = if opt.fsv_only || (opt.shop_vouchers && !opt.platform_vouchers && !opt.claim_platform_vouchers && !opt.collection_vouchers) {
            None
        } else {
            selected_platform_voucher.or(platform_vouchers_target)
        };
    
        print_voucher_info("freeshipping_voucher", &freeshipping_voucher).await;
        print_voucher_info("platform_voucher", &final_voucher).await;
        print_voucher_info("shop_voucher", &selected_shop_voucher).await;

        let _claim_and_checkout = tokio::try_join!(
            async {
                if !promotionid.is_empty() && !signature.is_empty() {
                    if opt.no_claim_platform_vouchers {
                        voucher::get_voucher_data(client.clone(), &promotionid, &signature, vc_header.clone()).await
                    } else {
                        voucher::save_voucher(client.clone(), &promotionid, &signature, vc_header.clone()).await
                    }
                } else if !voucher_code_platform.is_empty() {
                    voucher::save_platform_voucher_by_voucher_code(
                        client.clone(),
                        &voucher_code_platform,
                        vc_header.clone(),
                    ).await
                } else {
                    Ok(None)
                }
            },
            async {
                let raw_checkout_data = Arc::new(task_ng::get_body_builder(&device_info, &chosen_payment, Arc::new(freeshipping_voucher), Arc::new(final_voucher), Arc::new(selected_shop_voucher.map(|v| vec![v])), use_coins, &mut temp).await?);
                let (place_order_tx, place_order_rx) = tokio::sync::watch::channel::<Option<task_ng::PlaceOrderBody>>(None);
                let (trigger_tx, mut trigger_rx) = tokio::sync::mpsc::channel::<()>(1);
                let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(max_threads);
                let stop_flag = Arc::new(AtomicBool::new(false));
                let stop_flag_trigger = Arc::clone(&stop_flag);
                // Task trigger: kirim sinyal setiap 50ms
                tokio::spawn(async move {
                    loop {
                        if stop_flag_trigger.load(Ordering::Relaxed) {
                            break;
                        }
                        if trigger_tx.send(()).await.is_err() {
                            break;
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    }
                });
                // Task refresher: jalankan get_ng setiap kali ada trigger, update ke watch
                {
                    let client = Arc::clone(&client);
                    let shared_headers = Arc::clone(&shared_headers);
                    let get_body_clone = Arc::clone(&raw_checkout_data);
                    let chosen_payment_clone = Arc::clone(&chosen_payment);
                    let place_order_tx = place_order_tx.clone();
                    let stop_flag_refresher = Arc::clone(&stop_flag);
                    let bypass_breaker = Arc::new(AtomicBool::new(false));

                    tokio::spawn(async move {
                        while trigger_rx.recv().await.is_some() {
                            if stop_flag_refresher.load(Ordering::Relaxed) {
                                break;
                            }
                            if bypass_breaker.load(Ordering::Relaxed) {
                                break;
                            }
                            let client = Arc::clone(&client);
                            let headers = Arc::clone(&shared_headers);
                            let chosen_payment = Arc::clone(&chosen_payment_clone);
                            let get_body_clone = Arc::clone(&get_body_clone);
                            let bypass_breaker = Arc::clone(&bypass_breaker);

                            tokio::spawn({
                                let place_order_tx = place_order_tx.clone();
                                async move {
                                    if let Ok(mut body) = task_ng::get_ng(
                                        client,
                                        headers,
                                        &get_body_clone.0,
                                        &chosen_payment,
                                        get_body_clone.1.clone(),
                                    ).await {
                                        if let Some(limit) = adjusted_max_price {
                                            if opt.bypass {
                                                if let Some(ref mut checkout_price_data) = body.checkout_price_data {
                                                    task::recalculate_shipping_subtotal(checkout_price_data, limit);
                                                }
                                                let _ = place_order_tx.send(Some(body));
                                                bypass_breaker.store(true, Ordering::Relaxed);
                                            }else{
                                                let price_opt = body.checkout_price_data.as_ref()
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
                                                    }
                                                    Some(price) => {
                                                        println!(
                                                            "[{}]Harga cocok ({} <= {}).",
                                                            chrono::Local::now().format("%H:%M:%S.%3f"),
                                                            price, limit
                                                        );
                                                        // Lanjut proses
                                                        let _ = place_order_tx.send(Some(body));
                                                    }
                                                    None => {
                                                        println!(
                                                            "[{}]Gagal membaca merchandise_subtotal, ulangi...",
                                                            chrono::Local::now().format("%H:%M:%S.%3f")
                                                        );
                                                        // Retry
                                                    }
                                                }
                                            }
                                        } else {
                                            let _ = place_order_tx.send(Some(body));
                                        }
                                    }
                                }
                            });
                        }
                    });
                }
                for i in 0..max_threads {
                    println!("Running on thread: {}", i);
                    let tx = tx.clone();
                    let shared_headers = Arc::clone(&shared_headers);
                    let client = Arc::clone(&client);
                    let mut place_order_rx = place_order_rx.clone();
                    let stop_flag = Arc::clone(&stop_flag);
                    let telmet = telmet.clone();
            
                    tokio::spawn(async move {
                        let mut try_count = 0;
                        while try_count < 3 && !stop_flag.load(Ordering::Relaxed) {
                            place_order_rx.changed().await.ok();
                            let place_order_body = match *place_order_rx.borrow() {
                                Some(ref body) => body.clone(),
                                None => continue,
                            };
                            if stop_flag.load(Ordering::Relaxed) {
                                return;
                            }
                            telmet.write(&format!("Request PlaceOrder on {} try {} : {:#?}", i, try_count, place_order_body));
                            let mpp = match task_ng::place_order_ng(client.clone(), shared_headers.clone(), &place_order_body).await
                            {
                                Ok(response) => response,
                                Err(err) => {
                                    eprintln!("Error in place_order_ng: {:?}", err);
                                    continue;
                                }
                            };
                            telmet.write(&format!("Response PlaceOrder on {} try {}: {:#?}", i, try_count, mpp));
                            // Mengecek apakah `mpp` memiliki field `checkoutid`
                            println!("Current time: {}", Local::now().format("%H:%M:%S.%3f"));
                            if let Some(error) = mpp.get("error") {
                                match error.as_str().unwrap_or_default() {
                                    "error_fraud" => {
                                        println!("[{}]Gagal: error_fraud", Local::now().format("%H:%M:%S.%3f"));
                                        stop_flag.store(true, Ordering::Relaxed);
                                        return;
                                    }
                                    "error_freeze" => {
                                        println!("[{}]Gagal: error_freeze", Local::now().format("%H:%M:%S.%3f"));
                                        stop_flag.store(true, Ordering::Relaxed);
                                        return;
                                    }
                                    _ => {}
                                }
                            }
                            if let Some(checkout_id) = mpp.get("checkoutid") {
                                let checkout_id = checkout_id.as_i64().unwrap();
                                let url = format!("https://shopee.co.id/mpp/{}?flow_source=3", checkout_id);
                                println!("[{}]{}", Local::now().format("%H:%M:%S.%3f"), url);
                                let _ = tx.send(url).await;
                                stop_flag.store(true, Ordering::Relaxed);
                                return;
                            }
                            try_count += 1;
                        }
                        eprintln!("[{}]Gagal 3x percobaan", Local::now().format("%H:%M:%S.%3f"));
                        stop_flag.store(true, Ordering::Relaxed);
                    });
                }
                drop(tx); // Tutup pengirim setelah semua tugas selesai
                while let Some(url) = rx.recv().await {
                    println!("[{}]{}", Local::now().format("%H:%M:%S.%3f"), url);
                    if config.telegram_notif {
                        let msg = format!("Auto Buy Shopee {}\nREPORT!!!\nUsername     : {}\nProduct      : {}\nVariant      : {}\nLink Payment : {}\nCheckout berhasil!", version_info, userdata.username, name, chosen_model.name, url);
                        telegram::send_msg(client.clone(), &config, &msg).await?;
                    }
                }
                Ok(())
            }
        )?;
    } else if !token.is_empty(){
        loop{
            // Loop untuk menyesuaikan `merchandise_subtotal`
            let checkout_price_data;
            let order_update_info;
            let dropshipping_info;
            let promotion_data;
            let shoporders;
            let shipping_orders;
            let display_meta_data;
            let fsv_selection_infos;
            let buyer_info;
            let client_event_info;
            let buyer_txn_fee_info;
            let disabled_checkout_info;
            let buyer_service_fee_info;
            let iof_info;
            loop{
                let get_body = task::get_wtoken_builder(&token, &device_info, &product_info, &address_info, quantity, &chosen_model, &chosen_payment, &chosen_shipping).await?;
                let (
                    price_data, update_info, dropship_info, promo_data, _payment_data, 
                    orders, shipping_orders_data, meta_data, fsv_infos, buyer_info_data, 
                    event_info, txn_fee_info, disabled_info, service_fee_info, iof_data
                ) = task::checkout_get(client.clone(), shared_headers.clone(), &get_body.clone()).await?;
                // Cek apakah `merchandise_subtotal` sesuai dengan `max_price * 100000`
                if let Some(merchandise_subtotal) = price_data["merchandise_subtotal"].as_i64() {
                    println!("merchandise_subtotal: {}", merchandise_subtotal);
                    let max_price_no_comma = max_price.replace(",", "");
                    let cleaned_max_price = max_price_no_comma.trim(); 
                    if let Ok(parsed_max_price) = cleaned_max_price.parse::<i64>() {
                        let adjusted_max_price = parsed_max_price * 100_000;
                        println!("max_price (setelah dikali 100000): {}", adjusted_max_price);
                        if merchandise_subtotal <= adjusted_max_price {
                            println!("Harga merchandise_subtotal sesuai dengan max_price * 100000.");
                            // Menyimpan variabel dari loop dalam ke luar
                            checkout_price_data = price_data;
                            order_update_info = update_info;
                            dropshipping_info = dropship_info;
                            promotion_data = promo_data;
                            shoporders = orders;
                            shipping_orders = shipping_orders_data;
                            display_meta_data = meta_data;
                            fsv_selection_infos = fsv_infos;
                            buyer_info = buyer_info_data;
                            client_event_info = event_info;
                            buyer_txn_fee_info = txn_fee_info;
                            disabled_checkout_info = disabled_info;
                            buyer_service_fee_info = service_fee_info;
                            iof_info = iof_data;
                            break; // Keluar dari loop dalam jika kondisi terpenuhi
                        } else {
                            println!("Harga merchandise_subtotal lebih besar dari max_price * 100000.");
                        }
                    } else {
                        println!("Gagal mengonversi max_price menjadi angka.");
                    }
                } else {
                    println!("Gagal mendapatkan nilai merchandise_subtotal.");
                }
                println!("Harga merchandise_subtotal tidak sesuai, ulangi...");
                // Loop akan berlanjut jika kondisi tidak terpenuhi
            }
            let place_order_body = task::place_order_builder(&device_info, &checkout_price_data, &order_update_info, &dropshipping_info, &promotion_data, &chosen_payment, &shoporders, &shipping_orders, &display_meta_data, &fsv_selection_infos, &buyer_info, &client_event_info, &buyer_txn_fee_info, &disabled_checkout_info, &buyer_service_fee_info, &iof_info).await?;
            let mpp = task::place_order(client.clone(), &cookie_data, &place_order_body).await?;
            // Mengecek apakah `mpp` memiliki field `checkoutid`
            println!("Current time: {}", Local::now().format("%H:%M:%S.%3f"));
            if let Some(checkout_id) = mpp.get("checkoutid") {
                let checkout_id = checkout_id.as_i64().unwrap();
                let url = format!("https://shopee.co.id/mpp/{}?flow_source=3", checkout_id);
                println!("{}", url);
                break;
            }
        } 
    }else {
        loop{
            let get_body = task::get_builder(&device_info, &address_info, &[(*chosen_model).clone()], &chosen_payment, &chosen_shipping, None, None, None).await?;
            let (checkout_price_data, order_update_info, dropshipping_info, promotion_data, _selected_payment_channel_data, shoporders, shipping_orders, display_meta_data, fsv_selection_infos, buyer_info, client_event_info, buyer_txn_fee_info, disabled_checkout_info, buyer_service_fee_info, iof_info) = task::checkout_get(client.clone(), shared_headers.clone(), &get_body).await?;
            let place_order_body = task::place_order_builder(&device_info, &checkout_price_data, &order_update_info, &dropshipping_info, &promotion_data, &chosen_payment, &shoporders, &shipping_orders, &display_meta_data, &fsv_selection_infos, &buyer_info, &client_event_info, &buyer_txn_fee_info, &disabled_checkout_info, &buyer_service_fee_info, &iof_info).await?;
            let mpp = task::place_order(client.clone(), &cookie_data, &place_order_body).await?;
            // Mengecek apakah `mpp` memiliki field `checkoutid`
            println!("Current time: {}", Local::now().format("%H:%M:%S.%3f"));
            if let Some(checkout_id) = mpp.get("checkoutid") {
                let checkout_id = checkout_id.as_i64().unwrap();
                let url = format!("https://shopee.co.id/mpp/{}?flow_source=3", checkout_id);
                println!("{}", url);
                break;
            }
        }
    }
    
	
	println!("\nTask completed! Current time: {}", Local::now().format("%H:%M:%S.%3f"));
    // Tunggu input dari pengguna sebelum keluar
    println!("Tekan 'Enter' untuk keluar.");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Gagal membaca baris");
    Ok(())
}

async fn print_voucher_info(voucher_type: &str, voucher: &Option<voucher::Vouchers>) {
    match voucher {
        Some(v) => println!(
            "[{}]{}: {}, {}, {}", Local::now().format("%H:%M:%S.%3f"),
            voucher_type, v.promotionid, v.voucher_code, v.signature
        ),
        None => println!("[{}]{} is None", Local::now().format("%H:%M:%S.%3f"), voucher_type),
    }
}

async fn check_and_adjust_time(task_time_dt: &NaiveDateTime) -> NaiveDateTime {
	let mut updated_task_time_dt = *task_time_dt;
	let current_time = Local::now().naive_local();

	if updated_task_time_dt.signed_duration_since(current_time) < Duration::zero() {
		println!("Waktu yang dimasukkan telah berlalu.");
		let input = get_user_input("Apakah Anda ingin menyetel waktu untuk besok? (yes/no): ");
		match input.trim().to_lowercase().as_str() {
			"yes" | "y" => {
				updated_task_time_dt += Duration::days(1);
				println!("Waktu telah disesuaikan untuk hari berikutnya: {}", updated_task_time_dt);
			}
			_ => println!("Pengaturan waktu tidak diubah."),
		}
	}
	updated_task_time_dt
}
async fn countdown_to_task(task_time_dt: &NaiveDateTime) {
	let task_time_dt = check_and_adjust_time(&task_time_dt).await;
	loop {
		let current_time = Local::now().naive_local();
		let time_until_task = task_time_dt.signed_duration_since(current_time);
		if time_until_task <= Duration::zero() {
			println!("\nTask completed! Current time: {}", current_time.format("%H:%M:%S.%3f"));
			break;
		}
		print!("\r{}", format_duration(time_until_task));
		io::stdout().flush().unwrap();
		tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
	}
}
fn format_duration(duration: Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;
    let milliseconds = duration.num_milliseconds() % 1_000;

    format!("{:02}:{:02}:{:02}.{:03}", &hours, &minutes, &seconds, &milliseconds)
}
fn parse_task_time(task_time_str: &str) -> Result<NaiveDateTime, Box<dyn std::error::Error>> {
	let today = Local::now().date_naive();
	let dt = NaiveDateTime::parse_from_str(&format!("{} {}", today.format("%Y-%m-%d"), task_time_str), "%Y-%m-%d %H:%M:%S%.f")?;
	Ok(dt)
}

fn get_or_prompt<'a>(opt: Option<&'a str>, prompt: &str) -> Cow<'a, str> {
    match opt {
        Some(s) => Cow::Borrowed(s),
        None => Cow::Owned(get_user_input(prompt)),
    }
}

#[cfg(not(windows))]
fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
#[cfg(windows)]
fn get_user_input(prompt: &str) -> String {
    use std::io::{self, Write};
    use std::ptr;
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::System::Console::{GetStdHandle, ReadConsoleW, STD_INPUT_HANDLE};

    print!("{}", prompt);
    io::stdout().flush().unwrap();
    unsafe {
        let h_stdin = GetStdHandle(STD_INPUT_HANDLE);
        if h_stdin == INVALID_HANDLE_VALUE {
            println!("{}", io::Error::last_os_error());
        }
        let mut buffer: [u16; 512] = [0; 512];
        let mut chars_read: u32 = 0;
        let success = ReadConsoleW(
            h_stdin,
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as u32,
            &mut chars_read as *mut u32,
            ptr::null_mut(),
        );
        if success == 0 {
            println!("{}", io::Error::last_os_error());
        }
        let input = String::from_utf16_lossy(&buffer[..chars_read as usize]);
        input.trim().to_string()
    }
}

fn select_cookie_file() -> Result<String> {
    println!("Daftar file cookie yang tersedia:");
    let files = match std::fs::read_dir("./akun") {
        Ok(files) => files,
        Err(err) => return Err(err.into()),
    };

    let mut file_options = Vec::new();
    for (index, file) in files.enumerate() {
        if let Ok(file) = file {
            let file_name = file.file_name();
            println!("{}. {}", index + 1, file_name.to_string_lossy());
            file_options.push(file_name.to_string_lossy().to_string());
        }
    }

    let selected_file = loop {
        if let Ok(index) = get_user_input("Pilih nomor file cookie yang ingin digunakan: ").trim().parse::<usize>() {
            if index > 0 && index <= file_options.len() {
                break file_options[index - 1].clone();
            }
        }
    };

    Ok(selected_file)
}

fn format_thousands(num: i64) -> String {
    let num_str = num.to_string();
    let mut formatted = String::new();
    let len = num_str.len();
    for (i, c) in num_str.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            formatted.push('.');
        }
        formatted.push(c);
    }
    formatted
}
fn human_readable_time(epoch: i64) -> DateTime<Local> {
    let utc = DateTime::<Utc>::from_timestamp(epoch, 0).expect("Invalid timestamp");
    utc.with_timezone(&Local)
}

fn args_checking(opt: &Opt){
	if opt.pro_id.is_some() && opt.sign.is_some() && !opt.claim_platform_vouchers {
        eprintln!("Error: The --pro-id and --sign argument requires --claim-platform-vouchers to be enabled.");
        std::process::exit(1);	
    }else if opt.pro_id.is_some() && !opt.claim_platform_vouchers {
        eprintln!("Error: The --pro-id argument requires --claim-platform-vouchers to be enabled.");
        std::process::exit(1);
	}else if opt.pro_id.is_some() && opt.claim_platform_vouchers && !opt.sign.is_some() {
        eprintln!("Error: The --pro-id argument need --sign argument to be function.");
        std::process::exit(1);
    }else if opt.sign.is_some() && !opt.claim_platform_vouchers {
		eprintln!("Error: The --sign argument requires --claim-platform-vouchers to be enabled.");
		std::process::exit(1);
	}else if opt.sign.is_some() && opt.claim_platform_vouchers  && !opt.pro_id.is_some() {
		eprintln!("Error: The --sign argument need --pro-id argument to be function.");
		std::process::exit(1);
    }else if opt.code_platform.is_some() && !opt.platform_vouchers {
		eprintln!("Error: The --code-platform argument requires --platform-vouchers to be enabled.");
        std::process::exit(1);
	}else if opt.code_shop.is_some() && !opt.shop_vouchers {
		eprintln!("Error: The --code-shop argument requires --shop-vouchers to be enabled.");
        std::process::exit(1);
    } else if opt.no_fsv && (!opt.claim_platform_vouchers && !opt.platform_vouchers && !opt.shop_vouchers) {
        eprintln!("Error: The --no-fsv argument requires at least one of --claim-platform-vouchers, --platform-vouchers, or --shop-vouchers to be enabled.");
        std::process::exit(1);
    }
}