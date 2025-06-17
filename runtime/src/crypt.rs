use std::io::Cursor;
use base64::encode;
use byteorder::{BigEndian, WriteBytesExt};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuyerPaymentInfo {
    pub is_jko_app_installed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GpsLocationInfo {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_fingerprint: String,
    pub device_sz_fingerprint: String,
    pub tongdun_blackbox: String,
    pub buyer_payment_info: BuyerPaymentInfo,
    pub gps_location_info: GpsLocationInfo,
}

#[derive(Debug)]
pub struct W {
    pub f3733a: i64,
}

impl W {
    pub fn new() -> Self {
        let random_uuid = uuid::Uuid::new_v4();
        let most_significant_bits = random_uuid.as_u128() >> 64;
        let least_significant_bits = random_uuid.as_u128() & 0xFFFFFFFFFFFFFFFF;
        let combined = (most_significant_bits ^ least_significant_bits) as i64;
        let f3733a = if combined < 0 { -combined } else { combined };
        W { f3733a }
    }
}

pub fn generate_device_id() -> String {
    let w_arr = [W::new(), W::new(), W::new(), W::new()];
    let mut buffer = Cursor::new(vec![0u8; 32]);
    for w in w_arr.iter() {
        buffer.write_i64::<BigEndian>(w.f3733a).unwrap();
    }
    let encoded_string = encode(buffer.into_inner());
    encoded_string 
}

/*fn main() {
    let device_id = generate_device_id();
    println!("Generated Device ID: {}", device_id);
    let device_fingerprint = generate_device_fingerprint();
    println!("Generated Device Fingerprint: {}", device_fingerprint);
}*/

pub fn generate_device_fingerprint() -> String {
    format!("{}_unknown", random_hex_string(16))
}

pub fn random_hex_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    let charset = b"0123456789abcdef";
    let mut result = String::with_capacity(len);
    for _ in 0..len {
        let idx = rng.gen_range(0..16);
        result.push(charset[idx] as char);
    }
    result
}

pub fn create_devices(fp: &str) -> DeviceInfo {
    let device_id = generate_device_id();
    let device_fingerprint = generate_device_fingerprint();
    let device_info = DeviceInfo {
        device_id,
        device_fingerprint,
        device_sz_fingerprint: fp.to_string(),
        tongdun_blackbox: "td_disable_for_ID".to_string(),
        buyer_payment_info: BuyerPaymentInfo {
            is_jko_app_installed: false,
        },
        gps_location_info: GpsLocationInfo {},
    };
    println!("{:?}", device_info);
    device_info
}

pub fn generate_csrftoken(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

fn to_hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
// Documentation for get_x_sap_ri function more on https://xsblog.site/post/6 or https://blog.csdn.net/dxxmsl/article/details/140381283 
pub fn get_x_sap_ri() -> (String, u32, u8) {
    // Timestamp dalam detik
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    // Buffer 26 byte
    let mut buffer = [0u8; 26];

    // Simpan timestamp ke posisi 0..4 (little endian)
    buffer[0..4].copy_from_slice(&timestamp.to_le_bytes());

    // 22 byte acak
    let random_bytes: Vec<u8> = (0..22).map(|_| rand::random()).collect();
    buffer[4..26].copy_from_slice(&random_bytes);

    // Masukkan ke buffer mulai dari index 4
    buffer[4..].copy_from_slice(&random_bytes);

    // Modifikasi byte ke-11 (index ke-11 dalam buffer berarti byte ke-7 dari random_array)
    let mut magic = buffer[11];
    magic = (3 << 4) + (15 & magic);
    buffer[11] = magic;

    // Set nilai tetap
    buffer[12] = 3;
    buffer[13] = 1;

    // Konversi ke hex string
    let hex_string = to_hex_string(&buffer);

    // Return hasil
    (hex_string, timestamp, random_bytes[0])
}