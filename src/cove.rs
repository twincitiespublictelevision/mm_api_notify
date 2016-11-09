extern crate curl;
extern crate url;
extern crate time;
extern crate crypto;

use url::{Url, ParseError};
use crypto::md5::Md5;
use crypto::sha2::Sha1;

const API_ID: &'static str = "KTCA-ad82ca26-2d29-47f5-b4e7-24605cc834fa"
const API_KEY: &'static str = "9dc5083a-df6b-4c48-96c8-e32c2ad12720"

///
/// Makes an API call
///
pub fn video_api<'a>(endpoint: &str, filters: Vec<[&str; 2]>, fields: Vec<&str>) -> &'a str {
    let mut url = format!("http://api.pbs.org/cove/v1/{}", endpoint);

    for filter in filters {
        url = format!("{}&{}={}", url, filter[0], filter[1]);
    }

    let dst = Vec::new();
    let mut easy = Easy::new();
    easy.url(url).unwrap();

    let mut transfer = easy.transfer();
    transfer.write_function(|data| {
        dst.extend_from_slice(data);
    }).unwrap();
    transfer.perform().unwrap();

    return dst;
}

/// 
/// Calculates the signature necessary to call the API
///
fn calc_signature(url: &str) - &str {
    let md5 = Md5::new();
    md5.input_str(time::now());

    let string_to_sign = format!("GET{}{}{}{}", url, time::now(), API_ID, md5.result_str());
    let sha1 = Sha1::new();
    sha1.input_str(string_to_sign);

    sha1.result_str()
}

///
/// Puts the URL in the format that COVE needs with the query params sorted.
///
fn normalize_url(url: &str) -> &str {
    let parts = Url::parse(url).unwrap();
    let query = parts.query().split("&").collect().sort().join("&")

    format!("{}://{}/{}?{}", parts.scheme(), parts.host(), parts.path(), query)
}