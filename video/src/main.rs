use rquest as reqwest;
use reqwest::tls::Impersonate;
use reqwest::{ClientBuilder, Body, Version, StatusCode};
use reqwest::header::HeaderValue;
use runtime::prepare::{self};
use runtime::crypt::{self};
use std::io::{self, Write};
use std::process;
use anyhow::Result;
use chrono::{self, Local, NaiveDateTime, Utc};
use serde_json::Value;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let version_info = env!("CARGO_PKG_VERSION");
    // Welcome Header
    println!("Autorun Shopee Video [Version {}]", version_info);
    println!("");

    // Get account details
    let selected_file = select_cookie_file()?;
    
    let cookie_content = prepare::read_cookie_file(&selected_file);
	
    let csrftoken = prepare::extract_csrftoken(&cookie_content);
    println!("csrftoken: {}", csrftoken);

    let device_id = crypt::generate_device_id();
    let device_fingerprint = crypt::generate_device_fingerprint();

    let task_time_str = get_user_input("Enter task time (HH:MM:SS.NNNNNNNNN): ");
    let task_time_dt = parse_task_time(&task_time_str)?;
    // Process HTTP with common function
    countdown_to_task(task_time_dt).await;

    let (task_id, eventcode, period_num, mut reward) = cek_reward(&cookie_content, &device_id, &device_fingerprint).await?;
    println!("{}, {}, {}, {}",task_id, eventcode, period_num, reward.expect("REASON").to_string());

    // Tambah 1 pada reward jika ada
    if let Some(reward_value) = reward {
        reward = Some(reward_value + 1);
    }

    // Loop reward berjalan hingga reward > 61
    loop {
        if let Some(current_reward) = reward {
            if current_reward >= 61 {
                println!("Reward reached 61, Finish...");
                break; // Keluar dari loop setelah reward mencapai 61
            }
            let (rep_task_id, rep_eventcode, rep_period_num, rep_reward) = report_reward(&cookie_content, &device_id, &device_fingerprint, task_id, period_num, current_reward).await?;
            println!("{}, {}, {}, {}",rep_task_id, rep_eventcode, rep_period_num, rep_reward.expect("REASON").to_string());
            if current_reward == 5 || current_reward == 15 || current_reward == 60{
                claim_reward(&cookie_content, &device_id, &device_fingerprint, task_id, eventcode, period_num, current_reward).await?;
            }
            let next_task_time_dt = Local::now().naive_local() + chrono::Duration::minutes(1);
            countdown_to_task(next_task_time_dt).await;

            println!(); // Pindah ke baris baru setelah countdown selesai
            println!("{}", current_reward);

            // Tambahkan reward setelah countdown selesai
            reward = Some(current_reward + 1);
        }
    }
    Ok(())
}

