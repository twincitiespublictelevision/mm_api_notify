extern crate serde;
extern crate serde_json;

use serde_json::Map;
use serde_json::Value as Json;

use hooks::Emitter;
use config::HookConfig;
use objects::{Object, Ref};
use types::StorageEngine;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
                Some(p) => {
                    Payload::from_object(&p, store)
                        .map_or(Json::Null, |payload| Json::Object(payload.data))
                }
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

    pub fn emitter<'a, 'b, T: Emitter<'a, 'b>, F>(&'a self, config: &'b HookConfig, con: F) -> T
        where F: FnOnce(&'a Payload, &'b HookConfig) -> T
    {
        con(self, config)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Map;
    use serde_json::Value as Json;

    use std::collections::BTreeMap;

    use hooks::{Emitter, HttpEmitter, Payload};
    use objects::{Importable, Object, Ref};
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
        let obj_link = "http://0.0.0.0/obj-test-id/".to_string();
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
                                  obj_link.clone());
            assert_eq!(Payload::from_object(&obj, &store), None);
        }
    }

    #[test]
    fn valid_object() {
        let obj = Object::new("obj-test-id".to_string(),
                              Json::Object(Map::new()),
                              "asset".to_string(),
                              "http://0.0.0.0/obj-test-id/".to_string());
        let store = SinkStore::new(None).unwrap();

        let mut data = Map::new();
        data.insert("id".to_string(), Json::String("obj-test-id".to_string()));
        data.insert("type".to_string(), Json::String("asset".to_string()));
        data.insert("parent".to_string(), Json::Null);

        assert_eq!(Payload::from_object(&obj, &store).unwrap(),
                   Payload::new(data));
    }

    #[test]
    fn valid_object_with_parent() {
        let parent_json = json!({
            "data": {
                "id": "test-parent",
                "attributes": {
                    "updated_at": "2017-01-01T00:00:00Z"
                },
                "type": "franchise"
            },
            "links": {
                "self": "http://0.0.0.0/parent"
            }
        });

        let parent = Object::from_json(&parent_json).unwrap();

        let mut store = SinkStore::new(None).unwrap();
        store.set_response(parent);

        let obj_json = json!({
            "data": {
                "id": "test-child",
                "attributes": {
                    "franchise": {
                        "id": "test-parent",
                        "attributes": {
                            "updated_at": "2017-01-01T00:00:00Z"
                        },
                        "type": "franchise",
                        "links": {
                            "self": "http://0.0.0.0/parent"
                        }
                    },
                    "updated_at": "2017-01-01T00:00:00Z"
                },
                "type": "show"
            },
            "links": {
                "self": "http://0.0.0.0/child"
            }
        });

        let obj = Object::from_json(&obj_json).unwrap();

        if let Json::Object(payload_map) =
            json!({
            "updated_at": "2017-01-01T00:00:00Z",
            "id": "test-child",
            "type": "show",
            "parent": {
                "updated_at": "2017-01-01T00:00:00Z",
                "id": "test-parent",
                "type": "franchise",
                "parent": null
            }
        }) {

            let payload = Payload { data: payload_map };

            let test_payload = Payload::from_object(&obj, &store).unwrap();

            assert_eq!(payload, test_payload);
        } else {
            panic!("Failed to create payload map");
        }
    }

    #[test]
    fn provides_emitter_of_self() {
        let config = BTreeMap::new();

        if let Json::Object(payload_map) =
            json!({
            "id": "test-child",
            "type": "show",
            "updated_at": "2017-01-01T00:00:00Z",
            "parent": {
                "id": "test-parent",
                "parent": null,
                "updated_at": "2017-01-01T00:00:00Z",
                "type": "franchise"
            }
        }) {
            let payload = Payload { data: payload_map };
            let emit = HttpEmitter::new(&payload, &config);

            let test_emit = payload.emitter(&config, HttpEmitter::new);

            assert_eq!(emit, test_emit);
        } else {
            panic!("Failed to create payload map");
        }
    }
}
