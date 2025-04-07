use rquest as reqwest;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::tls::Impersonate;
use reqwest::redirect::Policy as RedirectPolicy;
use serde_json::Value;
use tokio::io::{self, BufWriter, AsyncWriteExt};
use std::cmp::Ordering;
use indicatif::{ProgressBar, ProgressStyle};
use futures_util::StreamExt;
use std::time::Instant;
use std::path::Path;
use once_cell::sync::Lazy;
use std::env;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};

pub static HEADER_TEST: Lazy<HeaderMap> = Lazy::new(|| {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static("Android app Shopee appver=29344 app_type=1"));
    headers
});

const REPO_OWNER: &str = "cari-404";
const REPO_NAME: &str = "ABS";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const OS: &str = std::env::consts::OS;
const ARCH: &str = std::env::consts::ARCH;
const OUTPUT_PATH_SLIM: &str = "update.7z";
#[cfg(target_os = "windows")]
const OUTPUT_PATH_DEFAULT: &str = "update.zip";
#[cfg(not(target_os = "windows"))]
const OUTPUT_PATH_DEFAULT: &str = "update.tar.gz";
static OUTPUT_PATH: Lazy<&'static str> = Lazy::new(|| {
    let is_slim = env::args().any(|arg| arg == "slim");
    if is_slim {
        OUTPUT_PATH_SLIM
    } else {
        OUTPUT_PATH_DEFAULT
    }
});

fn compare_versions(local: &str, remote: &str) -> Ordering {
    let parse = |s: &str| s.split('.').filter_map(|p| p.parse::<u32>().ok()).collect::<Vec<_>>();
    let local_parts = parse(local);
    let remote_parts = parse(remote);
    local_parts.cmp(&remote_parts)
}

async fn get_latest_release() -> Option<String> {
    let url = format!("https://api.github.com/repos/{}/{}/releases/latest", REPO_OWNER, REPO_NAME);
    let client = Client::new();
    let response = client.get(&url)
        .header("User-Agent", "rust-updater")
        .send()
        .await
        .ok()?;
    
    if response.status().is_success() {
        let json: Value = response.json().await.ok()?;
        json["tag_name"].as_str().map(|s| s.trim_start_matches('v').to_string())
    } else {
        None
    }
}

async fn download_latest_release(url: &str) -> tokio::io::Result<()> {
    use tokio::fs::OpenOptions;
    println!("URL unduhan: {}", url);
    let client = ClientBuilder::new()
        .gzip(true)
        .redirect(RedirectPolicy::limited(10))
        .build()
        .expect("Failed to Create Client");
    let response = client.get(url)
        .send()
        .await
        .map_err(|e| {
            eprintln!("Gagal mengunduh: {}", e);
            io::Error::new(io::ErrorKind::Other, "Gagal mengunduh file")
        })?;
    println!("Status: {}", response.status());
    let total_size = response.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {bytes_per_sec}")
        .expect("Invalid template")
        .progress_chars("#>-"));

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true) // Hindari data lama tertinggal
        .open(*OUTPUT_PATH)
        .await?;
    let mut file = BufWriter::new(file);
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let start = Instant::now();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| {
            eprintln!("Error saat mengunduh chunk: {}", e);
            io::Error::new(io::ErrorKind::Other, "Error saat menerima data")
        })?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        pb.inc(chunk.len() as u64);
        pb.set_position(downloaded);
        let elapsed = start.elapsed().as_secs_f64();
        let speed = (pb.position() as f64 / elapsed) / 1024.0;
        pb.set_message(format!("{:.2} KB/s", speed));
    }
    file.flush().await?;
    pb.finish_with_message("Download selesai!");
    let actual_size = tokio::fs::metadata(*OUTPUT_PATH).await?.len();
    if actual_size != total_size {
        eprintln!("File mungkin corrupt! Ukuran seharusnya {} bytes, tetapi hanya {} bytes.", total_size, actual_size);
        return Err(io::Error::new(io::ErrorKind::Other, "Ukuran file tidak sesuai"));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
async fn extract_archive(exp: bool) -> io::Result<()> {
    use zip::ZipArchive;
    use std::io::Read;

    // Buat folder update-dir
    let update_dir = Path::new("update-dir");
    if !update_dir.exists() {
        tokio::fs::create_dir(update_dir).await?;
    }
    if exp {
        sevenz_rust2::decompress_file(*OUTPUT_PATH, "update-dir").expect("complete");
    }else{
        let file = tokio::fs::File::open(*OUTPUT_PATH).await?;
        let mut archive = ZipArchive::new(file.into_std().await)?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = update_dir.join(file.name());
            if file.name().ends_with('/') {
                tokio::fs::create_dir_all(&outpath).await?;
            } else {
                if let Some(parent) = outpath.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                let mut outfile = tokio::fs::File::create(&outpath).await?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                outfile.write_all(&buffer).await?;
            }
        }
    }
    let _ = std::process::Command::new("cmd")
        .arg("/C")
        .arg("update-dir\\updater.exe upgrade")
        .spawn();

    Ok(())
}

