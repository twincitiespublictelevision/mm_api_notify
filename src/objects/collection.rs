extern crate mongodb;
extern crate serde;
extern crate serde_json;

use self::serde_json::Value as Json;
use self::mongodb::db::Database;
use std::fmt;
use std::cmp::PartialEq;
use std::slice::Iter;
use std::vec::IntoIter;
use types::ThreadedAPI;
use error::IngestResult;
use error::IngestError;
use objects::parser;
use objects::import::Importable;

#[derive(Debug, PartialEq)]
pub struct Collection<T>
    where T: Importable
{
    pub items: Vec<T>,
    pub links: Json,
}

impl<T> Collection<T>
    where T: Importable
{
    pub fn new(items: Vec<T>, links: Json) -> Collection<T> {
        Collection {
            items: items,
            links: links,
        }
    }

    pub fn iter(&self) -> Iter<T> {
        self.items.iter()
    }

    pub fn into_iter(self) -> IntoIter<T> {
        self.items.into_iter()
    }

    pub fn from_json<F>(mut json: Json, transform: F) -> IngestResult<Collection<T>>
        where F: Fn(Json) -> IngestResult<T>
    {
        let json_map = json.as_object_mut();

        match json_map {
            Some(map) => {
                let items = map.remove("data")
                    .map_or(vec![], |objects| {
                        objects.as_array()
                            .map_or(vec![], |array| {
                                array.to_vec()
                                    .into_iter()
                                    .filter_map(|o| transform(o).ok())
                                    .collect::<Vec<T>>()
                            })
                    });

                let links = match map.remove("links") {
                    Some(link_data) => link_data,
                    None => serde_json::from_str("{}").unwrap(),
                };

                Ok(Collection::new(items, links))
            }
            None => Err(IngestError::InvalidDocumentDataError),
        }
    }

    pub fn import(&self,
                  api: &ThreadedAPI,
                  db: &Database,
                  import_refs: bool,
                  run_start_time: i64) {

        let empty_root_path = vec![];
        self.iter().map(|item| item.import(api, db, import_refs, run_start_time, &empty_root_path));
    }

    pub fn prev_page<F>(&self, api: &ThreadedAPI, transform: F) -> Option<Collection<T>>
        where F: Fn(Json) -> IngestResult<T>
    {
        match self.links.find("next") {
            Some(&Json::String(ref next_url)) => {
                self.get_page(api, next_url.as_str(), transform).ok()
            }
            _ => None,
        }
    }

    pub fn next_page<F>(&self, api: &ThreadedAPI, transform: F) -> Option<Collection<T>>
        where F: Fn(Json) -> IngestResult<T>
    {
        match self.links.find("prev") {
            Some(&Json::String(ref prev_url)) => {
                self.get_page(api, prev_url.as_str(), transform).ok()
            }
            _ => None,
        }
    }

    fn get_page<F>(&self, api: &ThreadedAPI, url: &str, transform: F) -> IngestResult<Collection<T>>
        where F: Fn(Json) -> IngestResult<T>
    {
        parser::parse_response(api.url(url)).and_then(|json| Collection::from_json(json, transform))
    }
}

impl<T> fmt::Display for Collection<T>
    where T: Importable
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} items, {}", self.items.len(), self.links)
    }
}

#[cfg(test)]
mod tests {
    use serde_json;
    use objects::collection::Collection;
    use objects::reference::Ref;
    #[test]
    fn test_json_parse() {
        let json_str = "{\"data\":[{\"id\":1,\"attributes\":{},\"type\":\"asset\"},{\"id\":2,\
                        \"attributes\":{},\"type\":\"asset\"}]}";

        let json: serde_json::error::Result<serde_json::Value> = serde_json::from_str(json_str);
        let items: Vec<serde_json::Value> = json.unwrap()
            .find("data")
            .unwrap()
            .as_array()
            .unwrap()
            .to_vec();
        let refs: Vec<Ref> =
            items.into_iter().filter_map(|item| Ref::from_json(item).ok()).collect::<Vec<Ref>>();
        let links = serde_json::from_str("{}").unwrap();
        let coll1 = Collection::new(refs, links);

        let json = serde_json::from_str(json_str).unwrap();
        let coll2 = Collection::from_json(json, |x| Ref::from_json(x)).unwrap();

        assert_eq!(coll1, coll2)
    }
}
