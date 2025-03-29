use rquest as reqwest;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::redirect::Policy as RedirectPolicy;
use serde_json::Value;
use tokio::fs::File;
use tokio::io::{self, BufWriter, AsyncWriteExt};
use std::cmp::Ordering;
use indicatif::{ProgressBar, ProgressStyle};
use futures_util::StreamExt;
use std::time::Instant;
use std::path::Path;

const REPO_OWNER: &str = "cari-404"; // Ganti dengan pemilik repo
const REPO_NAME: &str = "ABS"; // Ganti dengan nama repo
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION"); // Versi lokal aplikasi
const OS: &str = std::env::consts::OS;
const ARCH: &str = std::env::consts::ARCH;
#[cfg(target_os = "windows")]
const OUTPUT_PATH: &str = "update.zip"; // Nama file hasil unduhan di Windows
#[cfg(not(target_os = "windows"))]
const OUTPUT_PATH: &str = "update.tar.gz"; // Nama file hasil unduhan di Linux/MacOS

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
    println!("URL unduhan: {}", url);
    let client = ClientBuilder::new()
        .gzip(true)
        .redirect(RedirectPolicy::limited(10))
        .build()
        .expect("Failed to Create Client");
    let response = client.get(url)
        .send()
        .await
        .expect("Failed to download");
    println!("Status: {}", response.status());
    let total_size = response.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {bytes_per_sec}")
        .expect("Invalid template")
        .progress_chars("#>-"));

    let mut file = BufWriter::new(File::create(OUTPUT_PATH).await?);
    let mut stream = response.bytes_stream();
    let start = Instant::now();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.expect("Error while downloading");
        file.write_all(&chunk).await?;
        pb.inc(chunk.len() as u64);
        let elapsed = start.elapsed().as_secs_f64();
        let speed = (pb.position() as f64 / elapsed) / 1024.0;
        pb.set_message(format!("{:.2} KB/s", speed));
    }
    pb.finish_with_message("Download selesai!");
    Ok(())
}

#[cfg(target_os = "windows")]
async fn extract_archive() -> io::Result<()> {
    use zip::ZipArchive;
    use std::io::Read;

    let file = tokio::fs::File::open(OUTPUT_PATH).await?;
    let mut archive = ZipArchive::new(file.into_std().await)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = Path::new(".").join(file.name());
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
    Ok(())
}

#[cfg(not(target_os = "windows"))]
async fn extract_archive() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::path::PathBuf;
    use std::fs::{self, File};
    use std::os::unix::fs::PermissionsExt;
    use flate2::read::GzDecoder;
    use tar::Archive;
    use tokio::task;

    task::spawn_blocking(|| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let file = File::open(OUTPUT_PATH)?;
        let decompressed = GzDecoder::new(file);
        let mut archive = Archive::new(decompressed);
        archive.unpack(Path::new("."))?;
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
        make_all_executable(Path::new("."))?;
        Ok(())
    })
    .await??;
    println!("Please Wait....");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("Overwrite permission files");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    println!("Almost Done");

    Ok(())
}


#[tokio::main]
async fn main() {
    println!("Versi saat ini: {}", CURRENT_VERSION);
    let os = get_os();
    let arch = if ARCH == "x86"{
        "i686"
    }else{
        ARCH
    };
    let archive = if OS == "windows" {
        "zip"
    } else {
        "tar.gz"
    };
    if let Some(latest_version) = get_latest_release().await {
        println!("Versi terbaru: {}", latest_version);
        match compare_versions(CURRENT_VERSION, &latest_version) {
            Ordering::Less => {
                println!("Versi baru tersedia! Mengunduh...");
                let download_url = format!("https://github.com/{}/{}/releases/download/v{}/ABS_{}-{}-v{}.{}", REPO_OWNER, REPO_NAME, latest_version, os, arch, latest_version, archive);
                if download_latest_release(&download_url).await.is_ok() {
                    println!("Unduhan selesai. Simpan sebagai: {}", OUTPUT_PATH);
                    if let Err(e) = extract_archive().await {
                        println!("Gagal mengekstrak arsip: {}", e);
                    }
                    println!("Ekstraksi selesai. Silakan jalankan aplikasi baru.");
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
                } else {
                    println!("Gagal mengunduh update.");
                }
            }
            _ => println!("Aplikasi sudah versi terbaru."),
        }
    } else {
        println!("Gagal mengecek versi terbaru.");
    }
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