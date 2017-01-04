extern crate mongodb;
extern crate time;
extern crate core_data_client;
extern crate serde;
extern crate serde_json;

use self::serde_json::error::Result as JsonResult;
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
use self::serde_json::Value as Json;

pub fn run(api: &ThreadedAPI, db: &Database, run_start_time: i64) -> IngestResult<Duration> {

    let start_time = time::now();

    // Get the initial page show list page to determine how many pages to run
    let number_of_pages = get_number_of_pages(api);

    match number_of_pages {
        Ok(count) => {
            let mut page_pool = WorkerPool::new(config::pool_size_for("show_page_list"));

            for page_number in 1..(count + 1) {
                let shared_db = db.clone();
                let shared_api = api.clone();
                page_pool.add_worker(thread::spawn(move || {
                    run_show_page(page_number, &shared_api, &shared_db, run_start_time);
                }));
            }

            page_pool.wait_for_workers();

            Ok(time::now() - start_time)
        }
        Err(err) => Err(err),
    }
}

pub fn run_show(show_identifier: &str,
                api: &ThreadedAPI,
                db: &Database,
                run_start_time: i64)
                -> IngestResult<Duration> {

    let start_time = time::now();

    let response = api.shows().get(show_identifier);

    let show = match response {
        Ok(response_string) => {
            let full_json: JsonResult<Json> = serde_json::from_str(response_string.as_str());
            match full_json {
                Ok(mut json) => {
                    let json_map = json.as_object_mut();
                    match json_map {
                        Some(map) => {
                            let data = map.remove("data");
                            match data {
                                Some(show_object) => Object::from_json(show_object),
                                None => Err(IngestError::InvalidDocumentDataError),
                            }
                        }
                        None => Err(IngestError::InvalidDocumentDataError),
                    }
                }
                Err(err) => Err(IngestError::Parse(err)),
            }
        }
        Err(err) => Err(IngestError::API(err)),
    };

    match show {
        Ok(show) => show.import(&api, &db, true, run_start_time, Vec::new()),
        Err(_) => (),
    };

    Ok(time::now() - start_time)
}

fn run_show_page(page: usize, api: &ThreadedAPI, db: &Database, run_start_time: i64) {
    let show_list = get_page(page, api);

    match show_list {
        Ok(show_objects) => {

            let mut show_pool = WorkerPool::new(config::pool_size_for("show_list"));

            for show in show_objects {

                let shared_db = db.clone();
                let shared_api = api.clone();

                show_pool.add_worker(thread::spawn(move || {
                    show.import(&shared_api, &shared_db, true, run_start_time, Vec::new());
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
        Ok(mut document) => {
            let doc_map = document.as_object_mut();

            match doc_map {
                Some(doc) => {
                    let data_map = doc.remove("data");

                    match data_map {
                        Some(data) => {
                            match data.as_array() {
                                Some(show_array) => {
                                    Ok(show_array.into_iter()
                                        .cloned()
                                        .filter_map(|o| Object::from_json(o).ok())
                                        .collect())
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
                                count.as_f64().map_or(None, |f_count| {
                                    per_page.map_or(None, |per_page| {
                                        per_page.as_f64()
                                            .map_or(None, |f_per_page| {
                                                Some((f_count / f_per_page).ceil())
                                            })
                                    })
                                })
                            });

                            match pages {
                                Some(number_of_pages) => {

                                    // If number of pages is larger than usize, something is wrong
                                    if number_of_pages > usize::MAX as f64 {
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
            match serde_json::from_str(response_string.as_str()) {
                Ok(json) => Ok(json),
                Err(err) => Err(IngestError::Parse(err)),
            }
        }
        Err(err) => Err(IngestError::API(err)),
    }
}
