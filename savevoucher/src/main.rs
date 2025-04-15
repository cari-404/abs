/*
This version base of save_voucher 1.2.1
Whats new In 1.2.2 :
integarted tp ABS
*/
use runtime::prepare;
use runtime::voucher::{self, Vouchers};
use runtime::telegram::{self};
use anyhow::Result;
use std::io::{self, Write};
use chrono::{Local, Duration, NaiveDateTime, DateTime, Timelike};
use structopt::StructOpt;
use tokio::sync::Notify;
use std::sync::Arc;
use std::borrow::Cow;

#[derive(Debug, StructOpt)]
#[structopt(name = "Auto save voucher Shopee", about = "Make fast save from shopee.co.id")]
struct Opt {
	#[structopt(short, long, help = "selected file cookie")]
    file: Option<String>,
	#[structopt(short, long, help = "time to run checkout")]
	time: Option<String>,

    #[structopt(short, long, help = "select modes")]
	mode: Option<String>,

	#[structopt(short, long, help = "Set promotionid(need claim_platform_vouchers)")]
    pro_id: Option<String>,
	#[structopt(short, long, help = "Set signature(need claim_platform_vouchers)")]
    sign: Option<String>,
	#[structopt(short, long, help = "Set Voucher from collection_id")]
    collectionid: Option<String>,
}

