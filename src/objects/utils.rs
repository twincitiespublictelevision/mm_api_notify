extern crate serde_json;

use self::serde_json::Value as Json;
use core_data_client::CDCResult;

use error::IngestResult;
use error::IngestError;

pub fn parse_response(response: CDCResult<String>) -> IngestResult<Json> {
    match response.map_err(IngestError::API) {
        Ok(json_string) => serde_json::from_str(json_string.as_str()).map_err(IngestError::Parse),
        Err(err) => Err(err),
    }
}
