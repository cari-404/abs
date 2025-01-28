/*
This version base of save_voucher 1.2.1
Whats new In 1.2.2 :
integarted tp ABS
*/
use runtime::prepare;
use runtime::voucher::{self, Vouchers};
use anyhow::Result;
use std::fs::File;
use std::io::{self, Read, Write};
use chrono::{Local, Duration, NaiveDateTime};
use structopt::StructOpt;
use tokio::sync::Notify;
use std::sync::Arc;

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
	Normal,
	Collection,
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
	
	println!("-------------------------------------------");
	println!("save_vouchers [Version {}]", version_info);
	println!("");
	println!("Dapatkan Info terbaru di https://google.com");
	println!("");
	println!("-------------------------------------------");
	let opt = Opt::from_args();
	let mode = select_mode(&opt);
	
	let selected_file = opt.file.clone().unwrap_or_else(|| select_cookie_file().expect("Folder akun dan file cookie tidak ada\n"));

	// Read the content of the selected cookie file
	let file_path = format!("./akun/{}", selected_file);
	let mut cookie_content = String::new();
	File::open(&file_path)?.read_to_string(&mut cookie_content)?;
	let cookie_data = prepare::create_cookie(&cookie_content);
	match mode {
		Mode::Normal => {
			println!("Contoh input: \npromotion_id: 856793882394624, \nSignature: 8e8a4ced8d6905570114f163a08a15de55c3fed560f8a3a8a25e6e179783b480");

            let promotion_id = opt.pro_id.clone().unwrap_or_else(|| get_user_input("Masukan Promotion_Id: "));
            let signature = opt.sign.clone().unwrap_or_else(|| get_user_input("Masukan Signature: "));	
			
			let task_time_str = opt.time.clone().unwrap_or_else(|| get_user_input("Enter task time (HH:MM:SS.NNNNNNNNN): "));
			let task_time_dt = parse_task_time(&task_time_str)?;
			let max_threads = if num_cpus::get() > 4 { 
				num_cpus::get() 
			} else {
				4 
			}; 
			let (tx, mut rx) = tokio::sync::mpsc::channel::<Option<Vouchers>>(max_threads);
			let notify = Arc::new(Notify::new());
			// Process HTTP with common function
			countdown_to_task(task_time_dt).await;
			for _ in 0..max_threads {
				let tx = tx.clone();
				let cookie_data = cookie_data.clone();
				let promotion_id = promotion_id.clone();
				let signature = signature.clone();
				let notify = notify.clone();
				tokio::spawn(async move {
					let resp = match voucher::save_voucher(&promotion_id, &signature, &cookie_data).await {
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
					println!("Vouchers ditemukan: {:?}", vouchers);
					break; 
				} else {
					println!("Tidak ada vouchers.");
				}
			}
			notify.notify_waiters();
		}
		Mode::Collection => {
			println!("Contoh input: collection_id: 12923214728");
			let voucher_collectionid = opt.collectionid.clone().unwrap_or_else(|| get_user_input("Masukan collection_id: "));
			let task_time_str = opt.time.clone().unwrap_or_else(|| get_user_input("Enter task time (HH:MM:SS.NNNNNNNNN): "));
			let task_time_dt = parse_task_time(&task_time_str)?;
			// Process HTTP with common function
			countdown_to_task(task_time_dt).await;
			let (promotion_id, signature) = voucher::some_function(&voucher_collectionid, &cookie_data).await?;
			println!("promotion_id : {}", promotion_id);
			println!("signature	: {}", signature);
			voucher::save_voucher(&promotion_id, &signature, &cookie_data).await?;
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

        let input = opt.mode.clone().unwrap_or_else(|| get_user_input("Masukkan pilihan (1/2): "));

		match input.trim() {
			"1" => return Mode::Normal,
			"2" => return Mode::Collection,
			_ => println!("Pilihan tidak valid, coba lagi."),
		}
	}
}
async fn check_and_adjust_time(task_time_dt: NaiveDateTime) -> NaiveDateTime {
	let mut updated_task_time_dt = task_time_dt;
	let current_time = Local::now().naive_local();
	let time_until_task = updated_task_time_dt.signed_duration_since(current_time);

	if time_until_task < Duration::zero() {
		// Jika waktu sudah berlalu, tawarkan untuk menyesuaikan waktu
		println!("Waktu yang dimasukkan telah berlalu.");
		println!("Apakah Anda ingin menyetel waktu untuk besok? (yes/no): ");
		
		let mut input = String::new();
		io::stdin().read_line(&mut input).expect("Gagal membaca baris");

		match input.trim().to_lowercase().as_str() {
			"yes" | "y" => {
				// Tambahkan satu hari ke waktu target
				updated_task_time_dt += Duration::days(1);
				println!("Waktu telah disesuaikan untuk hari berikutnya: {}", updated_task_time_dt);
			}
			_ => println!("Pengaturan waktu tidak diubah."),
		}
	}

	updated_task_time_dt
}

async fn countdown_to_task(task_time_dt: NaiveDateTime) {
	let task_time_dt = check_and_adjust_time(task_time_dt).await;

	loop {
		let current_time = Local::now().naive_local();
		let time_until_task = task_time_dt.signed_duration_since(current_time);

		if time_until_task <= Duration::zero() {
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

fn tugas_utama() {
	println!("Performing the task...");
	println!("\nTask completed! Current time: {}", Local::now().format("%H:%M:%S.%3f"));
}
fn get_user_input(prompt: &str) -> String {
	print!("{}", prompt);
	io::stdout().flush().unwrap();
	let mut input = String::new();
	io::stdin().read_line(&mut input).unwrap();
	input.trim().to_string()
}
fn format_duration(duration: Duration) -> String {
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