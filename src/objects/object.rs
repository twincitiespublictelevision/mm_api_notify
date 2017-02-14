extern crate bson;
extern crate chrono;
extern crate rayon;
extern crate serde_json;

use self::bson::{Bson, Document};
use self::chrono::{DateTime, UTC};
use self::rayon::prelude::*;
use self::serde_json::Value as Json;

use std::fmt;

use api::Payload;
use error::IngestResult;
use error::IngestError;
use objects::Collection;
use objects::Importable;
use objects::Ref;
use objects::utils;
use runtime::Runtime;
use types::{ImportResult, ThreadedAPI, ThreadedStore};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Object {
    #[serde(rename = "_id")]
    pub id: String,
    pub attributes: Json,
    #[serde(rename = "type")]
    pub object_type: String,
    pub links: Json,
}

impl Object {
    pub fn new(id: String, attributes: Json, object_type: String, links: Json) -> Object {
        Object {
            id: id,
            attributes: attributes,
            object_type: object_type,
            links: links,
        }
    }

    pub fn parent(&self, store: &ThreadedStore) -> Option<Object> {
        vec!["episode", "season", "special", "show", "franchise"]
            .iter()
            .filter_map(|parent_type| {
                self.attributes.lookup(parent_type).and_then(|parent| Ref::from_json(parent).ok())
            })
            .filter_map(|parent_ref| {
                store.get(parent_ref.id.as_str(), parent_ref.ref_type.as_str())
            })
            .collect::<Vec<Object>>()
            .pop()
    }

    pub fn from_bson(bson: Bson) -> Option<Object> {
        bson::from_bson(utils::map_bson_dates_to_string(bson)).ok()
    }

    pub fn as_document(&self) -> IngestResult<Document> {

        match self.as_bson() {
            Ok(serialized) => {
                if let bson::Bson::Document(document) = serialized {
                    Ok(document)
                } else {
                    Err(IngestError::InvalidDocumentDataError)
                }
            }
            Err(err) => Err(IngestError::Serialize(err)),
        }
    }

    fn as_bson(&self) -> bson::EncoderResult<Bson> {
        bson::to_bson(&self).map(utils::map_string_to_bson_dates)
    }

    fn import_children(&self, runtime: &Runtime, follow_refs: bool, since: i64) -> ImportResult {

        vec!["assets", "episodes", "seasons", "shows", "specials"]
            .par_iter()
            .map(|child_type| {
                self.child_collection(&runtime.api, child_type)
                    .and_then(|child_collection| {
                        Some(child_collection.import(runtime, follow_refs, since))
                    })
                    .unwrap_or((0, 1))
            })
            .reduce(|| (0, 0), |(p1, f1), (p2, f2)| (p1 + p2, f1 + f2))
    }

    fn child_collection(&self, api: &ThreadedAPI, child_type: &str) -> Option<Collection> {
        let mut url = self.links.find("self").unwrap().as_str().unwrap().to_string();

        url.push_str(child_type);
        url.push('/');
        utils::parse_response(api.url(url.as_str()))
            .and_then(|api_json| Collection::from_json(&api_json))
            .ok()
    }
}

impl Importable for Object {
    type Value = Object;

    fn import(&self, runtime: &Runtime, follow_refs: bool, since: i64) -> ImportResult {

        if runtime.verbose {
            println!("Importing {} {} {}",
                     self.id,
                     self.object_type,
                     self.attributes.lookup("title").unwrap().as_str().unwrap());
        }

        let updated_at_time = self.attributes
            .find("updated_at")
            .and_then(|update_string| update_string.as_str())
            .and_then(|updated_str| updated_str.parse::<DateTime<UTC>>().ok())
            .and_then(|date| Some(date.timestamp()))
            .unwrap_or(0);

        // Check the updated_at date to determine if the db needs to
        // update this object
        let mut update_result = if updated_at_time >= since {

            let res = runtime.store.put(self);

            if res.is_some() && runtime.config.enable_hooks {
                Payload::from_object(self, &runtime.store)
                    .and_then(|payload| Some(payload.emitter(&runtime.config).update()));
            }

            res.map_or_else(|| (0, 1), |_| (1, 0))
        } else {
            (0, 0)
        };

        if follow_refs {
            let child_results = self.import_children(runtime, follow_refs, since);
            update_result = (update_result.0 + child_results.0, update_result.1 + child_results.1);
        };

        update_result
    }

    fn from_json(json: &Json) -> IngestResult<Object> {

        let mut source = json.clone();

        let id = source.lookup("data").and_then(|data| {
            data.lookup("id")
                .and_then(|id| id.as_str().and_then(|id_str| Some(id_str.to_string())))
        });

        let obj_type = source.lookup("data").and_then(|data| {
            data.lookup("type")
                .and_then(|o_type| o_type.as_str().and_then(|type_str| Some(type_str.to_string())))
        });

        source.as_object_mut()
            .and_then(|map| {

                let attrs = map.remove("data").and_then(|mut data| {
                    data.as_object_mut().and_then(|data_map| data_map.remove("attributes"))
                });

                let links = map.remove("links");

                match (id, attrs, obj_type, links) {
                    (Some(p1), Some(p2), Some(p3), Some(p4)) => Some(Object::new(p1, p2, p3, p4)),
                    _ => None,
                }
            })
            .ok_or(IngestError::InvalidObjDataError)
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.attributes.fmt(f)
    }
}
