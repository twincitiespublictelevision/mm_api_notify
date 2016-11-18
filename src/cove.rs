extern crate hyper;
extern crate url;
extern crate time;
extern crate crypto;
extern crate rustc_serialize;
extern crate rand;

use self::url::Url;
use self::crypto::md5::Md5;
use self::crypto::sha1::Sha1;
use self::crypto::digest::Digest;
use self::crypto::hmac::Hmac;
use self::crypto::mac::Mac;
use self::hyper::client::Client;
use std::str;
use std::io::Read;
use self::rustc_serialize::hex::ToHex;
use self::rustc_serialize::json::Json;

const API_ID: &'static str = "KTCA-ad82ca26-2d29-47f5-b4e7-24605cc834fa";
const API_SECRET: &'static str = "9dc5083a-df6b-4c48-96c8-e32c2ad12720";

///
/// Makes an API call
///
pub fn video_api(endpoint: &str, params: Vec<[&str; 2]>) -> Json {
    let mut url = format!("http://api.pbs.org/cove/v1/{}/?", endpoint);
    let mut params_str:String = String::from("");

    for param in params {
        params_str = format!("{}{}={}&", params_str, param[0], param[1]);
    }

    let timestamp = time::now().to_timespec().sec;
    let mut md5 = Md5::new();
    let random_int:u32 = rand::random();
    md5.input_str(random_int.to_string().as_str());
    let nonce = md5.result_str();
    url = normalize_url(format!("{}{}consumer_key={}&timestamp={}&nonce={}", url, params_str, API_ID, timestamp, nonce));
    let signature = calc_signature(&url, timestamp, nonce);
    let client = Client::new();
    let mut res = client.get(format!("{}&signature={}", url, signature).as_str()).send().unwrap();
    let mut data = Vec::new();
    res.read_to_end(&mut data).unwrap();
    let data = Json::from_str(String::from_utf8(data).unwrap().as_str()).unwrap();

    data
}

/// 
/// Calculates the signature necessary to call the API
///
fn calc_signature<'a>(url: &String, timestamp: i64, nonce: String) -> String {
    let string_to_sign = format!("GET{}{}{}{}", url, timestamp, API_ID, nonce);
    let mut hmac = Hmac::new(Sha1::new(), Vec::from(API_SECRET).as_slice());
    hmac.input(string_to_sign.as_bytes());

    hmac.result().code().to_hex()
}

///
/// Puts the URL in the format that COVE needs with the query params sorted.
///
fn normalize_url(url: String) -> String {
    let parts = Url::parse(url.as_str()).unwrap();
    let query: &str = parts.query().unwrap();
    let mut vec_query: Vec<&str> = query.split("&").collect();
    vec_query.sort();
    
    format!("{}://{}{}?{}", parts.scheme(), parts.host().unwrap(), parts.path(), vec_query.join("&"))
}