async fn claim_reward(cookie_content: &str, device_id: &str, device_fingerprint: &str, task_id: i64, eventcode: i64, period_num: i64, current_reward: i64) -> Result<(), Box<dyn std::error::Error>> {
    let csrftoken = prepare::extract_csrftoken(&cookie_content);
    let current_time = Utc::now();
    let body_json = json!({
		"task_id": task_id,
        "event_code": eventcode,
		"reward_id": current_reward,
        "period_num": period_num,
        "reward_type": 100,
        "claim_type": 1,
        "reward_sequence": current_reward,
        "extra_info": {
            "device_id": device_id,
            "shopee_df": device_fingerprint,
            "security_device_fingerprint": "",
            "rn_req_origin": true
        },
	  });
    // Convert struct to JSON
    let body_str = serde_json::to_string(&body_json).unwrap();
	let body = Body::from(body_str.clone());    
	let url2 = format!("https://ug-api.sv.shopee.co.id/api/v2/biz/reward/claim?_timestamp={}&version=1", current_time.timestamp());
	println!("{}", url2);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("okhttp/3.12.4 app_type=1 platform=native_android os_ver=34 appver=33350 Cronet/102.0.5005.61"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("Accept", HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert("Accept-Encoding", HeaderValue::from_static("gzip"));
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("language", HeaderValue::from_static("id"));
    headers.insert("did", reqwest::header::HeaderValue::from_str(device_id)?);
    headers.insert("fid", reqwest::header::HeaderValue::from_str(device_fingerprint)?);
    headers.insert("x-requested-from", HeaderValue::from_static("rn"));
    headers.insert("x-sap-type", HeaderValue::from_static("1"));
    headers.insert("region", HeaderValue::from_static("ID"));
    headers.insert("shopee_df", reqwest::header::HeaderValue::from_str(device_fingerprint)?);
    headers.insert("sv-referer", HeaderValue::from_static("reward_home_page"));
    headers.insert("os-type", HeaderValue::from_static("2"));
    headers.insert("sv-source-page", HeaderValue::from_static("copy_link"));
    headers.insert("sv-pre-page", HeaderValue::from_static("trending_page"));
    headers.insert("requestinfo", HeaderValue::from_static("{\"deviceInfo\":{\"brand\":\"Xiaomi\",\"appDeviceName\":\"Brand/xiaomi Model/mi_a1 OSVer/34 Manufacturer/Xiaomi\",\"model\":\"Mi A1\",\"appOSVersion\":\"34\",\"platform\":0},\"networkInfo\":{\"networkType\":\"wifi\"},\"locationInfo\":{\"addresses\":[],\"gps\":{}}}"));
    headers.insert("requestinfo-enc", HeaderValue::from_static(""));
    headers.insert("sv-from-source", HeaderValue::from_static("BGB_VIDEO_PAGE_MERGE_TAB"));
    headers.insert("client-info", HeaderValue::from_static(""));
    headers.insert("x-request-id", HeaderValue::from_static(""));
    headers.insert("sfid", HeaderValue::from_static(""));
    headers.insert("shopee_http_dns_mode", HeaderValue::from_static("1"));
    headers.insert("x-shopee-client-timezone", HeaderValue::from_static("Asia/Jakarta"));
    headers.insert("cache-control", HeaderValue::from_static("no-cache, no-store"));
    headers.insert("client-request-id", HeaderValue::from_static("069f6695-5e8b-4d2b-a621-de7bd51f9a1f.283"));
    headers.insert("af-ac-enc-dat", HeaderValue::from_static(""));
    headers.insert("af-ac-enc-id", HeaderValue::from_static(""));
    headers.insert("af-ac-enc-sz-token", HeaderValue::from_static(""));
    headers.insert("x-sap-ri", HeaderValue::from_static("8fab8288812ce5572fd20624a59333cea398a23b43b3f793"));
    headers.insert("x-csrftoken", reqwest::header::HeaderValue::from_str(&csrftoken)?);
    let mut cookie = cookie_content.to_owned();
    cookie.push_str(&format!("; shopee_rn_version={}", current_time.timestamp()));
    headers.insert("cookie", reqwest::header::HeaderValue::from_str(&cookie)?);

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
    } else {
        println!("Status: {}", response.status());
        println!("Harap Ganti akun");
        process::exit(1);
    }
    Ok(())
}

async fn report_reward(cookie_content: &str, device_id: &str, device_fingerprint: &str, task_id: i64, period_num: i64, current_reward: i64) -> Result<(i64, i64, i64, Option<i64>), Box<dyn std::error::Error>> {
    let csrftoken = prepare::extract_csrftoken(&cookie_content);
    let current_time = Utc::now();
    let body_json = json!({
		"task_id": task_id,
		"reward_id": current_reward,
        "period_num": period_num,
        "extra_info": {
            "device_id": device_id,
            "shopee_df": device_fingerprint,
            "security_device_fingerprint": ""
        },
        "from_source":  "copy_link",
	  });
    // Convert struct to JSON
    let body_str = serde_json::to_string(&body_json).unwrap();
	let body = Body::from(body_str.clone());    
	let url2 = format!("https://ug-api.sv.shopee.co.id/api/v2/biz/reward/report/time/round?_timestamp={}&version=1", current_time.timestamp());
	println!("{}", url2);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("okhttp/3.12.4 app_type=1 platform=native_android os_ver=34 appver=33350 Cronet/102.0.5005.61"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("Accept", HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert("Accept-Encoding", HeaderValue::from_static("gzip"));
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("language", HeaderValue::from_static("id"));
    headers.insert("did", reqwest::header::HeaderValue::from_str(device_id)?);
    headers.insert("fid", reqwest::header::HeaderValue::from_str(device_fingerprint)?);
    headers.insert("x-requested-from", HeaderValue::from_static("rn"));
    headers.insert("x-sap-type", HeaderValue::from_static("1"));
    headers.insert("region", HeaderValue::from_static("ID"));
    headers.insert("shopee_df", reqwest::header::HeaderValue::from_str(device_fingerprint)?);

    headers.insert("sv-referer", HeaderValue::from_static("trending_page"));
    headers.insert("os-type", HeaderValue::from_static("2"));
    headers.insert("sv-from-source", HeaderValue::from_static("copy_link"));
    headers.insert("sv-req-timestamp", reqwest::header::HeaderValue::from_str(&current_time.timestamp().to_string())?);
    headers.insert("sv-source-page", HeaderValue::from_static(""));
    headers.insert("sv-pre-page", HeaderValue::from_static(""));
    headers.insert("requestinfo", HeaderValue::from_static("{\"deviceInfo\":{\"brand\":\"Xiaomi\",\"appDeviceName\":\"Brand/xiaomi Model/mi_a1 OSVer/34 Manufacturer/Xiaomi\",\"model\":\"Mi A1\",\"appOSVersion\":\"34\",\"platform\":0},\"networkInfo\":{\"networkType\":\"wifi\"},\"locationInfo\":{\"addresses\":[],\"gps\":{}}}"));
    headers.insert("requestinfo-enc", HeaderValue::from_static(""));
    headers.insert("client-info", HeaderValue::from_static(""));
    headers.insert("x-request-id", HeaderValue::from_static(""));
    headers.insert("sfid", HeaderValue::from_static(""));
    headers.insert("shopee_http_dns_mode", HeaderValue::from_static("1"));
    headers.insert("x-shopee-client-timezone", HeaderValue::from_static("Asia/Jakarta"));
    headers.insert("cache-control", HeaderValue::from_static("no-cache, no-store"));
    headers.insert("client-request-id", HeaderValue::from_static("069f6695-5e8b-4d2b-a621-de7bd51f9a1f.283"));
    headers.insert("af-ac-enc-dat", HeaderValue::from_static(""));
    headers.insert("af-ac-enc-id", HeaderValue::from_static(""));
    headers.insert("af-ac-enc-sz-token", HeaderValue::from_static(""));
    headers.insert("x-sap-ri", HeaderValue::from_static("8fab8288812ce5572fd20624a59333cea398a23b43b3f793"));
    headers.insert("x-csrftoken", reqwest::header::HeaderValue::from_str(&csrftoken)?);
    let mut cookie = cookie_content.to_owned();
    cookie.push_str(&format!("; shopee_rn_version={}", current_time.timestamp()));
    headers.insert("cookie", reqwest::header::HeaderValue::from_str(&cookie)?);

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
        let v: Value = response.json().await?;
        println!("Body: {}", v);
        // Extract the task_id
        // Extract task_id, event_code, and period_num
        let rep_task_id = v["data"]["refresh_config"]["task_pendant_config"]["task_id"]
            .as_i64()
            .ok_or("task_id not found")?;
        let rep_event_code = v["data"]["refresh_config"]["task_pendant_config"]["event_code"]
            .as_i64()
            .ok_or("event_code not found")?;
        let rep_period_num = v["data"]["refresh_config"]["task_pendant_config"]["period_num"]
            .as_i64()
            .ok_or("period_num not found")?;

        // Access the task_rewards array
        let rep_rewards = v["data"]["refresh_config"]["task_pendant_config"]["task_rewards"]
            .as_array()
            .ok_or("task_rewards not found")?;

        // Find the last completed reward
        let mut last_completed_reward_id = 0;
        for reward in rep_rewards.iter().rev() {
            if let Some(completed) = reward["completed"].as_bool() {
                if completed {
                    last_completed_reward_id = reward["reward_id"].as_i64().expect("REASON");
                    break;
                }
            }
        }

        // Return extracted values and the last completed reward_id
        Ok((rep_task_id, rep_event_code, rep_period_num, Some(last_completed_reward_id)))
    } else {
        println!("Status: {}", response.status());
        println!("Harap Ganti akun");
        process::exit(1);
    }
}

async fn cek_reward(cookie_content: &str, device_id: &str, device_fingerprint: &str) -> Result<(i64, i64, i64, Option<i64>), Box<dyn std::error::Error>> {
    let current_time = Utc::now();
	let url2 = format!("https://ug-api.sv.shopee.co.id/api/v2/reward/config?_timestamp={}&version=1&from_source=app_auto_streaming&shopee_df={}&engine_type=2&source=1", current_time.timestamp(), device_fingerprint);
	println!("{}", url2);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("okhttp/3.12.4 app_type=1 platform=native_android os_ver=34 appver=33350 Cronet/102.0.5005.61"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("Accept", HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert("language", HeaderValue::from_static("id"));
    headers.insert("did", reqwest::header::HeaderValue::from_str(device_id)?);
    headers.insert("fid", reqwest::header::HeaderValue::from_str(device_fingerprint)?);
    headers.insert("x-requested-from", HeaderValue::from_static("rn"));
    headers.insert("sv-referer", HeaderValue::from_static("content_mix_feed"));
    headers.insert("os-type", HeaderValue::from_static("2"));
    headers.insert("sv-from-source", HeaderValue::from_static("app_auto_streaming"));
    headers.insert("sv-req-timestamp", reqwest::header::HeaderValue::from_str(&current_time.timestamp().to_string())?);
    headers.insert("sv-source-page", HeaderValue::from_static(""));
    headers.insert("sv-pre-page", HeaderValue::from_static(""));
    headers.insert("requestinfo", HeaderValue::from_static("{\"deviceInfo\":{\"brand\":\"Xiaomi\",\"appDeviceName\":\"Brand/xiaomi Model/mi_a1 OSVer/34 Manufacturer/Xiaomi\",\"model\":\"Mi A1\",\"appOSVersion\":\"34\",\"platform\":0},\"networkInfo\":{\"networkType\":\"wifi\"},\"locationInfo\":{\"addresses\":[],\"gps\":{}}}"));
    headers.insert("requestinfo-enc", HeaderValue::from_static(""));
    headers.insert("client-info", HeaderValue::from_static(""));
    headers.insert("x-request-id", HeaderValue::from_static(""));
    headers.insert("sfid", HeaderValue::from_static(""));
    headers.insert("x-shopee-client-timezone", HeaderValue::from_static("Asia/Jakarta"));
    headers.insert("cache-control", HeaderValue::from_static("no-cache, no-store"));
    headers.insert("client-request-id", HeaderValue::from_static("069f6695-5e8b-4d2b-a621-de7bd51f9a1f.283"));
    headers.insert("af-ac-enc-dat", HeaderValue::from_static(""));
    headers.insert("af-ac-enc-id", HeaderValue::from_static(""));
    headers.insert("af-ac-enc-sz-token", HeaderValue::from_static(""));
    let mut cookie = cookie_content.to_owned();
    cookie.push_str(&format!("; shopee_rn_version={}", current_time.timestamp()));
    headers.insert("cookie", reqwest::header::HeaderValue::from_str(&cookie)?);

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
        .get(&url2)
        .headers(headers)
        .version(Version::HTTP_2) 
        .send()
        .await?;

    if response.status() == StatusCode::OK {
        //println!("Headers: {:#?}", response.headers());
        let v: Value = response.json().await?;
        println!("Body: {}", v);
        // Extract the task_id
        // Extract task_id, event_code, and period_num
        let task_id = v["data"]["task_pendant_config"]["task_id"]
            .as_i64()
            .ok_or("task_id not found")?;
        let event_code = v["data"]["task_pendant_config"]["event_code"]
            .as_i64()
            .ok_or("event_code not found")?;
        let period_num = v["data"]["task_pendant_config"]["period_num"]
            .as_i64()
            .ok_or("period_num not found")?;

        // Access the task_rewards array
        let rewards = v["data"]["task_pendant_config"]["task_rewards"]
            .as_array()
            .ok_or("task_rewards not found")?;

        // Find the last completed reward
        let mut last_completed_reward_id = 0;
        for reward in rewards.iter().rev() {
            if let Some(completed) = reward["completed"].as_bool() {
                if completed {
                    last_completed_reward_id = reward["reward_id"].as_i64().expect("REASON");
                    break;
                }
            }
        }

        // Return extracted values and the last completed reward_id
        Ok((task_id, event_code, period_num, Some(last_completed_reward_id)))
    } else {
        println!("Status: {}", response.status());
        println!("Harap Ganti akun");
        process::exit(1);
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