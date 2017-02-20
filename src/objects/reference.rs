extern crate chrono;
extern crate serde_json;

use self::serde_json::Value as Json;

use std::fmt;

use hooks::Payload;
use error::IngestResult;
use error::IngestError;
use objects::import::Importable;
use objects::object::Object;
use objects::utils;
use runtime::Runtime;
use types::{ImportResult, StorageEngine, ThreadedAPI};

#[derive(Debug, PartialEq)]
pub struct Ref {
    pub id: String,
    pub attributes: Json,
    pub ref_type: String,
    pub self_url: String,
}

enum ImportAction {
    Delete,
    Update,
}

impl<'a> From<&'a str> for ImportAction {
    fn from(str: &'a str) -> ImportAction {
        match str {
            "delete" => ImportAction::Delete,
            _ => ImportAction::Update,
        }
    }
}

impl Ref {
    pub fn new(id: String, attributes: Json, ref_type: String, self_url: String) -> Ref {
        Ref {
            id: id,
            attributes: attributes,
            ref_type: ref_type,
            self_url: self_url,
        }
    }

    fn import_general<T: StorageEngine, S: ThreadedAPI>(&self,
                                                        runtime: &Runtime<T, S>,
                                                        follow_refs: bool,
                                                        since: i64)
                                                        -> ImportResult {

        let res = utils::parse_response(runtime.api.url(self.self_url.as_str()))
            .and_then(|json| Object::from_json(&json))
            .and_then(|obj| Ok(obj.import(runtime, follow_refs, since)));

        if runtime.verbose && res.is_err() {
            println!("Skipping {} {} due to {:?}", self.id, self.ref_type, res);
        }

        res.unwrap_or((0, 1))
    }

    fn import_changelog<T: StorageEngine, S: ThreadedAPI>(&self,
                                                          runtime: &Runtime<T, S>,
                                                          since: i64,
                                                          action: ImportAction)
                                                          -> ImportResult {
        match action {
            ImportAction::Delete => {
                if runtime.config.enable_hooks && runtime.config.hooks.is_some() {
                    match runtime.config.hooks {
                        Some(ref hooks) => Payload::from_ref(self).emitter(&hooks).delete(),
                        _ => 0,
                    };
                }

                (0, 0)
            }
            ImportAction::Update => self.import_general(runtime, false, since),
        }
    }
}

impl Importable for Ref {
    fn import<T: StorageEngine, S: ThreadedAPI>(&self,
                                                runtime: &Runtime<T, S>,
                                                follow_refs: bool,
                                                since: i64)
                                                -> ImportResult {

        // When importing a reference we branch based on an inspection of the attributes. If this
        // a changelog reference then we prefer to use a custom import.
        let action = self.attributes.get("action").and_then(|action| action.as_str());

        match action {
            Some(action_str) => {
                self.import_changelog(runtime, since, ImportAction::from(action_str))
            }
            None => self.import_general(runtime, follow_refs, since),
        }
    }

    fn from_json(json: &Json) -> IngestResult<Ref> {

        json.clone()
            .as_object_mut()
            .and_then(|map| {
                let id = map.remove("id")
                    .and_then(|id_val| id_val.as_str().and_then(|id_str| Some(id_str.to_string())));

                let attributes = map.remove("attributes");

                let attrs = attributes.clone();

                let ref_type = map.remove("type").and_then(|ref_type_val| {
                    ref_type_val.as_str().and_then(|ref_type_str| Some(ref_type_str.to_string()))
                });

                let self_url = map.remove("links")
                    .and_then(|mut links| {
                        links.as_object_mut()
                            .and_then(|link_map| link_map.remove("self"))
                            .and_then(|self_val| {
                                self_val.as_str().and_then(|self_str| Some(self_str.to_string()))
                            })
                    });

                match (id, attrs, ref_type, self_url) {
                    (Some(p1), Some(p2), Some(p3), Some(p4)) => Some(Ref::new(p1, p2, p3, p4)),
                    _ => None,
                }
            })
            .ok_or(IngestError::InvalidRefDataError)
    }
}

impl fmt::Display for Ref {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.attributes.fmt(f)
    }
}

#[cfg(test)]
mod tests {

    use serde_json::Map;
    use serde_json::Value as Json;

    use config::{APIConfig, Config, DBConfig};
    use client::{APIClient, TestClient};
    use error::IngestError;
    use objects::{Importable, Ref};
    use storage::{SinkStore, Storage};
    use runtime::Runtime;

    #[test]
    fn translates_from_valid_fields() {
        let ref_json = json!({
            "id": "test-id",
            "attributes": {},
            "type": "show",
            "links": {
                "self": "http://0.0.0.0/test"
            }
        });

        let test_ref = Ref::new("test-id".to_string(),
                                Json::Object(Map::new()),
                                "show".to_string(),
                                "http://0.0.0.0/test".to_string());

        assert_eq!(test_ref, Ref::from_json(&ref_json).unwrap())
    }

    #[test]
    fn missing_required_fields_fails() {
        let missing_id = json!({
            "attributes": {},
            "type": "show",
            "links": {
                "self": "http://0.0.0.0/test"
            }
        });

        match Ref::from_json(&missing_id) {
            Err(IngestError::InvalidRefDataError) => (),
            _ => panic!("Ref should not be creatable without id"),
        };

        let missing_attributes = json!({
            "id": "test-id",
            "type": "show",
            "links": {
                "self": "http://0.0.0.0/test"
            }
        });

        match Ref::from_json(&missing_attributes) {
            Err(IngestError::InvalidRefDataError) => (),
            _ => panic!("Ref should not be creatable without attributes"),
        };

        let missing_type = json!({
            "id": "test-id",
            "attributes": {},
            "links": {
                "self": "http://0.0.0.0/test"
            }
        });

        match Ref::from_json(&missing_type) {
            Err(IngestError::InvalidRefDataError) => (),
            _ => panic!("Ref should not be creatable without type"),
        };

        let missing_links = json!({
            "id": "test-id",
            "attributes": {},
            "type": "show",
        });

        match Ref::from_json(&missing_links) {
            Err(IngestError::InvalidRefDataError) => (),
            _ => panic!("Ref should not be creatable without links"),
        };

        let missing_self_url = json!({
            "id": "test-id",
            "attributes": {},
            "type": "show",
            "links": {}
        });

        match Ref::from_json(&missing_self_url) {
            Err(IngestError::InvalidRefDataError) => (),
            _ => panic!("Ref should not be creatable without self url"),
        };
    }

    #[test]
    fn imports_ref_obj_from_api() {
        let test_resp = "{\"id\": \"test-id\", \"attributes\": {\"updated_at\": \
                         \"2017-01-01T00:00:00Z\"}, \"type\": \"show\", \"links\": \
                         {\"self\": \"http://0.0.0.0/test\"}}";

        let store = SinkStore::new(None).unwrap();
        let mut client = TestClient::new(None).unwrap();

        client.set_response(test_resp.to_string());

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
            enable_hooks: false,
            hooks: None,
        };

        let runtime = Runtime {
            api: client,
            config: config,
            store: store,
            verbose: false,
        };

        let test_ref = Ref::new("test-id".to_string(),
                                Json::Object(Map::new()),
                                "show".to_string(),
                                "http://0.0.0.0/test".to_string());

        test_ref.import(&runtime, false, 0);

        let req = runtime.api.reqs.lock().unwrap().pop().unwrap();

        assert_eq!(req, "http://0.0.0.0/test".to_string())
    }

    #[test]
    fn emits_delete() {
        unimplemented!()
    }
}
