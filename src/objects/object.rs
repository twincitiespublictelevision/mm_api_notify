// extern crate rustc_serialize;
extern crate mongodb;
extern crate bson;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate chrono;

// use self::rustc_serialize::json::Json;
use self::chrono::DateTime;
use self::chrono::UTC;
use self::serde_json::Value as Json;
use self::mongodb::db::{Database, ThreadedDatabase};
use self::mongodb::coll::options::FindOneAndUpdateOptions;
use self::bson::{Document, Bson};
use std::fmt;
use std::thread;
use config;
use types::ThreadedAPI;
use objects::reference::Ref;
use worker_pool::WorkerPool;
use error::IngestResult;
use error::IngestError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Object {
    #[serde(rename = "_id")]
    pub id: Json,
    pub attributes: Json,
    pub links: Json,
    pub object_type: Json,
}

impl Object {
    pub fn new(id: Json, attributes: Json, links: Json, object_type: Json) -> Object {
        Object {
            id: id,
            attributes: attributes,
            links: links,
            object_type: object_type,
        }
    }

    pub fn from_json(mut json: Json) -> IngestResult<Object> {
        let mut json_map = json.as_object_mut();

        match json_map {
            Some(map) => {

                let id = map.remove("id");
                let attributes = map.remove("attributes");
                let links = map.remove("links").unwrap_or(serde_json::from_str("{}").unwrap());
                let obj_type = map.remove("type");

                match and_list(vec![id, attributes, obj_type]) {
                    Some(mut value_list) => {
                        Ok(Object::new(value_list.remove(0),
                                       value_list.remove(0),
                                       links,
                                       value_list.remove(0)))
                    }
                    None => Err(IngestError::InvalidObjDataError),
                }
            }
            None => Err(IngestError::InvalidObjDataError),
        }
    }

    // TODO: Handle the rest of the references

    pub fn assets(&self) -> Option<Vec<Ref>> {
        self.references("assets")
    }

    pub fn collection(&self) -> Option<Ref> {
        self.reference("collection")
    }

    pub fn collections(&self) -> Option<Vec<Ref>> {
        self.references("collections")
    }

    pub fn episode(&self) -> Option<Ref> {
        self.reference("episode")
    }

    pub fn episodes(&self) -> Option<Vec<Ref>> {
        self.references("episodes")
    }

    pub fn extras(&self) -> Option<Vec<Ref>> {
        self.references("extras")
    }

    pub fn franchise(&self) -> Option<Ref> {
        self.reference("franchise")
    }

    pub fn season(&self) -> Option<Ref> {
        self.reference("season")
    }

    pub fn seasons(&self) -> Option<Vec<Ref>> {
        self.references("seasons")
    }

    pub fn show(&self) -> Option<Ref> {
        self.reference("show")
    }

    pub fn shows(&self) -> Option<Vec<Ref>> {
        self.references("shows")
    }

    pub fn special(&self) -> Option<Ref> {
        self.reference("special")
    }

    pub fn specials(&self) -> Option<Vec<Ref>> {
        self.references("specials")
    }

    pub fn attribute(&self, property: &str) -> Option<&str> {
        self.attributes.find(property).map_or(None, |type_value| type_value.as_str())
    }

    pub fn parent(&self) -> Option<Ref> {
        vec![// self.collection(),
             self.episode(),
             self.franchise(),
             self.season(),
             self.show(),
             self.special()]
            .into_iter()
            .fold(None, |parent, current| match parent {
                None => current,
                _ => parent,
            })
    }

    pub fn import(&self,
                  api: &ThreadedAPI,
                  db: &Database,
                  import_refs: bool,
                  run_start_time: i64) {
        // TODO: Do database stuff
        println!("Import {} with id => {}", self.object_type, self.id);

        // Make sure that we can find a collection for the type
        match self.object_type.as_str() {
            Some(type_string) => {

                // Check the updated_at date to determine if the db needs to
                // update this object
                let updated_at_string =
                    self.attributes.find("updated_at").unwrap().as_str().unwrap();

                // let updated_at_time = time::strptime(updated_at_string, "%Y-%m-%dT%H:%M:%SZ");
                let updated_at_time = match updated_at_string.parse::<DateTime<UTC>>() {
                    Ok(date) => date.timestamp(),
                    Err(_) => 0,
                };

                if updated_at_time > run_start_time {
                    // println!("do update");
                    match self.as_document() {
                        Ok(doc) => {
                            let coll = db.collection(type_string);
                            let id = self.id.as_str().unwrap();

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

                if import_refs {
                    self.import_refs(api, db, import_refs, run_start_time);
                }
            }
            None => (),
        }
    }

    fn import_refs(&self,
                   api: &ThreadedAPI,
                   db: &Database,
                   import_refs: bool,
                   run_start_time: i64) {

        let mut pool =
            WorkerPool::new(config::pool_size_for(self.object_type.as_str().unwrap_or("")));
        let refs: Vec<Option<Vec<Ref>>> = vec![self.assets(),
                                               // self.collections(),
                                               self.episodes(),
                                               self.extras(),
                                               self.seasons(),
                                               self.shows(),
                                               self.specials()];

        for optional_refs in refs {

            match optional_refs {
                Some(ref_list) => {
                    for reference in ref_list {
                        let shared_api = api.clone();
                        let shared_db = db.clone();
                        pool.add_worker(thread::spawn(move || {
                            reference.import(&shared_api, &shared_db, import_refs, run_start_time);
                        }));
                    }
                }
                None => (),
            }
        }

        pool.wait_for_workers();
    }

    fn reference(&self, ref_name: &str) -> Option<Ref> {

        self.attributes.find(ref_name).map_or(None, |ref_data| Ref::from_json(ref_data).ok())
    }

    fn references(&self, ref_name: &str) -> Option<Vec<Ref>> {

        self.attributes
            .find(ref_name)
            .map_or(None, |objects| {
                objects.as_array()
                    .map_or(None, |array| {
                        Some(array.into_iter()
                            .filter_map(|o| Ref::from_json(o).ok())
                            .collect::<Vec<Ref>>())
                    })
            })

    }

    // fn as_bson(&self) -> Bson {
    //
    //     Bson::from_json(&self.attributes)
    // }

    fn as_document(&self) -> IngestResult<Document> {

        let bson_result = bson::to_bson(&self);

        match bson_result {
            Ok(serialized) => {
                if let bson::Bson::Document(document) = serialized {
                    Ok(document)
                } else {
                    Err(IngestError::General)
                }
            }
            Err(err) => Err(IngestError::Serialize(err)),
        }


        // let mut doc = Document::new();

        // doc.insert("_id", Bson::from_json(&self.id));
        // doc.insert("attributes", Bson::from_json(&self.attributes));
        // doc.insert("links", Bson::from_json(&self.links));
        // doc.insert("type", Bson::from_json(&self.object_type));
        // doc.insert("parent", Bson::from_json(&self.parent().unwrap().id));

        // doc
    }
}

impl fmt::Display for Object {
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
