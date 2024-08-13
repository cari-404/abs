use rquest as reqwest;
use reqwest::impersonate::Impersonate;
use reqwest::{ClientBuilder, header::HeaderMap, Error, Response, Version, StatusCode};
use reqwest::header::HeaderValue;
use serde::{Serialize, Deserialize};
use std::process;
use serde_json::Value;
use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::io;

crate prepare::{self, ModelInfo, ShippingInfo, PaymentInfo};

pub async fn get_recommend_platform_vouchers(cookie_content: &str, ) -> Result<(String), Box<dyn std::error::Error>>{
    let body_json = json!({

      });
      // Convert struct to JSON
      let body_str = serde_json::to_string(&body_json).unwrap();
      let body = Body::from(body_str.clone());
      println!("{:?}", body_str);
      //println!("Request Headers:\n{:?}", headers);
  
      let url2 = format!("https://mall.shopee.co.id/api/v4/checkout/get");
      println!("{}", url2);
      // Buat klien HTTP
      let client = ClientBuilder::new()
          .danger_accept_invalid_certs(true)
          .impersonate(Impersonate::Cronet)
          .enable_ech_grease()
          .permute_extensions()
          .gzip(true)
          //.use_boring_tls(boring_tls_connector) // Use Rustls for HTTPS
          .build()?;
  
      // Buat permintaan HTTP POST
      let response = client
          .post(&url2)
          .header("Content-Type", "application/json")
          .headers(headers)
          .body(body)
          .version(Version::HTTP_2) 
          .send()
          .await?;
}