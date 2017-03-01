extern crate rayon;
extern crate serde_json;

use self::rayon::prelude::*;
use self::serde_json::Value as Json;

use std::fmt;

use error::IngestResult;
use error::IngestError;
use objects::import::Importable;
use objects::reference::Ref;
use objects::utils;
use runtime::Runtime;
use types::{ImportResult, StorageEngine, ThreadedAPI};

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

    fn get_collection<S: ThreadedAPI>(&self, api: &S, url: &str) -> IngestResult<Collection> {
        utils::parse_response(api.url(url))
            .and_then(|json| Collection::from_json(&json))
            .or_else(|err| {
                error!("Failed to query {} due to {}", url, err);
                Err(err)
            })
    }

    fn import_page<T: StorageEngine, S: ThreadedAPI>(&self,
                                                     runtime: &Runtime<T, S>,
                                                     follow_refs: bool,
                                                     since: i64)
                                                     -> ImportResult {
        self.page
            .par_iter()
            .map(|item| item.import(runtime, follow_refs, since))
            .reduce(|| (0, 0), |(p1, f1), (p2, f2)| (p1 + p2, f1 + f2))
    }
}

impl Importable for Collection {
    fn import<T: StorageEngine, S: ThreadedAPI>(&self,
                                                runtime: &Runtime<T, S>,
                                                follow_refs: bool,
                                                since: i64)
                                                -> ImportResult {

        let num_pages = (self.total as f64 / self.page_size as f64).ceil() as usize + 1;

        self.links
            .get("first")
            .and_then(|first_url| {
                first_url.as_str().and_then(|base_url| {
                    Some((1..num_pages)
                        .collect::<Vec<usize>>()
                        .par_iter()
                        .map(|page_num| {
                            let mut page_url = String::new();
                            page_url.push_str(base_url);
                            page_url.push(if base_url.contains('?') { '&' } else { '?' });
                            page_url.push_str("page=");
                            page_url.push_str(page_num.to_string().as_str());

                            self.get_collection(&runtime.api, page_url.as_str())
                                .and_then(|collection| {
                                    Ok(collection.import_page(runtime, follow_refs, since))
                                })
                                .unwrap_or((0, 1))
                        })
                        .reduce(|| (0, 0), |(p1, f1), (p2, f2)| (p1 + p2, f1 + f2)))
                })
            })
            .or_else(|| Some(self.import_page(runtime, follow_refs, since)))
            .unwrap_or((0, 1))
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
                        .and_then(|items| match pagination_data {
                            Some((Some(per_page), Some(total))) => Some((items, per_page, total)),
                            _ => None,
                        })
                        .and_then(|(items, per_page, total)| {
                            Some(Collection::new(items,
                                                 links.clone(),
                                                 per_page as usize,
                                                 total as usize))
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

    use std::collections::HashSet;
    use std::iter::FromIterator;

    use config::{APIConfig, Config, DBConfig, LogConfig};
    use objects::{Collection, Importable, Ref};
    use runtime::Runtime;
    use storage::{SinkStore, Storage};
    use client::{APIClient, TestClient};

    #[test]
    fn test_json_parse() {
        let json_str = "{\"data\":[{\"id\":1,\"attributes\":{},\"type\":\"asset\"},{\"id\":2,\
                        \"attributes\":{},\"type\":\"asset\"}],\"links\":{},\"meta\":\
                        {\"pagination\":{\"per_page\":2,\"count\":26}}}";

        let json: serde_json::error::Result<serde_json::Value> = serde_json::from_str(json_str);
        let items: Vec<serde_json::Value> = json.unwrap()
            .get("data")
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

    #[test]
    fn import_fetches_all_pages() {
        let coll_json = json!({
            "data": [
                {
                    "id": 1,
                    "attributes": {},
                    "type": "asset"
                }
            ],
            "links": {
                "first": "http://0.0.0.0/test"
            },
            "meta": {
                "pagination": {
                    "per_page": 5,
                    "count": 25
                }
            }
        });

        let coll = Collection::from_json(&coll_json).unwrap();

        let store = SinkStore::new(None).unwrap();
        let mut client = TestClient::new(None).unwrap();

        client.set_response("{}".to_string());

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
            log: LogConfig { location: None },
            enable_hooks: false,
            hooks: None,
        };

        let reporter = client.clone();

        let runtime = Runtime {
            api: client,
            config: config,
            store: store,
            verbose: false,
        };

        coll.import(&runtime, false, 0);

        let endpoints = vec![
            "http://0.0.0.0/test?page=1".to_string(),
            "http://0.0.0.0/test?page=2".to_string(),
            "http://0.0.0.0/test?page=3".to_string(),
            "http://0.0.0.0/test?page=4".to_string(),
            "http://0.0.0.0/test?page=5".to_string()
        ];

        let endpoints_set: HashSet<String> = HashSet::from_iter(endpoints);

        let reqs = reporter.get_reqs();
        let test_set: HashSet<String> = HashSet::from_iter(reqs);

        assert_eq!(endpoints_set, test_set);
    }
}
