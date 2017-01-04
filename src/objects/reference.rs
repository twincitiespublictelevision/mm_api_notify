extern crate mongodb;
extern crate serde;
extern crate serde_json;
extern crate chrono;

use self::chrono::DateTime;
use self::chrono::UTC;
use self::serde_json::Value as Json;
use self::serde_json::error::Result as JsonResult;
use self::mongodb::db::{Database, ThreadedDatabase};
use std::fmt;
use types::ThreadedAPI;
use error::IngestResult;
use error::IngestError;
use objects::object::Object;

pub struct Ref {
    pub id: Json,
    pub attributes: Json,
    pub ref_type: Json,
}

impl Ref {
    pub fn new(id: &Json, attributes: &Json, ref_type: &Json) -> Ref {
        Ref {
            id: id.clone(),
            attributes: attributes.clone(),
            ref_type: ref_type.clone(),
        }
    }

    pub fn from_json(json: &Json) -> IngestResult<Ref> {

        let id = json.find("id");
        let attributes = json.find("attributes");
        let ref_type = json.find("type");

        match and_list(vec![id, attributes, ref_type]) {
            Some(mut value_list) => {
                Ok(Ref::new(value_list.remove(0),
                            value_list.remove(0),
                            value_list.remove(0)))
            }
            None => Err(IngestError::InvalidDocumentDataError),
        }
    }

    pub fn import(&self,
                  api: &ThreadedAPI,
                  db: &Database,
                  import_refs: bool,
                  run_start_time: i64,
                  path_from_root: Vec<String>) {

        // Optimization: Asset types can not have child elements so if they are not going to be
        // updated, then do not perform a lookup
        let ref_type = self.ref_type.as_str().unwrap_or("");

        let obj_lookup = match ref_type {
            "asset" => {
                let updated_at = self.attributes.find("updated_at");

                match updated_at {
                    Some(date_string) => {
                        let date_str = date_string.as_str().unwrap_or("");
                        let updated_at_time = date_str.parse::<DateTime<UTC>>();

                        match updated_at_time {
                            Ok(value) => {
                                if value.timestamp() > run_start_time {
                                    Some(self.to_object(api))
                                } else {
                                    None
                                }
                            }
                            Err(_) => Some(self.to_object(api)),
                        }
                    }
                    None => Some(self.to_object(api)),
                }
            }
            _ => Some(self.to_object(api)),
        };

        match obj_lookup {
            Some(Ok(obj)) => obj.import(api, db, import_refs, run_start_time, path_from_root),
            Some(Err(_)) => (),
            None => (),
        };
    }

    pub fn to_object(&self, api: &ThreadedAPI) -> IngestResult<Object> {
        let object_type = self.ref_type.as_str().unwrap_or("");
        let object_id = self.id.as_str().unwrap_or("");

        if object_id == "" {
            return Err(IngestError::InvalidRefDataError);
        }

        let response = match object_type {
            "asset" => {
                api.assets()
                    .get(object_id)
                    .map_err(IngestError::API)
            }
            "collection" => {
                api.collections()
                    .get(object_id)
                    .map_err(IngestError::API)
            }
            "episode" => {
                api.episodes()
                    .get(object_id)
                    .map_err(IngestError::API)
            }
            "franchise" => {
                api.franchises()
                    .get(object_id)
                    .map_err(IngestError::API)
            }
            "season" => {
                api.seasons()
                    .get(object_id)
                    .map_err(IngestError::API)
            }
            "show" => {
                api.shows()
                    .get(object_id)
                    .map_err(IngestError::API)
            }
            "special" => {
                api.specials()
                    .get(object_id)
                    .map_err(IngestError::API)
            }
            _ => Err(IngestError::InvalidRefDataError),
        };

        match response {
            Ok(json_string) => {
                let full_json: JsonResult<Json> = serde_json::from_str(json_string.as_str());
                match full_json {
                    Ok(mut json) => {
                        let json_map = json.as_object_mut();

                        match json_map {
                            Some(map) => {
                                let data = map.remove("data");

                                match data {
                                    Some(data_value) => Object::from_json(data_value),
                                    None => Err(IngestError::InvalidObjDataError),
                                }
                            }
                            None => Err(IngestError::InvalidObjDataError),
                        }
                    }
                    Err(err) => Err(IngestError::Parse(err)),
                }
            }
            Err(err) => Err(err),
        }
    }

    pub fn value(&self, property: &str) -> Option<&str> {
        self.attributes.find(property).map_or(None, |type_value| type_value.as_str())
    }
}

impl fmt::Display for Ref {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.attributes.fmt(f)
    }
}

fn and_list<T>(options: Vec<Option<T>>) -> Option<Vec<T>> {

    let state = Some(Vec::new());

    options.into_iter().fold(state, |result, option| {
        result.and_then(|mut list| {
            option.and_then(|value| {
                list.push(value);
                Some(list)
            })
        })
    })
}
