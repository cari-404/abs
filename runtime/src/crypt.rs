use std::io::Cursor;
use base64::encode;
use byteorder::{BigEndian, WriteBytesExt};
use rand::Rng;
use serde_json::json;

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

fn generate_device_fingerprint() -> String {
    format!("{}_unknow", random_hex_string(16))
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

pub fn create_devices(fp: &str) -> serde_json::Value {
    let device_id = generate_device_id();
    let device_fingerprint = generate_device_fingerprint();
    let body_json = json!({
        "device_id": device_id,
        "device_fingerprint": device_fingerprint,
        "device_sz_fingerprint": fp,
        "tongdun_blackbox": "td_disable_for_ID",
        "buyer_payment_info": {
            "is_jko_app_installed": false
        },
        "gps_location_info": {}
    });
    println!("{body_json}");
    body_json
}