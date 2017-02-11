extern crate chrono;
extern crate mongodb;
extern crate serde;
extern crate serde_json;

use self::mongodb::db::{Database, ThreadedDatabase};
use self::serde_json::Value as Json;

use std::fmt;

use api::Payload;
use config::get_config;
use error::IngestResult;
use error::IngestError;
use objects::import::Importable;
use objects::object::Object;
use objects::utils;
use types::ImportResult;
use types::ThreadedAPI;

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

    fn import_general(&self,
                      api: &ThreadedAPI,
                      db: &Database,
                      follow_refs: bool,
                      since: i64)
                      -> ImportResult {

        let res = utils::parse_response(api.url(self.self_url.as_str()))
            .and_then(|json| Object::from_json(&json))
            .and_then(|obj| Ok(obj.import(api, db, follow_refs, since)));

        if res.is_err() {
            println!("Skipping {} {} due to {:?}", self.ref_type, self.id, res);
        }

        res.unwrap_or((0, 1))
    }

    fn import_changelog(&self,
                        api: &ThreadedAPI,
                        db: &Database,
                        since: i64,
                        action: ImportAction)
                        -> ImportResult {
        match action {
            ImportAction::Delete => {
                if get_config().map_or(false, |conf| conf.enable_hooks) {
                    Payload::from_ref(self).emitter().delete();
                }

                (0, 0)
            }
            ImportAction::Update => self.import_general(api, db, false, since),
        }
    }
}

impl Importable for Ref {
    type Value = Ref;

    fn import(&self,
              api: &ThreadedAPI,
              db: &Database,
              follow_refs: bool,
              since: i64)
              -> ImportResult {

        // When importing a reference we branch based on an inspection of the attributes. If this
        // a changelog reference then we prefer to use a custom import.
        let action = self.attributes.lookup("action").and_then(|action| action.as_str());

        match action {
            Some(action_str) => {
                self.import_changelog(api, db, since, ImportAction::from(action_str))
            }
            None => self.import_general(api, db, follow_refs, since),
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
