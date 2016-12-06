extern crate mongodb;
extern crate time;
extern crate core_data_client;
extern crate rustc_serialize;

use self::rustc_serialize::json::Json;
use mongodb::db::Database;
use std::thread;
use std::usize;
use config;
use error::IngestError;
use error::IngestResult;
use objects::Object;
use worker_pool::WorkerPool;
use types::ThreadedAPI;
use time::Duration;

pub fn run(api: &ThreadedAPI, db: &Database) -> IngestResult<Duration> {

    let start_time = time::now();

    // Get the initial page show list page to determine how many pages to run
    let number_of_pages = get_number_of_pages(api);

    match number_of_pages {
        Ok(count) => {
            let mut page_pool = WorkerPool::new(config::pool_size_for("show_list"));

            for page_number in 1..count {
                let shared_db = db.clone();
                let shared_api = api.clone();
                page_pool.add_worker(thread::spawn(move || {
                    run_show_page(page_number, &shared_api, &shared_db);
                }));
            }

            page_pool.wait_for_workers();

            Ok(time::now() - start_time)
        }
        Err(err) => Err(err),
    }
}

fn run_show_page(page: usize, api: &ThreadedAPI, db: &Database) {
    let show_list = get_page(page, api);

    match show_list {
        Ok(show_objects) => {

            let mut show_pool = WorkerPool::new(config::pool_size_for("show_list"));

            for show in show_objects {

                let shared_db = db.clone();
                let shared_api = api.clone();

                show_pool.add_worker(thread::spawn(move || {
                    show.import(&shared_api, &shared_db, true);
                }));
            }

            show_pool.wait_for_workers();
        }
        Err(_) => (),
    };
}

fn get_page(page: usize, api: &ThreadedAPI) -> IngestResult<Vec<Object>> {

    let doc_result = get_list_doc(page, api);

    match doc_result {
        Ok(document) => {
            match document.find("data") {
                Some(show_list) => {
                    match show_list.as_array() {
                        Some(show_array) => {
                            Ok(show_array.into_iter().map(Object::from_json).collect())
                        }
                        None => Err(IngestError::InvalidDocumentDataError),
                    }
                }
                None => Err(IngestError::InvalidDocumentDataError),
            }
        }
        Err(err) => Err(err),
    }
}

fn get_number_of_pages(api: &ThreadedAPI) -> IngestResult<usize> {

    let doc_result = get_list_doc(1, api);

    match doc_result {
        Ok(document) => {
            match document.find("meta") {
                Some(meta) => {
                    match meta.find("pagination") {
                        Some(pagination) => {
                            let count = pagination.find("count");
                            let per_page = pagination.find("per_page");


                            let pages = count.map_or(None, |count| {
                                count.as_u64().map_or(None, |u_count| {
                                    per_page.map_or(None, |per_page| {
                                        per_page.as_u64()
                                            .map_or(None, |u_per_page| Some(u_count / u_per_page))
                                    })
                                })
                            });

                            match pages {
                                Some(number_of_pages) => {

                                    // If number of pages is larger than usize, something is wrong
                                    if number_of_pages > usize::MAX as u64 {
                                        Err(IngestError::InvalidDocumentDataError)
                                    } else {
                                        Ok(number_of_pages as usize)
                                    }
                                }
                                None => Err(IngestError::InvalidDocumentDataError),
                            }

                        }
                        None => Err(IngestError::InvalidDocumentDataError),
                    }
                }
                None => Err(IngestError::InvalidDocumentDataError),
            }
        }
        Err(err) => Err(err),
    }
}

fn get_list_doc(page: usize, api: &ThreadedAPI) -> IngestResult<Json> {
    let response = api.shows().list(page);

    match response {
        Ok(response_string) => {
            match Json::from_str(response_string.as_str()) {
                Ok(json) => Ok(json),
                Err(err) => Err(IngestError::Parse(err)),
            }
        }
        Err(err) => Err(IngestError::API(err)),
    }
}
