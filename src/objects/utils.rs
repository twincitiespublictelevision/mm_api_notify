extern crate bson;
extern crate chrono;
extern crate serde_json;

use self::bson::Bson;
use self::chrono::{DateTime, UTC};
use mm_client::MMCResult;
use self::serde_json::Value as Json;

use error::IngestResult;
use error::IngestError;

pub fn parse_response(response: MMCResult<String>) -> IngestResult<Json> {
    match response.map_err(IngestError::API) {
        Ok(json_string) => serde_json::from_str(json_string.as_str()).map_err(IngestError::Parse),
        Err(err) => Err(err),
    }
}

pub fn map_string_to_bson_dates(bson: Bson) -> Bson {
    match bson {
        Bson::String(string) => {
            match string.parse::<DateTime<UTC>>() {
                Ok(datetime) => Bson::UtcDatetime(datetime),
                _ => Bson::String(string),
            }
        }
        Bson::Document(doc) => {
            Bson::Document(doc.into_iter()
                .map(|(key, bson_val)| (key, map_string_to_bson_dates(bson_val)))
                .collect::<bson::Document>())
        }
        Bson::Array(elements) => {
            Bson::Array(elements.into_iter()
                .map(map_string_to_bson_dates)
                .collect::<Vec<Bson>>())
        }
        x => x,
    }
}

pub fn map_bson_dates_to_string(bson: Bson) -> Bson {
    match bson {
        Bson::UtcDatetime(datetime) => {
            Bson::String(datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        }
        Bson::Document(doc) => {
            Bson::Document(doc.into_iter()
                .map(|(key, bson_val)| (key, map_bson_dates_to_string(bson_val)))
                .collect::<bson::Document>())
        }
        Bson::Array(elements) => {
            Bson::Array(elements.into_iter()
                .map(map_bson_dates_to_string)
                .collect::<Vec<Bson>>())
        }
        x => x,
    }
}

#[cfg(test)]
mod tests {
    use bson::Bson;
    use chrono::{DateTime, UTC};
    use objects::utils;

    #[test]
    fn test_utc_datetime_replacement() {
        let test_date = "2017-01-19T14:58:55.121584Z";
        let test_datetime = test_date.parse::<DateTime<UTC>>();
        let test_bson_datetime = Bson::UtcDatetime(test_datetime.unwrap());

        let doc1 = doc!{
            "datetime_string" => test_date
        };

        let doc1_mapped = match utils::map_string_to_bson_dates(Bson::Document(doc1.clone())) {
            Bson::Document(doc) => doc,
            _ => panic!("Mapping Bson::Document resulted in non-Document Bson"),
        };

        let doc2 = doc!{
            "datetime_string" => test_bson_datetime
        };

        assert!(doc1 != doc2);
        assert!(doc1_mapped == doc2);
    }
}
