use rquest as reqwest;
use reqwest::tls::Impersonate;
use reqwest::{ClientBuilder, Body, Version, StatusCode};
use reqwest::header::HeaderValue;
use runtime::crypt::{self, DeviceInfo};
use runtime::prepare::{self, CookieData, UserData};
use runtime::telegram::{self, TeleInfo};
use std::io::{self, Write, Read};
use std::fs::File;
use std::process;
use anyhow::Result;
use serde_json::json;
use chrono::{Local, NaiveDateTime};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(prepare::universal_client_skip_headers().await);
    let version_info = env!("CARGO_PKG_VERSION");
    // Welcome Header
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
    println!("Autorun Shopee Draw [Version {}]", version_info);
    println!("");

    // Get account details
    let selected_file = select_cookie_file()?;
    let cookie_content = prepare::read_cookie_file(&selected_file);
	
    let cookie_data = prepare::create_cookie(&cookie_content);
    let base_headers = Arc::new(prepare::create_headers(&cookie_data));
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
    let userdata = prepare::info_akun(client, base_headers).await?;
    println!("Username  : {}", userdata.username);
	println!("Email     : {}", userdata.email);
	println!("Phone     : {}", userdata.phone);
    println!("User ID   : {}", userdata.userid);

    let task_time_str = get_user_input("Enter task time (HH:MM:SS.NNNNNNNNN): ");
    let task_time_dt = parse_task_time(&task_time_str)?;
    // Process HTTP with common function
    countdown_to_task(task_time_dt).await;
    claim_reward(&cookie_data, &userdata, &device_info, &config).await?;
    Ok(())
}
async fn claim_reward(cookie_data: &CookieData, user: &UserData, device_info: &DeviceInfo, config: &TeleInfo) -> Result<(), Box<dyn std::error::Error>> {
    let body_json = json!({});
    // Convert struct to JSON
    let body_str = serde_json::to_string(&body_json).unwrap();
	let body = Body::from(body_str.clone());    
	let url2 = format!("https://idgame.shopee.co.id/api/luckydraw/nold/v1/events/559ab38a6dada34e/draw-prize");
	println!("{}", url2);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("Accept", HeaderValue::from_static("*/*"));
    headers.insert("Accept-Encoding", HeaderValue::from_static("gzip"));
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Windows\""));
    headers.insert("sec-ch-ua", HeaderValue::from_static("\" Not A;Brand\";v=\"99\", \"Chromium\";v=\"99\", \"Google Chrome\";v=\"99\""));
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert("x-user-id", reqwest::header::HeaderValue::from_str(&user.userid.to_string())?);
    headers.insert("Origin", HeaderValue::from_static("https://idgame.shopee.co.id"));
    headers.insert("x-chaplin-version", HeaderValue::from_static("1.0.3"));
    headers.insert("x-game-mode", HeaderValue::from_static("nold_lucky_box"));
    headers.insert("x-app-version-name",  HeaderValue::from_static("0"));
    headers.insert("x-device-platform", HeaderValue::from_static("undefined"));
    headers.insert("x-dfp", reqwest::header::HeaderValue::from_str(&device_info.device_sz_fingerprint)?);
    headers.insert("x-platform", HeaderValue::from_static("128"));
    headers.insert("x-game-version", HeaderValue::from_static("1011001"));
    headers.insert("x-chaplin-appver", HeaderValue::from_static("0"));
    headers.insert("x-useragenttype", HeaderValue::from_static("4"));
    headers.insert("x-clienttype", HeaderValue::from_static("3"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    headers.insert("Referer", HeaderValue::from_static("https://idgame.shopee.co.id/"));
    headers.insert("Accept-Language", HeaderValue::from_static("en-US,en;q=0.9,id;q=0.8"));
    headers.insert("cookie", reqwest::header::HeaderValue::from_str(&cookie_data.cookie_content)?);

	let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .impersonate_skip_headers(Impersonate::Chrome130)
        .enable_ech_grease(true)
        .permute_extensions(true)
        .gzip(true)
        //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
        .build()?;

    // Buat permintaan HTTP POST
    let response = client
        .post(&url2)
        .headers(headers)
        .body(body)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    if response.status() == StatusCode::OK {
        //println!("Headers: {:#?}", response.headers());
        let body_resp = response.text().await?;
        println!("Body: {}", body_resp);
        let msg = format!("Username: {}\nIsi: {}", user.username, body_resp);
        if config.telegram_notif {
            telegram::send_msg(&config, &msg).await?;
        }
    } else {
        println!("Status: {}", response.status());
        println!("Harap Ganti akun");
        process::exit(1);
    }
    Ok(())
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
async fn check_and_adjust_time(task_time_dt: NaiveDateTime) -> NaiveDateTime {
	let mut updated_task_time_dt = task_time_dt;
	let current_time = Local::now().naive_local();
	let time_until_task = updated_task_time_dt.signed_duration_since(current_time);

	if time_until_task < chrono::Duration::zero() {
		// Jika waktu sudah berlalu, tawarkan untuk menyesuaikan waktu
		println!("Waktu yang dimasukkan telah berlalu.");
		println!("Apakah Anda ingin menyetel waktu untuk besok? (yes/no): ");
		
		let mut input = String::new();
		io::stdin().read_line(&mut input).expect("Gagal membaca baris");

		match input.trim().to_lowercase().as_str() {
			"yes" | "y" => {
				// Tambahkan satu hari ke waktu target
				updated_task_time_dt += chrono::Duration::days(1);
				println!("Waktu telah disesuaikan untuk hari berikutnya: {}", updated_task_time_dt);
			}
			_ => println!("Pengaturan waktu tidak diubah."),
		}
	}

	updated_task_time_dt
}
fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
async fn countdown_to_task(task_time_dt: NaiveDateTime) {
	let task_time_dt = check_and_adjust_time(task_time_dt).await;

	loop {
		let current_time = Local::now().naive_local();
		let time_until_task = task_time_dt.signed_duration_since(current_time);

		if time_until_task <= chrono::Duration::zero() {
			println!("\nTask completed! Current time: {}", current_time.format("%H:%M:%S.%3f"));
			tugas_utama();
			break;
		}

		let formatted_time = format_duration(time_until_task);
		print!("\r{}", formatted_time);
		io::stdout().flush().unwrap();

		tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
	}
}
fn format_duration(duration: chrono::Duration) -> String {
	let hours = duration.num_hours();
	let minutes = duration.num_minutes() % 60;
	let seconds = duration.num_seconds() % 60;
	let milliseconds = duration.num_milliseconds() % 1_000;

	format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds)
}
fn parse_task_time(task_time_str: &str) -> Result<NaiveDateTime, Box<dyn std::error::Error>> {
	let today = Local::now().date_naive();
	let dt = NaiveDateTime::parse_from_str(&format!("{} {}", today.format("%Y-%m-%d"), task_time_str), "%Y-%m-%d %H:%M:%S%.f")?;
	Ok(dt)
}
fn tugas_utama() {
	println!("Performing the task...");
	println!("\nTask completed! Current time: {}", Local::now().format("%H:%M:%S.%3f"));
}