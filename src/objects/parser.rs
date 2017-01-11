extern crate serde_json;

use self::serde_json::Value as Json;
use error::IngestResult;
use error::IngestError;
use core_data_client::CDCResult;

pub fn parse_response(response: CDCResult<String>) -> IngestResult<Json> {
    match response.map_err(IngestError::API) {
        Ok(json_string) => serde_json::from_str(json_string.as_str()).map_err(IngestError::Parse),
        Err(err) => Err(err),
    }
}