#[cfg(not(target_os = "windows"))]
async fn extract_archive(exp: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::path::PathBuf;
    use std::fs::{self, File};
    use std::os::unix::fs::PermissionsExt;
    use flate2::read::GzDecoder;
    use tar::Archive;
    use tokio::task;

    task::spawn_blocking(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let update_dir = Path::new("update-dir");
        if !update_dir.exists() {
            fs::create_dir(update_dir)?;
        }
        if exp {
            sevenz_rust2::decompress_file(*OUTPUT_PATH, "update-dir").expect("complete");
        }else {
            let file = File::open(*OUTPUT_PATH)?;
            let decompressed = GzDecoder::new(file);
            let mut archive = Archive::new(decompressed);
            archive.unpack(update_dir)?;
        };
        fn set_executable(path: PathBuf) -> std::io::Result<()> {
            let metadata = fs::metadata(&path)?;
            let mut perms = metadata.permissions();
            // Set izin eksekusi untuk pemilik, grup, dan lainnya
            perms.set_mode(perms.mode() | 0o755);
            fs::set_permissions(&path, perms)?;
            Ok(())
        }

        // Rekursif menambahkan izin eksekusi
        fn make_all_executable(dir: &Path) -> std::io::Result<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    make_all_executable(&path)?; // Rekursif jika direktori
                } else {
                    set_executable(path)?; // Set izin jika file
                }
            }
            Ok(())
        }
        // Terapkan izin eksekusi pada seluruh direktori hasil ekstraksi
        make_all_executable(update_dir)?;
        Ok(())
    })
    .await??;
    println!("Please Wait....");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("Overwrite permission files");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    println!("Almost Done");

    let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg("./update-dir/updater upgrade")
        .spawn();

    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut force = false;
    let mut slim = false;
    if args.len() > 1 {
        match args[1].as_str() {
            "help" => {
                println!("Penggunaan: updater [upgrade|force|test|slim]");
                println!("upgrade: Melakukan pembaruan aplikasi.");
                println!("force: Memaksa pembaruan meskipun versi terbaru sudah terpasang.");
                println!("test: Menguji koneksi ke URL tertentu.");
                println!("slim: Mengaktifkan fitur eksperimental (hanya untuk pengujian).");
                return;
            }
            "slim" => {
                println!("Fitur eksperimental aktif");
                slim = true;
                force = true;
            }
            "upgrade" => {
                if let Err(e) = run_updater() {
                    println!("Gagal melakukan update: {}", e);
                }
                return;
            }
            "force" => {
                force = true;
            }
            "test" => {
                println!("Test mode aktif");
                let mpp = match test("https://shp.ee/wtuna3t6555").await {
                    Ok(response) => response,
                    Err(e) => format!("Gagal: {}", e),
                };
                println!("Test URL: {}", mpp);
                return;
            }
            _ => {}
        }
    }
    println!("Versi saat ini: {}", CURRENT_VERSION);
    let os = get_os();
    let arch = if ARCH == "x86"{
        "i686"
    }else{
        ARCH
    };
    let archive = if slim {
        "7z"
    } else if OS == "windows" {
        "zip"
    } else {
        "tar.gz"
    };
    if let Some(latest_version) = get_latest_release().await {
        println!("Versi terbaru: {}", latest_version);
        if compare_versions(CURRENT_VERSION, &latest_version) == Ordering::Less || force {
            println!("Versi baru tersedia! Mengunduh...");
            let download_url = format!("https://github.com/{}/{}/releases/download/v{}/ABS_{}-{}-v{}.{}", REPO_OWNER, REPO_NAME, latest_version, os, arch, latest_version, archive);
            if download_latest_release(&download_url).await.is_ok() {
                println!("Unduhan selesai. Simpan sebagai: {}", *OUTPUT_PATH);
                if let Err(e) = extract_archive(slim).await {
                    println!("Gagal mengekstrak arsip: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
                println!("Ekstraksi selesai. Updater akan upgrade dengan versi terbaru.");
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                println!("Restarting..");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                std::process::exit(0);
            } else {
                println!("Gagal mengunduh update.");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }else {
            println!("Versi terbaru sudah terpasang.");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    } else {
        println!("Gagal mengecek versi terbaru.");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn test(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .redirect(RedirectPolicy::limited(10))
        .danger_accept_invalid_certs(true)
        .impersonate(Impersonate::Chrome130)
        .enable_ech_grease(true)
        .permute_extensions(true)
        .gzip(true)
        .build()?;

    let res = client.get(url)
        .send()
        .await?;

    let final_url = res.url().clone();
    println!("Final URL: {}", final_url);
    Ok(final_url.to_string())
}

fn run_updater() -> io::Result<()> {
    use std::fs;
    use std::io;
    use std::path::Path;
    println!("Menjalankan updater...");
    std::thread::sleep(std::time::Duration::from_secs(10));

    // Tentukan folder update dan folder tujuan
    let update_dir = "update-dir";  // Nama folder hasil ekstraksi

    // Fungsi rekursif untuk menyalin semua file dan folder
    fn copy_recursive(from: &Path, to: &Path) -> io::Result<()> {
        if from.is_dir() {
            fs::create_dir_all(to)?;
            for entry in fs::read_dir(from)? {
                let entry = entry?;
                let from_path = entry.path();
                let to_path = to.join(entry.file_name());
                copy_recursive(&from_path, &to_path)?;
            }
        } else if from.is_file() {
            fs::copy(from, to)?;
        }
        Ok(())
    }

    // Copy semua file dan folder dari update-dir ke direktori utama
    let update_path = Path::new(update_dir);
    let target_path = Path::new(".");
    copy_recursive(update_path, target_path)?;
    println!("Berhasil mengganti semua file aplikasi dengan versi baru.");

    std::thread::sleep(std::time::Duration::from_secs(10));

    println!("Update selesai!");
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(&["/C", "start", "launchng.exe"])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg("./abs")
            .spawn();
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn get_os() -> &'static str {
    use windows_version::OsVersion;
    let version = OsVersion::current();
    if version >= OsVersion::new(10, 0, 0, 10240) {
        OS
    } else if version >= OsVersion::new(6, 1, 0, 7600) {
        "windows7"
    } else if version >= OsVersion::new(5, 1, 0, 2600) {
        "windowsxp"
    } else {
        OS
    }
}

#[cfg(not(target_os = "windows"))]
fn get_os() -> &'static str {
    OS
}