extern crate rustc_serialize;
extern crate mongodb;
extern crate bson;

use self::rustc_serialize::json::Json;
use self::mongodb::db::{Database, ThreadedDatabase};
use self::mongodb::coll::options::FindOneAndUpdateOptions;
use self::bson::{Document, Bson};
use std::fmt;
use std::thread;
use config;
use types::ThreadedAPI;
use objects::reference::Ref;
use worker_pool::WorkerPool;

pub struct Object {
    pub id: Json,
    pub attributes: Json,
    pub links: Json,
    pub object_type: Json,
}

impl Object {
    pub fn new(id: &Json, attributes: &Json, links: &Json, object_type: &Json) -> Object {
        Object {
            id: id.clone(),
            attributes: attributes.clone(),
            links: links.clone(),
            object_type: object_type.clone(),
        }
    }

    pub fn from_json(json: &Json) -> Object {
        let default = Json::from_str("{}").unwrap();

        let id = json.find("id").unwrap_or(&default);
        let attributes = json.find("attributes").unwrap_or(&default);
        let links = json.find("links").unwrap_or(&default);
        let ref_type = json.find("type").unwrap_or(&default);

        Object::new(id, attributes, links, ref_type)
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
        self.attributes.find(property).map_or(None, |type_value| type_value.as_string())
    }

    pub fn parent(&self) -> Option<Ref> {
        vec![self.collection(),
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

    pub fn import(&self, api: &ThreadedAPI, db: &Database, import_refs: bool) {
        // TODO: Do database stuff
        println!("Import {} with id => {}", self.object_type, self.id);
        let id_bson = Bson::from_json(&self.id);
        let filter = doc! {
            "_id" => id_bson
        };

        let coll = db.collection(self.object_type.as_string().unwrap());
        let doc = self.as_document();

        let mut options = FindOneAndUpdateOptions::new();
        options.upsert = true;

        let res = coll.find_one_and_replace(filter, doc, Some(options));

        if import_refs {
            self.import_refs(api, db, import_refs);
        }
    }

    fn import_refs(&self, api: &ThreadedAPI, db: &Database, import_refs: bool) {

        let mut pool =
            WorkerPool::new(config::pool_size_for(self.object_type.as_string().unwrap_or("")));
        let refs: Vec<Option<Vec<Ref>>> = vec![self.assets(),
                                               self.collections(),
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
                            reference.import(&shared_api, &shared_db, import_refs);
                        }));
                    }
                }
                None => (),
            }
        }

        pool.wait_for_workers();
    }

    fn reference(&self, ref_name: &str) -> Option<Ref> {

        self.attributes.find(ref_name).map_or(None, |ref_data| Some(Ref::from_json(ref_data)))
    }

    fn references(&self, ref_name: &str) -> Option<Vec<Ref>> {

        self.attributes
            .find(ref_name)
            .map_or(None, |objects| {
                objects.as_array()
                    .map_or(None, |array| {
                        Some(array.into_iter()
                            .map(|o| Ref::from_json(o))
                            .collect::<Vec<Ref>>())
                    })
            })

    }

    fn as_bson(&self) -> Bson {

        Bson::from_json(&self.attributes)
    }

    fn as_document(&self) -> Document {

        let mut doc = Document::new();

        doc.insert("_id", Bson::from_json(&self.id));
        doc.insert("attributes", Bson::from_json(&self.attributes));
        doc.insert("links", Bson::from_json(&self.links));
        doc.insert("type", Bson::from_json(&self.object_type));
        doc.insert("parent", Bson::from_json(&self.parent().unwrap().id));

        doc
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.attributes.fmt(f)
    }
}
