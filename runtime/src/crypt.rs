use std::io::Cursor;
use base64::encode;
use byteorder::{BigEndian, WriteBytesExt};
use rand::Rng;
use serde::{Serialize, Deserialize};

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

    let mut buffer = Cursor::new(vec![0u8; 32]); // Mengalokasikan buffer 32 byte

    for w in w_arr.iter() {
        buffer.write_i64::<BigEndian>(w.f3733a).unwrap(); // Menulis nilai `f3733a` ke buffer
    }

    let encoded_string = encode(buffer.into_inner()); // Mengencode buffer ke Base64
    //println!("GenDeviceId: {}", &encoded_string); // Log hasil encoding

    encoded_string // Mengembalikan string hasil encoding
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

    println!("{:?}", device_info); // Debug print untuk output struct
    device_info
}