enum Mode {
	Collection,
	Food,
	Normal,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let version_info = env!("CARGO_PKG_VERSION");
	let max_threads = if num_cpus::get() > 4 { 
		num_cpus::get() 
	} else {
		4 
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
	
	println!("-------------------------------------------");
	println!("save_vouchers [Version {}]", version_info);
	println!("");
	println!("Dapatkan Info terbaru di https://google.com");
	println!("");
	println!("-------------------------------------------");
	let opt = Opt::from_args();
	let mode = select_mode(&opt);
	let client = Arc::new(prepare::universal_client_skip_headers());
	let cookie_data = prepare::create_cookie(&prepare::read_cookie_file(&opt.file.clone().unwrap_or_else(|| select_cookie_file().expect("Folder akun dan file cookie tidak ada\n"))));
	let userdata = prepare::info_akun(&cookie_data).await?;
	let vc_header = Arc::new(voucher::headers_checkout(&cookie_data));
	println!("Username  : {}", &userdata.username);
	match mode {
		Mode::Food => {
			println!("Contoh input: \npromotion_id: 1096081392418868, \nCode: MISSION11");
			let promotion_id = get_or_prompt(opt.pro_id.as_deref(), "Masukan Promotion_Id: ").to_string();
			let code = get_user_input("Masukan Code: ");
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
			let (tx, mut rx) = tokio::sync::mpsc::channel::<Option<Vouchers>>(max_threads);
			let notify = Arc::new(Notify::new());
			let cookie_data_clone = Arc::new(cookie_data.clone());
			let promotion_id_clone = Arc::new(promotion_id.clone());
			let code = Arc::new(code);
			// Process HTTP with common function
			countdown_to_task(&task_time_dt).await;
			for _ in 0..max_threads {
				let tx = tx.clone();
				let cookie_data = Arc::clone(&cookie_data_clone);
				let promotion_id = Arc::clone(&promotion_id_clone);
				let code = Arc::clone(&code);
				let notify = notify.clone();
				let client_clone = Arc::clone(&client);
				tokio::spawn(async move {
					let resp = match voucher::claim_food_voucher(client_clone, &cookie_data, &promotion_id, &code).await {
						Ok(Some(value)) => Some(value),
						Ok(None) => None,
						Err(e) => {
							eprintln!("Error: {:?}", e);
							None // Pastikan `resp` selalu memiliki tipe `Option<Vouchers>`
						}
					};
					if resp.is_some() {
						notify.notify_one(); // Beri sinyal ke thread lain
					}
				
					if let Some(value) = resp {
						if let Err(e) = tx.send(Some(value)).await {
							eprintln!("Gagal mengirim nilai: {:?}", e);
						}
					}
				});
			}
			drop(tx); // Tutup pengirim setelah semua tugas selesai
			while let Some(value) = rx.recv().await {
				if let Some(vouchers) = value {
					println!("[{}]Vouchers ditemukan: {:?}", Local::now().format("%H:%M:%S.%3f"), &vouchers);
					if config.telegram_notif {
						let msg = format!("Save Voucher Shopee {}\nREPORT!!!\nUsername     : {}\nMode      : Food\nVoucher Data      : {:?}\nClaim berhasil!", version_info, userdata.username, &vouchers);
						telegram::send_msg(&config, &msg).await?;
					}
					break; 
				} else {
					println!("Tidak ada vouchers.");
				}
			}
			notify.notify_waiters();
		}
		Mode::Normal => {
			println!("Contoh input: \npromotion_id: 856793882394624, \nSignature: 8e8a4ced8d6905570114f163a08a15de55c3fed560f8a3a8a25e6e179783b480");
            let promotion_id = get_or_prompt(opt.pro_id.as_deref(), "Masukan Promotion_Id: ").to_string();
            let signature = get_or_prompt(opt.sign.as_deref(), "Masukan Signature: ").to_string();
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
			let (tx, mut rx) = tokio::sync::mpsc::channel::<Option<Vouchers>>(max_threads);
			let notify = Arc::new(Notify::new());
			let promotion_id_clone = Arc::new(promotion_id.clone());
			let signature_clone = Arc::new(signature.clone());
			// Process HTTP with common function
			countdown_to_task(&task_time_dt).await;
			for _ in 0..max_threads {
				let tx = tx.clone();
				let vc_header = Arc::clone(&vc_header);
				let promotion_id = Arc::clone(&promotion_id_clone);
				let signature = Arc::clone(&signature_clone);
				let notify = notify.clone();
				let client_clone = Arc::clone(&client);

				tokio::spawn(async move {
					let resp = match voucher::save_voucher(client_clone, &promotion_id, &signature, vc_header).await {
						Ok(Some(value)) => Some(value),
						Ok(None) => None,
						Err(e) => {
							eprintln!("Error: {:?}", e);
							None // Pastikan `resp` selalu memiliki tipe `Option<Vouchers>`
						}
					};
					if resp.is_some() {
						notify.notify_one(); // Beri sinyal ke thread lain
					}
				
					if let Some(value) = resp {
						if let Err(e) = tx.send(Some(value)).await {
							eprintln!("Gagal mengirim nilai: {:?}", e);
						}
					}
				});
			}
			drop(tx); // Tutup pengirim setelah semua tugas selesai
			while let Some(value) = rx.recv().await {
				if let Some(vouchers) = value {
					println!("[{}]Vouchers ditemukan: {:?}", Local::now().format("%H:%M:%S.%3f"), &vouchers);
					if config.telegram_notif {
						let msg = format!("Save Voucher Shopee {}\nREPORT!!!\nUsername     : {}\nMode      : Normal\nVoucher Data      : {:?}\nClaim berhasil!", version_info, userdata.username, &vouchers);
						telegram::send_msg(&config, &msg).await?;
					}
					break; 
				} else {
					println!("Tidak ada vouchers.");
				}
			}
			notify.notify_waiters();
		}
		Mode::Collection => {
			println!("Contoh input: collection_id: 12923214728");
			let voucher_collectionid = get_or_prompt(opt.collectionid.as_deref(), "Masukan collection_id: ");
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
			// Process HTTP with common function
			countdown_to_task(&task_time_dt).await;
			let (promotion_id, signature) = voucher::some_function(client.clone(), &voucher_collectionid, &cookie_data).await?;
			println!("promotion_id : {}", &promotion_id);
			println!("signature	: {}", &signature);
			voucher::save_voucher(client.clone(), &promotion_id, &signature, vc_header).await?;
		}
	}
	println!("\nTask completed! Current time: {}", Local::now().format("%H:%M:%S.%3f"));
	Ok(())
}

fn select_mode(opt: &Opt) -> Mode {
	loop {
		println!("Pilih mode:");
		println!("1. Normal");
		println!("2. Collection");
		println!("3. Food");
        let input = opt.mode.clone().unwrap_or_else(|| get_user_input("Masukkan pilihan (1/2/..): "));
		match input.trim() {
			"1" => return Mode::Normal,
			"2" => return Mode::Collection,
			"3" => return Mode::Food,
			_ => println!("Pilihan tidak valid, coba lagi."),
		}
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
			tugas_utama();
			break;
		}
		print!("\r{}", format_duration(time_until_task));
		io::stdout().flush().unwrap();
		tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
	}
}

fn tugas_utama() {
	println!("Performing the task...");
	println!("\nTask completed! Current time: {}", Local::now().format("%H:%M:%S.%3f"));
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
        input.trim_end().to_string()
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