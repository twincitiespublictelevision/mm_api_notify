extern crate mongodb;
extern crate serde;
extern crate serde_json;
extern crate chrono;

use self::chrono::DateTime;
use self::chrono::UTC;
use self::serde_json::Value as Json;
use self::mongodb::db::Database;
use std::fmt;
use types::ThreadedAPI;
use error::IngestResult;
use error::IngestError;
use objects::object::Object;
use objects::parser::parse_response;
use core_data_client::Endpoints;
use objects::import::Importable;

#[derive(Debug, PartialEq)]
pub struct Ref {
    pub id: Json,
    pub attributes: Json,
    pub ref_type: Json,
}

impl Ref {
    pub fn new(id: Json, attributes: Json, ref_type: Json) -> Ref {
        Ref {
            id: id,
            attributes: attributes,
            ref_type: ref_type,
        }
    }
    pub fn from_json(mut json: Json) -> IngestResult<Ref> {
        let json_map = json.as_object_mut();

        match json_map {
            Some(map) => {

                let id = map.remove("id");
                let attributes = map.remove("attributes");
                let ref_type = map.remove("type");

                match and_list(vec![id, attributes, ref_type]) {
                    Some(mut value_list) => {
                        Ok(Ref::new(value_list.remove(0),
                                    value_list.remove(0),
                                    value_list.remove(0)))
                    }
                    None => Err(IngestError::InvalidRefDataError),
                }
            }
            None => Err(IngestError::InvalidRefDataError),
        }
    }

    pub fn to_object(&self, api: &ThreadedAPI) -> IngestResult<Object> {
        let object_type = self.ref_type.as_str().unwrap_or("");
        let object_id = self.id.as_str().unwrap_or("");

        if object_id == "" {
            return Err(IngestError::InvalidRefDataError);
        }

        let response = match object_type {
            "asset" => Ok(api.get(Endpoints::Asset, object_id)),
            "collection" => Ok(api.get(Endpoints::Collection, object_id)),
            "episode" => Ok(api.get(Endpoints::Episode, object_id)),
            "franchise" => Ok(api.get(Endpoints::Franchise, object_id)),
            "season" => Ok(api.get(Endpoints::Season, object_id)),
            "show" => Ok(api.get(Endpoints::Show, object_id)),
            "special" => Ok(api.get(Endpoints::Special, object_id)),
            _ => Err(IngestError::InvalidRefDataError),
        };

        response.and_then(|resp| {
            parse_response(resp).and_then(|mut json| {
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
            })
        })
    }

    pub fn value(&self, property: &str) -> Option<&str> {
        self.attributes.find(property).map_or(None, |type_value| type_value.as_str())
    }
}

impl Importable for Ref {
    // TODO: Refactor path_from_root to be a reference that is cloned
    // on mutation. Likely needs to be place in an ARC
    fn import(&self,
              api: &ThreadedAPI,
              db: &Database,
              import_refs: bool,
              run_start_time: i64,
              path_from_root: &Vec<String>) {

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
