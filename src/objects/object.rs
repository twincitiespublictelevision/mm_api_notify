extern crate bson;
extern crate chrono;
extern crate rayon;
extern crate serde_json;

use self::bson::{Bson, Document};
use self::chrono::{DateTime, UTC};
use self::rayon::prelude::*;
use self::serde_json::Value as Json;

use std::fmt;

use hooks::{Emitter, HttpEmitter, Payload};
use error::IngestResult;
use error::IngestError;
use objects::Collection;
use objects::Importable;
use objects::Ref;
use objects::utils;
use runtime::Runtime;
use types::{ImportResult, StorageEngine, ThreadedAPI};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Object {
    #[serde(rename = "_id")]
    pub id: String,
    pub attributes: Json,
    #[serde(rename = "type")]
    pub object_type: String,
    pub self_url: String,
}

impl Object {
    pub fn new(id: String, attributes: Json, object_type: String, self_url: String) -> Object {
        Object {
            id: id,
            attributes: attributes,
            object_type: object_type,
            self_url: self_url,
        }
    }

    pub fn parent<T: StorageEngine>(&self, store: &T) -> Option<Object> {

        match self.object_type.as_str() {
                "asset" => Some("parent_tree"),
                "episode" => Some("season"),
                "season" => Some("show"),
                "show" => Some("franchise"),
                "special" => Some("show"),
                _ => None,
            }
            .and_then(|parent_key| {
                self.attributes.get(parent_key).and_then(|parent| Ref::from_json(parent).ok())
            })
            .and_then(|parent_ref| {
                store.get(parent_ref.id.as_str(), parent_ref.ref_type.as_str())
                    .and_then(|res| res.ok())
            })
    }

    pub fn from_bson(bson: Bson) -> IngestResult<Object> {
        bson::from_bson(utils::map_bson_dates_to_string(bson)).map_err(IngestError::Deserialize)
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

    fn import_children<T: StorageEngine, S: ThreadedAPI>(&self,
                                                         runtime: &Runtime<T, S>,
                                                         follow_refs: bool,
                                                         since: i64)
                                                         -> ImportResult {

        let child_types = match self.object_type.as_str() {
            "episode" => vec!["asset"],
            "franchise" => vec!["asset", "show"],
            "season" => vec!["asset", "episode"],
            "show" => vec!["asset", "season", "special"],
            "special" => vec!["asset"],
            _ => vec![],
        };

        child_types.par_iter()
            .map(|child_type| {
                self.child_collection(&runtime.api, child_type)
                    .and_then(|child_collection| {

                        // TODO: Handle special case importing of collections
                        // Collections should never follow refs as all child elements are available
                        // under some other object.
                        // Collections need special casing for handling their paginated nature
                        // and unique storage requirements

                        Some(child_collection.import(runtime, follow_refs, since))
                    })
                    .unwrap_or((0, 1))
            })
            .reduce(|| (0, 0), |(p1, f1), (p2, f2)| (p1 + p2, f1 + f2))
    }

    fn child_collection<T: ThreadedAPI>(&self, api: &T, child_type: &str) -> Option<Collection> {
        let url = format!("{}{}s/?page-size=50", self.self_url, child_type);

        utils::parse_response(api.url(url.as_str()))
            .and_then(|api_json| Collection::from_json(&api_json))
            .ok()
    }

    fn import_parents<T: StorageEngine, S: ThreadedAPI>(&self,
                                                        runtime: &Runtime<T, S>,
                                                        _: bool,
                                                        since: i64)
                                                        -> ImportResult {

        let parent_types = match self.object_type.as_str() {
            "show" => vec!["franchise"],
            _ => vec![],
        };

        parent_types.par_iter()
            .map(|parent_type| match self.attributes.get(parent_type) {
                Some(parent_obj) => {
                    Ref::from_json(parent_obj)
                        .and_then(|refr| Ok(refr.import(runtime, false, since)))
                        .unwrap_or((0, 1))
                }
                _ => (0, 0),
            })
            .reduce(|| (0, 0), |(p1, f1), (p2, f2)| (p1 + p2, f1 + f2))
    }
}

impl Importable for Object {
    fn import<T: StorageEngine, S: ThreadedAPI>(&self,
                                                runtime: &Runtime<T, S>,
                                                follow_refs: bool,
                                                since: i64)
                                                -> ImportResult {

        info!("{:<10} {} {:<10} {}",
              "Importing",
              self.id,
              self.object_type,
              self.attributes.get("title").unwrap().as_str().unwrap());

        let updated_at_time = self.attributes
            .get("updated_at")
            .and_then(|update_string| update_string.as_str())
            .and_then(|updated_str| updated_str.parse::<DateTime<UTC>>().ok())
            .and_then(|date| Some(date.timestamp()))
            .unwrap_or(0);

        // Check the updated_at date to determine if the db needs to
        // update this object
        let mut update_result = if updated_at_time >= since {

            let res = runtime.store.put(self);

            if res.is_ok() && runtime.config.enable_hooks && runtime.config.hooks.is_some() {
                Payload::from_object(self, &runtime.store)
                    .and_then(|payload| {
                        runtime.config
                            .hooks
                            .as_ref()
                            .map(|hooks| payload.emitter(&hooks, HttpEmitter::new).update())
                    })
                    .or_else(|| {
                        error!("Failed to create payload from {}", self);
                        None
                    });
            };

            match res {
                Ok(_) => (1, 0),
                Err(err) => {
                    error!("Failed to write {} to cache due to {}", self, err);
                    (0, 1)
                }
            }
        } else {
            (0, 0)
        };

        if follow_refs {
            let child_results = self.import_children(runtime, follow_refs, since);
            update_result = (update_result.0 + child_results.0, update_result.1 + child_results.1);

            let parent_results = self.import_parents(runtime, follow_refs, since);
            update_result = (update_result.0 + parent_results.0,
                             update_result.1 + parent_results.1);
        };

        update_result
    }

