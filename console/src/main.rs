/*This Is a Auto Buy Shopee
Whats new in 1.0.0 :
    Introduce new version
    Add detail info flashsale
    Add detail time info
    fix formating telegram messsage
    memory management
Whats new in 0.10.9 :
    Add detail time info
    introdution updater
    fix memory usage
Whats new in 0.10.8 :
    refactoring data types
*/
use runtime::prepare::{self, ModelInfo, ShippingInfo, PaymentInfo, FSItems};
use runtime::task_ng::{SelectedGet, SelectedPlaceOrder, ChannelItemOptionInfo};
use runtime::task::{self};
use runtime::task_ng::{self};
use runtime::voucher::{self};
use runtime::crypt::{self};
use runtime::telegram::{self};
use chrono::{Local, Duration, NaiveDateTime};
use std::io::{self, Write, Read};
use std::process;
use anyhow::Result;
use std::fs::File;
use structopt::StructOpt;
use num_cpus;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

mod kurir_ng;

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

async fn heading_app(promotionid: &str, signature: &str, voucher_code_platform: &str, voucher_code_shop: &str, voucher_collectionid: &str, opt: &Opt, target_url: &str, task_time_str: &str, selected_file: &str, username: &str, name: &str, max_price: &str, chosen_model: &ModelInfo, chosen_shipping: &ShippingInfo, chosen_payment: &PaymentInfo) {
    let padding = 15;
    let version_info = env!("CARGO_PKG_VERSION");
	println!("---------------------------------------------------------------");
    println!("              Auto Buy Shopee [Version {}]              ", version_info);
    println!("{:<padding$}: {}", "Cookie file", selected_file, padding = padding);
    println!("{:<padding$}: {}", "Username", username, padding = padding);
    println!("{:<padding$}: {}", "URL", target_url, padding = padding);
    println!("{:<padding$}: {}", "Time", task_time_str, padding = padding);
    println!("{:<padding$}: {}", "Product", name, padding = padding);
    println!("{:<padding$}: {}", "Variant", chosen_model.name, padding = padding);
    println!("{:<padding$}: {}", "Model Id", chosen_model.modelid, padding = padding);
    println!("{:<padding$}: {}", "Kurir", chosen_shipping.channel_name, padding = padding);
    if !max_price.is_empty() {
        println!("{:<padding$}: {}", "Max Price", max_price, padding = padding);
    }
    println!("{:<padding$}: {}", "Payment", chosen_payment.name, padding = padding);
    if opt.claim_platform_vouchers {
        println!("{:<padding$}: {}", "Mode", "Klaim Platform Voucher", padding = padding);
        println!("{:<padding$}: {}", "Promotion_Id", opt.pro_id.clone().unwrap_or_else(|| promotionid.to_string()), padding = padding);
        println!("{:<padding$}: {}", "Signature", opt.sign.clone().unwrap_or_else(|| signature.to_string()), padding = padding);
    } else if opt.platform_vouchers {
        println!("{:<padding$}: {}", "Mode", "Code Platform Voucher", padding = padding);
        println!("{:<padding$}: {}", "Code", opt.code_platform.clone().unwrap_or_else(|| voucher_code_platform.to_string()), padding = padding);
    } else if opt.collection_vouchers {
        println!("{:<padding$}: {}", "Mode", "Voucher Collection", padding = padding);
        println!("{:<padding$}: {}", "Collection", opt.collectionid.clone().unwrap_or_else(|| voucher_collectionid.to_string()), padding = padding);
    } 
    if opt.shop_vouchers {
        println!("{:<padding$}: {}", "Mode", "Code Shop Voucher", padding = padding);
        println!("{:<padding$}: {}", "Code", opt.code_shop.clone().unwrap_or_else(|| voucher_code_shop.to_string()), padding = padding);
    }
    println!("---------------------------------------------------------------");
    println!("");
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut chosen_model = ModelInfo {
        name: String::from("NOT SET"),
        price: 0,
        stock: 0,
        modelid: 0,
        promotionid: 0,
    };
    let mut chosen_shipping = ShippingInfo {
		original_cost: i64::default(),
		channelid: i64::default(),
        channelidroot: i64::default(),
		channel_name: String::default(),
	};
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
    let mut quantity = 1;
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
    println!("Default Quantity: {}", quantity);
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
    
    let cookie_content = prepare::read_cookie_file(&selected_file);
	
    let cookie_data = prepare::create_cookie(&cookie_content);
    println!("csrftoken: {}", cookie_data.csrftoken);

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
	
    // Get target URL
	let target_url = opt.url.clone().unwrap_or_else(|| get_user_input("Masukan URL: "));
	
	let mut promotionid = String::new();
	let mut signature = String::new();
	let mut voucher_code_platform = String::new();
	let mut voucher_code_shop = String::new();
	let mut voucher_collectionid = String::new();

	if opt.platform_vouchers {
		println!("voucher code platform enable");
		voucher_code_platform = opt.code_platform.clone().unwrap_or_else(|| get_user_input("Masukan voucher code platform: "));
	}
	if opt.shop_vouchers {
		println!("voucher code shop enable");
		voucher_code_shop = opt.code_shop.clone().unwrap_or_else(|| get_user_input("Masukan voucher code shop: "));
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
	let task_time_str = opt.time.clone().unwrap_or_else(|| get_user_input("Enter task time (HH:MM:SS.NNNNNNNNN): "));
    let task_time_dt = parse_task_time(&task_time_str)?;

    clear_screen();
    heading_app(&promotionid, &signature, &voucher_code_platform, &voucher_code_shop, &voucher_collectionid, &opt, &target_url, &task_time_str, &selected_file, "", "", "", &chosen_model, &chosen_shipping, &chosen_payment).await;
	let url_1 = target_url.trim();
    let product_info = prepare::process_url(url_1);
    // Perform the main task
    let (info_result, address_result, product_result) = tokio::join!(
        prepare::info_akun(&cookie_data),
        prepare::address(&cookie_data),
        prepare::get_product(&product_info, &cookie_data)
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
    heading_app(&promotionid, &signature, &voucher_code_platform, &voucher_code_shop, &voucher_collectionid, &opt, &target_url, &task_time_str, &selected_file, &userdata.username, &name, "", &chosen_model, &chosen_shipping, &chosen_payment).await;
	println!("addressid  : {}", address_info.id);
	println!("name             : {}", name);
    // println!("models           : \n{:#?}", model_info);
    println!("Official Shop ?  : {}", is_official_shop);
    let fs_item = if fs_info.promotionid != 0 {
        println!("promotionid  : {}", fs_info.promotionid);
        println!("start_time   : {}", fs_info.start_time);
        println!("end_time     : {}", fs_info.end_time);
        let fs_items = prepare::get_flash_sale_batch_get_items(&cookie_data, &product_info, &fs_info).await?;
        fs_items
    }else {
        Vec::new()
    };
	
	//std::thread::sleep(std::time::Duration::from_secs(2));
    if let Some(model) = choose_model(&model_info, &opt, &fs_item) {
        chosen_model = model;
        println!("Anda memilih model: {:#?}", chosen_model);
        // Lanjutkan dengan logika berikutnya
    } else {
        println!("Model tidak valid.");
		process::exit(1);
        // Handle jika model tidak valid
    }

    println!("Anda memilih model: {}", chosen_model.name);

    let shipping_info = kurir_ng::get_shipping_data(&cookie_data, &device_info, &product_info, &address_info, quantity, &chosen_model, &chosen_payment, &chosen_shipping).await?;

    clear_screen();
    heading_app(&promotionid, &signature, &voucher_code_platform, &voucher_code_shop, &voucher_collectionid, &opt, &target_url, &task_time_str, &selected_file, &userdata.username, &name, "",&chosen_model, &chosen_shipping, &chosen_payment).await;

	if let Some(shipping) = kurir_ng::choose_shipping(&shipping_info, &opt) {
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
	heading_app(&promotionid, &signature, &voucher_code_platform, &voucher_code_shop, &voucher_collectionid, &opt, &target_url, &task_time_str, &selected_file, &userdata.username, &name, "", &chosen_model, &chosen_shipping, &chosen_payment).await;
	let max_price = opt.harga.clone().unwrap_or_else(|| get_user_input("Harga MAX:")).trim().to_string();
    quantity = opt.quantity.clone().unwrap_or_else(|| {
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
    let token = opt.token.clone().unwrap_or_else(|| get_user_input("Token Media: ")).trim().to_string();
	
    let payment_json_data = prepare::open_payment_file().await?;
	let payment_info = prepare::get_payment(&payment_json_data).await?;

	if let Some(payment) = choose_payment(&payment_info, &opt) {
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

	println!("{:?}", chosen_payment);
	clear_screen();
	heading_app(&promotionid, &signature, &voucher_code_platform, &voucher_code_shop, &voucher_collectionid, &opt, &target_url, &task_time_str, &selected_file, &userdata.username, &name, &max_price, &chosen_model, &chosen_shipping, &chosen_payment).await;
    println!("{:?}", chosen_payment);
    let cookie_data = Arc::new(cookie_data);
    let device_info = Arc::new(device_info);
    let product_info = Arc::new(product_info);
    let address_info = Arc::new(address_info);
    let chosen_model = Arc::new(chosen_model);
    let chosen_payment = Arc::new(chosen_payment);
    let chosen_shipping = Arc::new(chosen_shipping);
    let max_price_clone = Arc::new(max_price.clone());
    countdown_to_task(&task_time_dt).await;

    if opt.claim_platform_vouchers || opt.platform_vouchers || opt.collection_vouchers || opt.fsv_only || opt.shop_vouchers {
        if !voucher_collectionid.is_empty() {
            let (promo_id, sig) = voucher::some_function(&voucher_collectionid, &cookie_data).await?;
            promotionid = promo_id;
            signature = sig;
        }
        let cookie_data_clone = Arc::clone(&cookie_data);
        let product_info_clone = Arc::clone(&product_info);
        let shop_task = tokio::spawn(async move{
            if !voucher_code_shop.is_empty() {
                voucher::save_shop_voucher_by_voucher_code(&voucher_code_shop, &cookie_data_clone, &product_info_clone).await
            } else {
                Ok(None)
            }
        });
        let cookie_data_clone = Arc::clone(&cookie_data);
        let platform_task = tokio::spawn(async move{
            if !promotionid.is_empty() && !signature.is_empty() {
                if opt.no_claim_platform_vouchers {
                    return voucher::get_voucher_data(&promotionid, &signature, &cookie_data_clone).await;
                } else {
                    return voucher::save_voucher(&promotionid, &signature, &cookie_data_clone).await;
                }
            } else if !voucher_code_platform.is_empty() {
                return voucher::save_platform_voucher_by_voucher_code(&voucher_code_platform, &cookie_data_clone).await;
            }    
            Ok(None)            
        });
        let cookie_data_clone = Arc::clone(&cookie_data);
        let product_info_clone = Arc::clone(&product_info);
        let chosen_model_clone = Arc::clone(&chosen_model);
        let chosen_payment_clone = Arc::clone(&chosen_payment);
        let chosen_shipping_clone = Arc::clone(&chosen_shipping);
        let recommend_task = tokio::spawn(async move{
            if !opt.no_fsv {
                voucher::get_recommend_platform_vouchers(
                    &cookie_data_clone,
                    &product_info_clone,
                    quantity,
                    &chosen_model_clone,
                    &chosen_payment_clone,
                    &chosen_shipping_clone,
                )
                .await
            } else {
                Ok((None, None))
            }
        });        

        let selected_shop_voucher = shop_task.await??;
        let selected_platform_voucher = platform_task.await??;
        let (freeshipping_voucher, platform_vouchers_target) = recommend_task.await??;
        
        let final_voucher = if opt.fsv_only || (opt.shop_vouchers && !opt.platform_vouchers && !opt.claim_platform_vouchers && !opt.collection_vouchers) {
            None
        } else {
            selected_platform_voucher.or(platform_vouchers_target)
        };
    
        print_voucher_info("freeshipping_voucher", &freeshipping_voucher).await;
        print_voucher_info("platform_voucher", &final_voucher).await;
        print_voucher_info("shop_voucher", &selected_shop_voucher).await;

        let get_body = task_ng::get_body_builder(&device_info, &product_info, &address_info, quantity, &chosen_model, &chosen_payment, &chosen_shipping, Arc::new(freeshipping_voucher), Arc::new(final_voucher), Arc::new(selected_shop_voucher)).await?;
        let get_body = Arc::new(get_body);
        let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(max_threads);
        let stop_flag = Arc::new(AtomicBool::new(false));
        for i in 0..max_threads {
            println!("Running on thread: {}", i);
            let tx = tx.clone();
            let stop_flag = Arc::clone(&stop_flag);
            let cookie_data_clone = Arc::clone(&cookie_data);
            let device_info_clone = Arc::clone(&device_info);
            let chosen_payment_clone = Arc::clone(&chosen_payment);
            let get_body_clone = Arc::clone(&get_body);
            let max_price = Arc::clone(&max_price_clone);
    
            tokio::spawn(async move {
                let mut try_count = 0;
                loop{
                    if stop_flag.load(Ordering::Relaxed) {
                        break;
                    }
                    let place_order_body = match task_ng::get_ng(&cookie_data_clone, &get_body_clone, &device_info_clone, &chosen_payment_clone).await
                    {
                        Ok(body) => body,
                        Err(err) => {
                            eprintln!("Error in get_builder: {:?}", err);
                            continue;
                        }
                    };
                    if !max_price.is_empty(){
                        if let Some(ref data) = place_order_body.checkout_price_data {
                            if let Some(merchandise_subtotal) = data.get("merchandise_subtotal").and_then(|v| v.as_i64()) {
                                println!("merchandise_subtotal: {}", merchandise_subtotal);
                                let max_price_no_comma = max_price.replace(",", "");
                                let cleaned_max_price = max_price_no_comma.trim(); 
                                if let Ok(parsed_max_price) = cleaned_max_price.parse::<i64>() {
                                    let adjusted_max_price = parsed_max_price * 100_000;
                                    //println!("[{}]max_price (setelah dikali 100000): {}", Local::now().format("%H:%M:%S.%3f"), adjusted_max_price);
                                    if merchandise_subtotal <= adjusted_max_price {
                                        println!("[{}]Harga merchandise_subtotal sesuai dengan {}", Local::now().format("%H:%M:%S.%3f"), adjusted_max_price);
                                    } else {
                                        println!("[{}]Harga merchandise_subtotal lebih besar dari {}", Local::now().format("%H:%M:%S.%3f"), adjusted_max_price);
                                        continue;
                                    }
                                } else {
                                    println!("Gagal mengonversi max_price menjadi angka.");
                                    continue;
                                }
                            } else {
                                println!("Gagal mendapatkan nilai merchandise_subtotal.");
                                continue;
                            }
                        } else {
                            println!("[{}]Gagal mendapatkan data checkout_price_data.", Local::now().format("%H:%M:%S.%3f"));
                            continue;
                        }
                    }
                    let mpp = match task_ng::place_order_ng(&cookie_data_clone, &place_order_body).await
                    {
                        Ok(response) => response,
                        Err(err) => {
                            eprintln!("Error in place_order_ng: {:?}", err);
                            continue;
                        }
                    };
                    // Mengecek apakah `mpp` memiliki field `checkoutid`
                    println!("Current time: {}", Local::now().format("%H:%M:%S.%3f"));
                    if let Some(checkout_id) = mpp.get("checkoutid") {
                        let checkout_id = checkout_id.as_i64().unwrap();
                        let url = format!("https://shopee.co.id/mpp/{}?flow_source=3", checkout_id);
                        println!("[{}]{}", Local::now().format("%H:%M:%S.%3f"), url);
                        let _ = tx.send(url).await;
                        stop_flag.store(true, Ordering::Relaxed);
                        break;
                    }
                    if try_count == 3 {
                        eprintln!("[{}]Gagal mendapatkan checkoutid setelah 3 kali percobaan.", Local::now().format("%H:%M:%S.%3f"));
                        stop_flag.store(true, Ordering::Relaxed);
                        break;
                    }
                    try_count += 1;
                }
            });
        }
        drop(tx); // Tutup pengirim setelah semua tugas selesai
        while let Some(url) = rx.recv().await {
            println!("[{}]{}", Local::now().format("%H:%M:%S.%3f"), url);
            if config.telegram_notif {
                let msg = format!("Auto Buy Shopee {}\nREPORT!!!\nUsername     : {}\nProduct      : {}\nVariant      : {}\nLink Payment : {}\nCheckout berhasil!", version_info, userdata.username, name, chosen_model.name, url);
                telegram::send_msg(&config, &msg).await?;
            }
        }
    } else if !token.is_empty(){
        loop{
            // Loop untuk menyesuaikan `merchandise_subtotal`
            let checkout_price_data;
            let order_update_info;
            let dropshipping_info;
            let promotion_data;
            let selected_payment_channel_data;
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
                    price_data, update_info, dropship_info, promo_data, payment_data, 
                    orders, shipping_orders_data, meta_data, fsv_infos, buyer_info_data, 
                    event_info, txn_fee_info, disabled_info, service_fee_info, iof_data
                ) = task::checkout_get(&cookie_data, &get_body.clone()).await?;
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
                            selected_payment_channel_data = payment_data;
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
            let mpp = task::place_order(&cookie_data, &place_order_body).await?;
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
            let get_body = task::get_builder(&device_info, &product_info, &address_info, quantity, &chosen_model, &chosen_payment, &chosen_shipping, None, None, None).await?;
            let (checkout_price_data, order_update_info, dropshipping_info, promotion_data, _selected_payment_channel_data, shoporders, shipping_orders, display_meta_data, fsv_selection_infos, buyer_info, client_event_info, buyer_txn_fee_info, disabled_checkout_info, buyer_service_fee_info, iof_info) = task::checkout_get(&cookie_data, &get_body).await?;
            let place_order_body = task::place_order_builder(&device_info, &checkout_price_data, &order_update_info, &dropshipping_info, &promotion_data, &chosen_payment, &shoporders, &shipping_orders, &display_meta_data, &fsv_selection_infos, &buyer_info, &client_event_info, &buyer_txn_fee_info, &disabled_checkout_info, &buyer_service_fee_info, &iof_info).await?;
            let mpp = task::place_order(&cookie_data, &place_order_body).await?;
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
fn choose_payment(payments: &[PaymentInfo], opt: &Opt) -> Option<PaymentInfo> {
	println!("payment yang tersedia:");

    for (index, payment) in payments.iter().enumerate() {
        println!("{}. {} - Services: {} - Id: {}", index + 1, payment.name, payment.txn_fee / 100000, payment.channel_id);
    }

    if let Some(bayar) = opt.payment.clone() {
        // If opt.payment is present, find the payment with a matching name
        if let Some(selected_payment) = payments.iter().find(|payment| payment.name == bayar) {
            println!("{:?}", selected_payment);
            return Some(selected_payment.clone());
        } else {
            println!("Tidak ada payment dengan nama '{}'", bayar);
            return None;
        }
    }

	let user_input = get_user_input("Pilih payment yang disediakan: ");

    // Convert user input to a number
    if let Ok(choice_index) = user_input.trim().parse::<usize>() {
        // Return the selected payment based on the index
        println!("{:?}", payments.get(choice_index - 1).cloned());
        return payments.get(choice_index - 1).cloned();
    }

    None
}

fn choose_model(models: &[ModelInfo], opt: &Opt, fs_items: &[FSItems]) -> Option<ModelInfo> {
    println!("Variasi yang tersedia:");

    for (index, model) in models.iter().enumerate() {
        let flashsale = if let Some(item) = fs_items.iter().find(|item| item.modelids.contains(&model.modelid)) {
            format!(
                "[FLASHSALE] - Est: {} - Hide: {} - fs-stok: {}",
                format_thousands(item.price_before_discount * (100 - item.raw_discount) / 100 / 100000),
                item.hidden_price_display,
                item.stock
            )
        } else {
            String::new()
        };
        println!("{}. {} - Harga: {} - Stok: {} {}", index + 1, model.name, format_thousands(model.price / 100000), model.stock, flashsale);
    }
    // Check if there is only one model
    if models.len() == 1 {
        println!("Hanya satu variasi tersedia. Memilih secara otomatis.");
        return Some(models[0].clone());
    }

    if let Some(product) = opt.product.clone() {
        // If opt.product is present, find the model with a matching name
        if let Some(selected_model) = models.iter().find(|model| model.name == product) {
            println!("{:?}", selected_model);
            return Some(selected_model.clone());
        } else {
            println!("Tidak ada model dengan nama '{}'", product);
            return None;
        }
    }

    let user_input = get_user_input("Pilih Variasi yang disediakan: ");

    // Mengubah input pengguna ke dalam bentuk angka
    if let Ok(choice_index) = user_input.trim().parse::<usize>() {
        // If opt.product is not present, proceed with user input logic
        if let Some(selected_model) = models.get(choice_index - 1) {
            println!("{:?}", selected_model);
            return Some(selected_model.clone());
        }
    }

    None
}

async fn countdown_to_task(task_time_dt: &NaiveDateTime) {
    loop {
        let current_time = Local::now().naive_local();
        let task_time_naive = task_time_dt.time();
        let time_until_task = task_time_naive.signed_duration_since(current_time.time());

        if time_until_task < Duration::zero() {
            println!("\nTask completed! Current time: {}", current_time.format("%H:%M:%S.%3f"));
            break;
        }

        let formatted_time = format_duration(time_until_task);
        print!("\r{}", formatted_time);
        io::stdout().flush().unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }
}

fn format_duration(duration: Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;
    let milliseconds = duration.num_milliseconds() % 1_000;

    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds)
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

fn parse_task_time(task_time_str: &str) -> Result<NaiveDateTime> {
    match NaiveDateTime::parse_from_str(&format!("2023-01-01 {}", task_time_str), "%Y-%m-%d %H:%M:%S%.f") {
        Ok(dt) => Ok(dt),
        Err(e) => Err(e.into()),
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
		let input = get_user_input("Pilih nomor file cookie yang ingin digunakan: ");

        if let Ok(index) = input.trim().parse::<usize>() {
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