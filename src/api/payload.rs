extern crate serde;
extern crate serde_json;

use serde_json::Map;
use serde_json::Value as Json;
use serde_json::value::ToJson;

use api::Emitter;
use config::Config;
use objects::{Object, Ref};
use types::StorageEngine;

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

    pub fn from_object<T: StorageEngine>(object: &Object, store: &T) -> Option<Payload> {
        if let Json::Object(mut data) = object.attributes.clone() {
            data.insert("id".to_string(), Json::String(object.id.clone()));
            data.insert("type".to_string(), Json::String(object.object_type.clone()));

            let parent = match object.parent(store) {
                Some(p) => Payload::from_object(&p, store).map(|payload| payload.data).to_json(),
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

    pub fn emitter<'a, 'b>(&'a self, config: &'b Config) -> Emitter<'a, 'b> {
        Emitter::new(self, config)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Map;
    use serde_json::Value as Json;

    use api::payload::Payload;
    use objects::{Object, Ref};
    use storage::{SinkStore, Storage};

    #[test]
    fn payload_from_ref() {
        let id = "payload-test-id";
        let ref_type = "asset";

        let mut map = Map::new();
        map.insert("id".to_string(), Json::String(id.to_string()));
        map.insert("type".to_string(), Json::String(ref_type.to_string()));

        assert_eq!(Payload::new(map),
                   Payload::from_ref(&Ref::new(id.to_string(),
                                               Json::Object(Map::new()),
                                               ref_type.to_string(),
                                               "http://0.0.0.0".to_string())))
    }

    #[test]
    fn invalid_object_attributes() {
        let obj_id = "obj-test-id".to_string();
        let obj_type = "asset".to_string();
        let obj_links = Json::Object(Map::new());
        let store = SinkStore::new(None).unwrap();

        let json_types = vec![Json::Null,
                              Json::Bool(true),
                              Json::String("attr".to_string()),
                              Json::Array(vec![])]
            .into_iter();

        for json_type in json_types {
            let obj = Object::new(obj_id.clone(),
                                  json_type,
                                  obj_type.clone(),
                                  obj_links.clone());
            assert_eq!(Payload::from_object(&obj, &store), None);
        }
    }

    #[test]
    fn valid_object() {
        let obj = Object::new("obj-test-id".to_string(),
                              Json::Object(Map::new()),
                              "asset".to_string(),
                              Json::Object(Map::new()));
        let store = SinkStore::new(None).unwrap();

        let mut data = Map::new();
        data.insert("id".to_string(), Json::String("obj-test-id".to_string()));
        data.insert("type".to_string(), Json::String("asset".to_string()));
        data.insert("parent".to_string(), Json::Null);

        assert_eq!(Payload::from_object(&obj, &store).unwrap(),
                   Payload::new(data));
    }

    #[test]
    fn valid_object_with_parent() {}
}
