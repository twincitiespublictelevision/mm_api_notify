extern crate mongodb;
extern crate rayon;
extern crate serde;
extern crate serde_json;

use self::mongodb::db::Database;
use self::rayon::prelude::*;
use self::serde_json::Value as Json;

use std::fmt;
use std::cmp::PartialEq;
use types::ThreadedAPI;
use error::IngestResult;
use error::IngestError;
use objects::import::Importable;
use objects::reference::Ref;
use objects::utils;

#[derive(Debug, PartialEq)]
pub struct Collection {
    page: Vec<Ref>,
    links: Json,
}

impl Collection {
    pub fn new(page: Vec<Ref>, links: Json) -> Collection {
        Collection {
            page: page,
            links: links,
        }
    }

    pub fn page(&self) -> &Vec<Ref> {
        &self.page
    }

    pub fn prev_page(&self, api: &ThreadedAPI) -> Option<Collection> {
        match self.links.find("next") {
            Some(&Json::String(ref next_url)) => self.get_page(api, next_url.as_str()).ok(),
            _ => None,
        }
    }

    pub fn next_page(&self, api: &ThreadedAPI) -> Option<Collection> {
        match self.links.find("prev") {
            Some(&Json::String(ref prev_url)) => self.get_page(api, prev_url.as_str()).ok(),
            _ => None,
        }
    }

    fn get_page(&self, api: &ThreadedAPI, url: &str) -> IngestResult<Collection> {
        utils::parse_response(api.url(url)).and_then(|json| Collection::from_json(&json))
    }

    pub fn import_left(&self,
                       api: &ThreadedAPI,
                       db: &Database,
                       follow_refs: bool,
                       path_from_root: &Vec<&str>,
                       since: i64) {
        self.prev_page(api)
            .and_then(|collection| {
                Some(collection.import_left(api, db, follow_refs, path_from_root, since))
            });
        self.import_page(api, db, follow_refs, path_from_root, since);
    }

    pub fn import_right(&self,
                        api: &ThreadedAPI,
                        db: &Database,
                        follow_refs: bool,
                        path_from_root: &Vec<&str>,
                        since: i64) {
        self.next_page(api)
            .and_then(|collection| {
                Some(collection.import_right(api, db, follow_refs, path_from_root, since))
            });
        self.import_page(api, db, follow_refs, path_from_root, since);
    }

    fn import_page(&self,
                   api: &ThreadedAPI,
                   db: &Database,
                   follow_refs: bool,
                   path_from_root: &Vec<&str>,
                   since: i64) {
        &self.page
            .par_iter()
            .for_each(|object| object.import(api, db, follow_refs, path_from_root, since));
    }
}

impl Importable for Collection {
    type Value = Collection;

    fn import(&self,
              api: &ThreadedAPI,
              db: &Database,
              follow_refs: bool,
              path_from_root: &Vec<&str>,
              since: i64) {

        self.import_left(api, db, follow_refs, path_from_root, since);
        self.import_right(api, db, follow_refs, path_from_root, since);
        self.import_page(api, db, follow_refs, path_from_root, since);
    }

    fn from_json(json: &Json) -> IngestResult<Collection> {
        let json_chunks = json.as_object()
            .and_then(|map| Some((map.get("data"), map.get("links"))));

        match json_chunks {
                Some((Some(data), Some(links))) => {
                    data.as_array()
                        .and_then(|data_list| {
                            Some(data_list.iter()
                                .filter_map(|item| Ref::from_json(item).ok())
                                .collect::<Vec<Ref>>())
                        })
                        .and_then(|items| Some((Collection::new(items, links.clone()))))
                }
                _ => None,
            }
            .ok_or(IngestError::InvalidDocumentDataError)
    }
}

impl fmt::Display for Collection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} items, {}", self.page.len(), self.links)
    }
}

#[cfg(test)]
mod tests {
    use serde_json;
    use objects::collection::Collection;
    use objects::import::Importable;
    use objects::reference::Ref;
    #[test]
    fn test_json_parse() {
        let json_str = "{\"data\":[{\"id\":1,\"attributes\":{},\"type\":\"asset\"},{\"id\":2,\
                        \"attributes\":{},\"type\":\"asset\"}],\"links\":{}}";

        let json: serde_json::error::Result<serde_json::Value> = serde_json::from_str(json_str);
        let items: Vec<serde_json::Value> = json.unwrap()
            .find("data")
            .unwrap()
            .as_array()
            .unwrap()
            .to_vec();
        let refs: Vec<Ref> =
            items.iter().filter_map(|item| Ref::from_json(item).ok()).collect::<Vec<Ref>>();
        let links = serde_json::from_str("{}").unwrap();
        let coll1 = Collection::new(refs, links);

        let json = serde_json::from_str(json_str).unwrap();
        let coll2 = Collection::from_json(&json).unwrap();

        assert_eq!(coll1, coll2)
    }
}