    fn from_json(json: &Json) -> IngestResult<Object> {

        let mut source = json.clone();

        let id = source.get("data").and_then(|data| {
            data.get("id")
                .and_then(|id| id.as_str().and_then(|id_str| Some(id_str.to_string())))
        });

        let obj_type = source.get("data").and_then(|data| {
            data.get("type")
                .and_then(|o_type| o_type.as_str().and_then(|type_str| Some(type_str.to_string())))
        });

        source.as_object_mut()
            .and_then(|map| {

                let attrs = map.remove("data").and_then(|mut data| {
                    data.as_object_mut().and_then(|data_map| data_map.remove("attributes"))
                });

                let self_url = map.remove("links").and_then(|mut data| {
                    data.as_object_mut()
                        .and_then(|data_map| {
                            data_map.remove("self").unwrap().as_str().map(|s| s.to_string())
                        })
                });

                match (id, attrs, obj_type, self_url) {
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

#[cfg(test)]
mod tests {

    use bson::{Bson, Document};
    use chrono::{DateTime, UTC};
    use serde_json;
    use serde_json::{Map, Value as Json};

    use std::collections::BTreeMap;

    use client::{APIClient, TestClient};
    use config::{APIConfig, Config, DBConfig, LogConfig};
    use error::IngestError;
    use objects::{Importable, Object};
    use runtime::Runtime;
    use storage::{SinkStore, Storage};

    fn void_runtime() -> Runtime<SinkStore, TestClient> {
        let store = SinkStore::new(None).unwrap();
        let client = TestClient::new(None).unwrap();

        let empty = "".to_string();

        let config = Config {
            db: DBConfig {
                host: empty.clone(),
                port: 0,
                name: empty.clone(),
                username: empty.clone(),
                password: empty.clone(),
            },
            mm: APIConfig {
                key: empty.clone(),
                secret: empty.clone(),
                env: None,
                changelog_max_timespan: 0,
            },
            thread_pool_size: 0,
            min_runtime_delta: 0,
            log: LogConfig {
                location: None,
                level: None,
            },
            enable_hooks: false,
            hooks: None,
        };

        Runtime {
            api: client,
            config: config,
            store: store,
        }
    }

    #[test]
    fn translates_from_valid_json() {
        let test_obj = "{\"data\": {\"id\": \"test-id\", \"attributes\": {\"updated_at\": \
                         \"2017-01-01T00:00:00Z\"}, \"type\": \"show\"}, \"links\": \
                         {\"self\": \"http://0.0.0.0/test\"}}";

        let mut attr = Map::new();
        attr.insert("updated_at".to_string(),
                    Json::String("2017-01-01T00:00:00Z".to_string()));

        let obj = Object::new("test-id".to_string(),
                              Json::Object(attr),
                              "show".to_string(),
                              "http://0.0.0.0/test".to_string());

        assert_eq!(obj,
                   Object::from_json(&serde_json::from_str(test_obj).unwrap()).unwrap());
    }

    #[test]
    fn missing_required_fields_fail() {
        let missing_id = json!({
            "data": {
                "attributes": {
                    "updated_at": "2017-01-01T00:00:00Z"
                },
                "type": "show",
            },
            "links": {
                "self": "http://0.0.0.0/test"
            }
        });

        match Object::from_json(&missing_id) {
            Err(IngestError::InvalidObjDataError) => (),
            _ => panic!("Object should not be creatable without id"),
        };

        let missing_attributes = json!({
            "data": {
                "id": "test-id",
                "type": "show",
            },
            "links": {
                "self": "http://0.0.0.0/test"
            }
        });

        match Object::from_json(&missing_attributes) {
            Err(IngestError::InvalidObjDataError) => (),
            _ => panic!("Object should not be creatable without attributes"),
        };

        let missing_type = json!({
            "data": {
                "id": "test-id",
                "attributes": {
                    "updated_at": "2017-01-01T00:00:00Z"
                },
            },
            "links": {
                "self": "http://0.0.0.0/test"
            }
        });

        match Object::from_json(&missing_type) {
            Err(IngestError::InvalidObjDataError) => (),
            _ => panic!("Object should not be creatable without type"),
        };

        let missing_links = json!({
            "data": {
                "id": "test-id",
                "attributes": {
                    "updated_at": "2017-01-01T00:00:00Z"
                },
            },
            "type": "show",
        });

        match Object::from_json(&missing_links) {
            Err(IngestError::InvalidObjDataError) => (),
            _ => panic!("Object should not be creatable without links"),
        };
    }

    #[test]
    fn as_document_with_datetimes() {
        let mut attr = Map::new();
        attr.insert("updated_at".to_string(),
                    Json::String("2017-01-01T00:00:00Z".to_string()));

        let obj = Object::new("test-id".to_string(),
                              Json::Object(attr),
                              "show".to_string(),
                              "http://0.0.0.0/test".to_string());

        let mut attr = Document::new();
        let updated = Bson::UtcDatetime("2017-01-01T00:00:00Z".parse::<DateTime<UTC>>().unwrap());
        attr.insert("updated_at".to_string(), updated);

        let mut doc = Document::new();
        doc.insert("_id".to_string(), Bson::String("test-id".to_string()));
        doc.insert("attributes".to_string(), attr);
        doc.insert("type".to_string(), Bson::String("show".to_string()));
        doc.insert("self_url".to_string(),
                   Bson::String("http://0.0.0.0/test".to_string()));

        assert_eq!(obj.as_document().unwrap(), doc);
    }

    #[test]
    fn gets_parent_from_store() {
        let parent_obj_resp = "{\"data\": {\"id\": \"test-id\", \"attributes\": {\"updated_at\": \
                         \"2017-01-01T00:00:00Z\"}, \"type\": \"show\"}, \"links\": \
                         {\"self\": \"http://0.0.0.0/test\"}}";

        let mut store = SinkStore::new(None).unwrap();

        let parent_obj = Object::from_json(&serde_json::from_str(parent_obj_resp).unwrap())
            .unwrap();

        store.set_response(parent_obj.clone());

        let attr = json!({
            "franchise": {
                "id": "test-ref-id",
                "type": "franchise",
                "attributes": {},
                "links": {
                    "self": "http://0.0.0.0/ref-test"
                }
            }
        });

        let test_obj = Object::new("test-id".to_string(),
                                   attr,
                                   "show".to_string(),
                                   "http://0.0.0.0/test".to_string());

        let test_parent = test_obj.parent(&store).unwrap();

        assert_eq!(parent_obj, test_parent);
    }

    #[test]
    fn emits_update_if_new() {
        let e = "http://0.0.0.0/".to_string();

        let mut hook = BTreeMap::new();
        hook.insert("url".to_string(), e);

        let mut config = BTreeMap::new();
        config.insert("show".to_string(),
                      vec![hook.clone(), hook.clone(), hook.clone()]);

        let obj_json = json!({
            "data": {
                "id": "test-id",
                "type": "show",
                "attributes": {
                    "updated_at": "2017-02-21T20:42:27.010750Z"
                }
            },
            "links": {
                "self": ""
            }
        });

        let mut runtime = void_runtime();
        runtime.config.enable_hooks = true;
        runtime.config.hooks = Some(config);
        runtime.store.set_response(Object::from_json(&obj_json).unwrap());

        let obj = Object::from_json(&obj_json).unwrap();
        let test_res = obj.import(&runtime, false, 0);

        assert_eq!(test_res, (1, 0))
    }

    #[test]
    fn skips_emit_if_old() {
        let e = "http://0.0.0.0/".to_string();

        let mut hook = BTreeMap::new();
        hook.insert("url".to_string(), e);

        let mut config = BTreeMap::new();
        config.insert("show".to_string(),
                      vec![hook.clone(), hook.clone(), hook.clone()]);

        let obj_json = json!({
            "data": {
                "id": "test-id",
                "type": "show",
                "attributes": {
                    "updated_at": "2017-02-21T20:42:27.010750Z"
                }
            },
            "links": {
                "self": ""
            }
        });

        let mut runtime = void_runtime();
        runtime.config.enable_hooks = true;
        runtime.config.hooks = Some(config);
        runtime.store.set_response(Object::from_json(&obj_json).unwrap());

        let obj = Object::from_json(&obj_json).unwrap();
        let test_res = obj.import(&runtime, false, UTC::now().timestamp());

        assert_eq!(test_res, (0, 0))
    }
}
