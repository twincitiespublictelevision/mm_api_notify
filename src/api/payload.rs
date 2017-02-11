extern crate mongodb;
extern crate serde;
extern crate serde_json;

use self::mongodb::db::{Database, ThreadedDatabase};
use self::serde_json::Map;
use self::serde_json::Value as Json;
use self::serde_json::value::ToJson;

use api::Emitter;
use objects::Object;
use objects::Ref;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Payload {
    pub data: Map<String, Json>,
}

impl Payload {
    pub fn new(data: Map<String, Json>) -> Payload {
        Payload { data: data }
    }

    pub fn from_ref(refr: &Ref) -> Payload {
        let mut data = Map::new();
        data.insert("id".to_string(), Json::String(refr.id.clone()));
        data.insert("type".to_string(), Json::String(refr.ref_type.clone()));
        Payload::new(data)
    }

    pub fn from_object(object: &Object, db: &Database) -> Option<Payload> {
        if let Json::Object(mut data) = object.attributes.clone() {
            data.insert("id".to_string(), Json::String(object.id.clone()));
            data.insert("type".to_string(), Json::String(object.object_type.clone()));

            let parent = match object.parent(db) {
                Some(p) => Payload::from_object(&p, db).map(|payload| payload.data).to_json(),
                None => Json::Null,
            };
            data.insert("parent".to_string(), parent);

            data.remove("episode");
            data.remove("season");
            data.remove("special");
            data.remove("show");
            data.remove("franchise");

            Some(Payload::new(data))
        } else {
            None
        }
    }

    pub fn emitter(&self) -> Emitter {
        Emitter::new(&self)
    }
}
