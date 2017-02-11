extern crate bson;
extern crate chrono;
extern crate mongodb;
extern crate rayon;
extern crate serde;
extern crate serde_json;

use self::bson::Bson;
use self::bson::Document;
use self::chrono::{DateTime, UTC};
use self::mongodb::db::{Database, ThreadedDatabase};
use self::mongodb::coll::options::FindOneAndUpdateOptions;
use self::rayon::prelude::*;
use self::serde_json::Value as Json;

use std::fmt;

use api::Payload;
use config::get_config;
use error::IngestResult;
use error::IngestError;
use objects::Collection;
use objects::Importable;
use objects::Ref;
use objects::utils;
use types::ImportResult;
use types::ThreadedAPI;

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

    pub fn parent(&self, db: &Database) -> Option<Object> {
        vec!["episode", "season", "special", "show", "franchise"]
            .iter()
            .filter_map(|parent_type| {
                self.attributes.lookup(parent_type).and_then(|parent| Ref::from_json(parent).ok())
            })
            .filter_map(|parent_ref| {
                Object::lookup(parent_ref.id.as_str(), parent_ref.ref_type.as_str(), db)
            })
            .collect::<Vec<Object>>()
            .pop()
    }

    fn lookup(id: &str, item_type: &str, db: &Database) -> Option<Object> {
        let query = doc!{
            "_id" => id
        };

        let coll = db.collection(item_type);

        coll.find(Some(query), None).ok().and_then(|mut cursor| match cursor.next() {
            Some(Ok(doc)) => {
                bson::from_bson(utils::map_bson_dates_to_string(Bson::Document(doc))).ok()
            }
            _ => None,
        })
    }

    fn import_children(&self,
                       api: &ThreadedAPI,
                       db: &Database,
                       follow_refs: bool,
                       since: i64)
                       -> ImportResult {

        vec!["assets", "episodes", "seasons", "shows", "specials"]
            .par_iter()
            .map(|child_type| {
                self.child_collection(api, child_type)
                    .and_then(|child_collection| {
                        Some(child_collection.import(api, db, follow_refs, since))
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

    fn as_bson(&self) -> bson::EncoderResult<Bson> {
        bson::to_bson(&self).map(utils::map_string_to_bson_dates)
    }

    fn as_document(&self) -> IngestResult<Document> {

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
}

impl Importable for Object {
    type Value = Object;

    fn import(&self,
              api: &ThreadedAPI,
              db: &Database,
              follow_refs: bool,
              since: i64)
              -> ImportResult {

        println!("Import {} {}",
                 self.id,
                 self.attributes.lookup("title").unwrap().as_str().unwrap());

        let updated_at_time = self.attributes
            .find("updated_at")
            .and_then(|update_string| update_string.as_str())
            .and_then(|updated_str| updated_str.parse::<DateTime<UTC>>().ok())
            .and_then(|date| Some(date.timestamp()))
            .unwrap_or(0);

        // Check the updated_at date to determine if the db needs to
        // update this object
        let update_result = if updated_at_time > since {
            let res = match self.as_document() {
                Ok(doc) => {

                    let coll = db.collection(self.object_type.as_str());
                    let id = self.id.as_str();

                    let filter = doc! {
                            "_id" => id
                        };

                    let mut options = FindOneAndUpdateOptions::new();
                    options.upsert = true;

                    match coll.find_one_and_replace(filter, doc, Some(options)) {
                        Ok(_) => (1, 0),
                        Err(_) => (0, 1),
                    }
                }
                Err(_) => (0, 1),
            };

            if get_config().map_or(false, |conf| conf.enable_hooks) {
                Payload::from_object(&self, db)
                    .and_then(|payload| Some(payload.emitter().update()));
            }

            res
        } else {
            (0, 0)
        };

        if follow_refs {
            self.import_children(api, db, follow_refs, since);
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
