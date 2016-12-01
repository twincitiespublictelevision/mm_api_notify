extern crate rustc_serialize;
extern crate core_data_client;

use self::rustc_serialize::json::Json;
use core_data_client::Client;
use std::fmt;
use error::IngestResult;
use error::IngestError;
use objects::object::Object;

pub struct Ref {
    pub data: Json,
}

impl Ref {
    pub fn new(data: &Json) -> Ref {
        Ref { data: data.clone() }
    }

    pub fn to_object(&self, api: &Client) -> IngestResult<Object> {
        let object_type = self.value("type").unwrap_or("");
        let object_id = self.value("id").unwrap_or("");

        if object_id == "" {
            return Err(IngestError::InvalidRefDataError);
        }

        let response = match object_type {
            "episode" => {
                api.episodes()
                    .get(object_id)
                    .map_err(IngestError::API)
            }
            "season" => {
                api.seasons()
                    .get(object_id)
                    .map_err(IngestError::API)
            }
            _ => Err(IngestError::InvalidRefDataError),
        };

        match response {
            Ok(json_string) => {
                match Json::from_str(json_string.as_str()) {
                    Ok(json) => {
                        json.find("data").map_or(Err(IngestError::InvalidObjDataError),
                                                 |data_json| Ok(Object::new(data_json)))
                    }
                    Err(err) => Err(IngestError::Parse(err)),
                }
            }
            Err(err) => Err(err),
        }
    }

    pub fn value(&self, property: &str) -> Option<&str> {
        self.data.find(property).map_or(None, |type_value| type_value.as_string())
    }
}

impl fmt::Display for Ref {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.data.fmt(f)
    }
}
