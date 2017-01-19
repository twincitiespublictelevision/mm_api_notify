extern crate bson;
extern crate chrono;
extern crate mongodb;
extern crate rayon;
extern crate serde;
extern crate serde_json;

use self::bson::Bson;
use self::chrono::UTC;
use self::chrono::DateTime;
use self::bson::Document;
use self::mongodb::db::{Database, ThreadedDatabase};
use self::mongodb::coll::options::FindOneAndUpdateOptions;
use self::rayon::prelude::*;
use self::serde_json::Value as Json;

use std::fmt;

use error::IngestResult;
use error::IngestError;
use objects::collection::Collection;
use objects::import::Importable;
use objects::utils;
use types::ThreadedAPI;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Object {
    #[serde(rename = "_id")]
    pub id: String,
    pub attributes: Json,
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

    fn import_children(&self,
                       api: &ThreadedAPI,
                       db: &Database,
                       follow_refs: bool,
                       path_from_root: &Vec<&str>,
                       since: i64) {

        let mut path_for_children = path_from_root.clone();
        path_for_children.push(self.id.as_str());

        vec!["assets", "episodes", "extras", "seasons", "shows", "specials"]
            .par_iter()
            .for_each(|child_type| {
                self.child_collection(api, child_type).and_then(|child_collection| {
                    Some(child_collection.import(api, db, follow_refs, &path_for_children, since))
                });
            });
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
        bson::to_bson(&self)
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
              path_from_root: &Vec<&str>,
              since: i64) {

        println!("Import {}", &self);

        // Check the updated_at date to determine if the db needs to
        // update this object
        let updated_at_time = self.attributes
            .find("updated_at")
            .and_then(|update_string| update_string.as_str())
            .and_then(|updated_str| updated_str.parse::<DateTime<UTC>>().ok())
            .and_then(|date| Some(date.timestamp()))
            .unwrap_or(0);

        if updated_at_time > since {
            match self.as_document() {
                Ok(mut doc) => {

                    // Insert the path from the root in the parents key
                    doc.insert("parents", bson::to_bson(path_from_root).unwrap());

                    let coll = db.collection(self.object_type.as_str());
                    let id = self.id.as_str();

                    let filter = doc! {
                        "_id" => id
                    };

                    let mut options = FindOneAndUpdateOptions::new();
                    options.upsert = true;

                    let res = coll.find_one_and_replace(filter, doc, Some(options));
                }
                Err(_) => (),
            };
        }

        if follow_refs {
            self.import_children(api, db, follow_refs, path_from_root, since);
        }
    }

    fn from_json(json: &Json) -> IngestResult<Object> {

        json.clone()
            .as_object_mut()
            .and_then(|map| {
                let id = map.remove("id")
                    .and_then(|id_val| id_val.as_str().and_then(|id_str| Some(id_str.to_string())));

                let attrs = map.remove("data").and_then(|mut data_val| {
                    data_val.as_object_mut().and_then(|data_map| data_map.remove("attributes"))
                });

                let obj_type = map.remove("type").and_then(|type_val| {
                    type_val.as_str().and_then(|type_str| Some(type_str.to_string()))
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
