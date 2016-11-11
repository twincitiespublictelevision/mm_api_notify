extern crate hyper;
extern crate url;
extern crate time;
extern crate crypto;

use self::url::Url;
use self::crypto::md5::Md5;
use self::crypto::sha1::Sha1;
use self::crypto::digest::Digest;
use self::crypto::hmac::Hmac;
use self::crypto::mac::Mac;
use self::hyper::client::Client;
use std::str;
use std::io::Read;

const API_ID: &'static str = "KTCA-ad82ca26-2d29-47f5-b4e7-24605cc834fa";
const API_SECRET: &'static str = "9dc5083a-df6b-4c48-96c8-e32c2ad12720";

///
/// Makes an API call
///
pub fn video_api(endpoint: &str, filters: Vec<[&str; 2]>, fields: Vec<&str>) -> String {
    let mut url = format!("http://api.pbs.org/cove/v1/{}", endpoint);
    let mut filter_str:String = String::from("");

    for filter in filters {
        let separator = if filter_str == "" { "?" } else { "&amp" };
        filter_str = format!("{}{}{}={}", filter_str, separator, filter[0], filter[1]);
    }

    let timestamp = time::now().to_timespec().sec;
    let mut md5 = Md5::new();
    md5.input_str(timestamp.to_string().as_str());
    let nonce = md5.result_str();
    let separator = if filter_str == "" { "?" } else { "&amp" };
    url = normalize_url(format!("{}{}consumer_key={}&amp;timestamp={}&amp;nonce={}", url, separator, API_ID, timestamp, nonce));
    let signature = calc_signature(&url, timestamp, nonce);

    let client = Client::new();
    let mut res = client.get(format!("{}&amp;signature={}", url, signature).as_str()).send().unwrap();
    let mut data = Vec::new();
    res.read_to_end(&mut data).unwrap();

    String::from_utf8(data).unwrap()
}

/// 
/// Calculates the signature necessary to call the API
///
fn calc_signature<'a>(url: &String, timestamp: i64, nonce: String) -> String {
    let string_to_sign = format!("GET{}{}{}{}", url, timestamp, API_ID, nonce);
    let mut hmac = Hmac::new(Sha1::new(), Vec::from(API_SECRET).as_slice());
    hmac.input(string_to_sign.as_bytes());

    str::from_utf8(hmac.result().code()).unwrap().to_string()
}

///
/// Puts the URL in the format that COVE needs with the query params sorted.
///
fn normalize_url(url: String) -> String {
    let parts = Url::parse(url.as_str()).unwrap();
    let query: &str = parts.query().unwrap();
    let mut vec_query: Vec<&str> = query.split("&").collect();
    vec_query.sort();
    
    format!("{}://{}/{}?{}", parts.scheme(), parts.host().unwrap(), parts.path(), vec_query.join("&"))
}