extern crate mongodb;
extern crate rayon;
extern crate serde;
extern crate serde_json;

use self::mongodb::db::Database;
use self::rayon::prelude::*;
use self::serde_json::Value as Json;

use std::cmp::PartialEq;
use std::fmt;

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
    page_size: usize,
    total: usize,
}

impl Collection {
    pub fn new(page: Vec<Ref>, links: Json, page_size: usize, total: usize) -> Collection {
        Collection {
            page: page,
            links: links,
            page_size: page_size,
            total: total,
        }
    }

    pub fn page(&self) -> &Vec<Ref> {
        &self.page
    }

    pub fn prev_page(&self, api: &ThreadedAPI) -> Option<Collection> {
        match self.links.find("next") {
            Some(&Json::String(ref next_url)) => self.get_collection(api, next_url.as_str()).ok(),
            _ => None,
        }
    }

    pub fn next_page(&self, api: &ThreadedAPI) -> Option<Collection> {
        match self.links.find("prev") {
            Some(&Json::String(ref prev_url)) => self.get_collection(api, prev_url.as_str()).ok(),
            _ => None,
        }
    }

    fn get_collection(&self, api: &ThreadedAPI, url: &str) -> IngestResult<Collection> {
        utils::parse_response(api.url(url)).and_then(|json| Collection::from_json(&json))
    }

    fn import_page(&self,
                   api: &ThreadedAPI,
                   db: &Database,
                   follow_refs: bool,
                   path_from_root: &Vec<&str>,
                   since: i64) {
        &self.page
            .par_iter()
            .for_each(|item| item.import(api, db, follow_refs, path_from_root, since));
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

        let num_pages = (self.total as f64 / self.page_size as f64).ceil() as usize + 1;

        self.links
            .lookup("first")
            .and_then(|first_url| {
                first_url.as_str().and_then(|base_url| {
                    Some((1..num_pages).collect::<Vec<usize>>().par_iter().for_each(|page_num| {
                        let mut page_url = String::new();
                        page_url.push_str(base_url);
                        page_url.push(if base_url.contains("?") { '&' } else { '?' });
                        page_url.push_str("page=");
                        page_url.push_str(page_num.to_string().as_str());

                        self.get_collection(api, page_url.as_str()).and_then(|collection| {
                            Ok(collection.import_page(api, db, follow_refs, path_from_root, since))
                        });
                    }))
                })
            })
            .or_else(|| Some(self.import_page(api, db, follow_refs, path_from_root, since)));
    }

    fn from_json(json: &Json) -> IngestResult<Collection> {

        let json_chunks = json.as_object()
            .and_then(|map| Some((map.get("data"), map.get("links"), map.get("meta"))));

        match json_chunks {
                Some((Some(data), Some(links), Some(meta))) => {
                    let pagination_data = meta.as_object().and_then(|meta_map| {
                        meta_map.get("pagination").and_then(|pagination| {
                            pagination.as_object().and_then(|pagination_map| {
                                Some((pagination_map.get("per_page")
                                    .and_then(|per_page| per_page.as_u64()),
                                      pagination_map.get("count").and_then(|total| total.as_u64())))
                            })
                        })
                    });

                    data.as_array()
                        .and_then(|data_list| {
                            Some(data_list.iter()
                                .filter_map(|item| Ref::from_json(item).ok())
                                .collect::<Vec<Ref>>())
                        })
                        .and_then(|items| {
                            match pagination_data {
                                Some((Some(per_page), Some(total))) => {
                                    Some((items, per_page, total))
                                }
                                _ => None,
                            }
                        })
                        .and_then(|(items, per_page, total)| {
                            Some((Collection::new(items,
                                                  links.clone(),
                                                  per_page as usize,
                                                  total as usize)))
                        })
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
                        \"attributes\":{},\"type\":\"asset\"}],\"links\":{},\"meta\":\
                        {\"pagination\":{\"per_page\":2,\"count\":26}}}";

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
        let coll1 = Collection::new(refs, links, 2, 26);

        let json = serde_json::from_str(json_str).unwrap();
        let coll2 = Collection::from_json(&json).unwrap();

        assert_eq!(coll1, coll2)
    }
}
