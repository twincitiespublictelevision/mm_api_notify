extern crate rustc_serialize;
extern crate mongodb;

use self::rustc_serialize::json::Json;
use self::mongodb::db::{Database, ThreadedDatabase};
use std::fmt;
use types::ThreadedAPI;
use error::IngestResult;
use error::IngestError;
use objects::object::Object;

pub struct Ref {
    pub id: Json,
    pub attributes: Json,
    pub links: Json,
    pub ref_type: Json,
}

impl Ref {
    pub fn new(id: &Json, attributes: &Json, links: &Json, ref_type: &Json) -> Ref {
        Ref {
            id: id.clone(),
            attributes: attributes.clone(),
            links: links.clone(),
            ref_type: ref_type.clone(),
        }
    }

    pub fn from_json(json: &Json) -> Ref {
        let default = Json::from_str("{}").unwrap();

        let id = json.find("id").unwrap_or(&default);
        let attributes = json.find("attributes").unwrap_or(&default);
        let links = json.find("links").unwrap_or(&default);
        let ref_type = json.find("type").unwrap_or(&default);

        Ref::new(id, attributes, links, ref_type)
    }

    pub fn import(&self, api: &ThreadedAPI, db: &Database, import_refs: bool) {
        let lookup_result = self.to_object(api);

        match lookup_result {
            Ok(obj) => obj.import(api, db, import_refs),
            Err(_) => (),
        };
    }

    pub fn to_object(&self, api: &ThreadedAPI) -> IngestResult<Object> {
        let object_type = self.ref_type.as_string().unwrap_or("");
        let object_id = self.id.as_string().unwrap_or("");

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
                match Json::from_str(json_string.as_str()) {
                    Ok(json) => {
                        let data = json.find("data");

                        match data {
                            Some(data_json) => {
                                let default = Json::from_str("{}").unwrap();

                                let id = data_json.find("id").unwrap_or(&default);
                                let attr = data_json.find("attributes").unwrap_or(&default);
                                let links = data_json.find("links").unwrap_or(&default);
                                let obj_type = data_json.find("type").unwrap_or(&default);

                                Ok(Object::new(id, attr, links, obj_type))
                            }
                            None => Err(IngestError::InvalidObjDataError),
                        }

                        // json.find("data").map_or(Err(IngestError::InvalidObjDataError),
                        //                          |data_json| Ok(Object::new(data_json)))
                    }
                    Err(err) => Err(IngestError::Parse(err)),
                }
            }
            Err(err) => Err(err),
        }
    }

    pub fn value(&self, property: &str) -> Option<&str> {
        self.attributes.find(property).map_or(None, |type_value| type_value.as_string())
    }
}

impl fmt::Display for Ref {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.attributes.fmt(f)
    }
}
