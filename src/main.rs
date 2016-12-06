// extern crate video_ingest;
extern crate mongodb;
extern crate time;

#[macro_use]
extern crate bson;
// use video_ingest::video;
// use video_ingest::config;
// use std::env;
//

extern crate core_data_client;
extern crate rustc_serialize;

mod config;
mod error;
mod objects;
mod show_runner;
mod types;
mod worker_pool;

use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use core_data_client::Client as APIClient;
use std::sync::Arc;
use types::ThreadedAPI;

// fn run(page: usize, api: &ThreadedAPI, db: &Database) {
//
//     let response = api.shows().list(page);
//
//     let shows: Vec<Object> = Json::from_str(response.unwrap().as_str())
//         .unwrap()
//         .find("data")
//         .unwrap()
//         .as_array()
//         .unwrap()
//         .into_iter()
//         .map(Object::from_json)
//         .collect();
//
//     // shows[0].import(api, db, false);
//     for show in shows {
//         show.import(api, db, true);
//     }
//
//     // let mut pool = WorkerPool::new(POOL_SIZE);
//     //
//     // for show in shows {
//     //     let shared_client = client.clone();
//     //
//     //     pool.add_worker(thread::spawn(move || {
//     //         let show_title =
//     //             show.data.find("attributes").unwrap().find("title").unwrap().as_string().unwrap();
//     //         let seasons = show.seasons().unwrap_or(Vec::new());
//     //         let season_count = seasons.len();
//     //
//     //         println!("{}: Has {} seasons.", show_title, season_count);
//     //
//     //         let mut season_workers = WorkerPool::new(POOL_SIZE);
//     //
//     //         for season in seasons {
//     //             let shared_client = shared_client.clone();
//     //
//     //             season_workers.add_worker(thread::spawn(move || {
//     //                 let s = season.to_object(&shared_client).unwrap();
//     //                 let episodes = s.episodes().unwrap_or(Vec::new());
//     //                 let episode_count = episodes.len();
//     //                 println!("\tSeason {} has {} episodes.",
//     //                          s.attributes().unwrap().find("ordinal").unwrap(),
//     //                          episode_count);
//     //
//     //                 let mut episode_workers = WorkerPool::new(POOL_SIZE);
//     //
//     //                 for episode in episodes {
//     //                     let shared_client = shared_client.clone();
//     //
//     //                     episode_workers.add_worker(thread::spawn(move || {
//     //                         let e = episode.to_object(&shared_client).unwrap();
//     //                         let assets = e.assets().unwrap_or(Vec::new());
//     //                         let asset_count = assets.len();
//     //
//     //                         println!("\t\t{} has {} assets.",
//     //                                  e.attributes()
//     //                                      .unwrap()
//     //                                      .find("title")
//     //                                      .unwrap()
//     //                                      .as_string()
//     //                                      .unwrap(),
//     //                                  asset_count);
//     //                     }));
//     //                 }
//     //
//     //                 episode_workers.wait_for_workers();
//     //             }));
//     //         }
//     //
//     //         season_workers.wait_for_workers();
//     //     }));
//     // }
//     //
//     // pool.wait_for_workers();
// }

///
/// Starts processing
///
fn main() {

    // Set up the database connection.
    let client = Client::connect("localhost", 27017).ok().expect("Failed to initialize client.");
    let db = client.db(config::DB_NAME);
    db.auth(config::DB_USERNAME, config::DB_PASSWORD)
        .ok()
        .expect("Failed to authorize user.");

    let api: ThreadedAPI = Arc::new(APIClient::new("", ""));

    let run_time = show_runner::run(&api, &db);

    match run_time {
        Ok(time) => println!("Run took {} seconds", time),
        Err(_) => println!("Failed to run to completion"),
    }

    // let start_time = time::now();
    //
    // // // Set up the database connection.
    // let client = Client::connect("localhost", 27017).ok().expect("Failed to initialize client.");
    // let db = client.db(config::DB_NAME);
    // db.auth(config::DB_USERNAME, config::DB_PASSWORD)
    //     .ok()
    //     .expect("Failed to authorize user.");
    //
    //
    // let api: ThreadedAPI = Arc::new(APIClient::new("", ""));
    //
    // let pages: usize = 10;
    // let mut page_pool = WorkerPool::new(config::pool_size_for(""));
    //
    // for x in 1..pages {
    //     let shared_db = db.clone();
    //     let shared_api = api.clone();
    //     page_pool.add_worker(thread::spawn(move || {
    //         run(x, &shared_api, &shared_db);
    //     }));
    // }
    //
    // page_pool.wait_for_workers();

    // println!("Run took {} seconds", time::now() - start_time);
    // let response = client.shows().get("625ebeb7-040d-4a70-a6fd-47a04b1acf0f");

    // let show = Object::new(Json::from_str(response.unwrap().as_str())
    //     .unwrap()
    //     .find("data")
    //     .unwrap());



    // let season_handles: Vec<thread::JoinHandle<_>> = seasons.into_iter()
    //     .map(|season| {
    //         let season_client = client.clone();
    //         thread::spawn(move || {
    //             let s = season.to_object(&season_client).unwrap();
    //             let episodes = s.episodes().unwrap_or(Vec::new());
    //             let episode_count = episodes.len();
    //             println!("\tSeason {} has {} episodes.",
    //                      s.attributes().unwrap().find("ordinal").unwrap(),
    //                      episode_count);
    //
    //             let episode_handles: Vec<thread::JoinHandle<_>> = episodes.into_iter()
    //                 .map(|episode| {
    //                     let episode_client = season_client.clone();
    //                     thread::spawn(move || {
    //                         println!("\t\tMake request for {}", episode.value("id").unwrap());
    //                         let e = episode.to_object(&episode_client).unwrap();
    //                         let assets = e.assets().unwrap_or(Vec::new());
    //                         let asset_count = assets.len();
    //
    //                         println!("\t\t{} has {} assets.",
    //                                  e.attributes()
    //                                      .unwrap()
    //                                      .find("title")
    //                                      .unwrap()
    //                                      .as_string()
    //                                      .unwrap(),
    //                                  asset_count);
    //                     })
    //                 })
    //                 .collect();
    //
    //             for episode_handle in episode_handles {
    //                 episode_handle.join();
    //             }
    //         })
    //     })
    //     .collect();
    //
    // for season_handle in season_handles {
    //     season_handle.join();
    // }

    // for season in show.seasons().unwrap() {
    //     let s = season.to_object(&client).unwrap();
    //     let episodes = s.episodes().unwrap_or(Vec::new());
    //     let episode_count = episodes.len();
    //     println!("\tSeason {} has {} episodes.",
    //              s.attributes().unwrap().find("ordinal").unwrap(),
    //              episode_count);
    //
    //     let handles: Vec<thread::JoinHandle<_>> = episodes.into_iter()
    //         .map(|episode| {
    //             thread::spawn(move || {
    //                 println!("Make request for {}", episode.value("id").unwrap());
    //                 let e = episode.to_object(&Client::new("", "")).unwrap();
    //                 let assets = e.assets().unwrap_or(Vec::new());
    //                 let asset_count = assets.len();
    //
    //                 println!("\t\t{} has {} assets.",
    //                          e.attributes().unwrap().find("title").unwrap().as_string().unwrap(),
    //                          asset_count);
    //             })
    //         })
    //         .collect();
    //
    //     for handle in handles {
    //         handle.join();
    //     }
    // }

    // let show = Object::new(match data.find("data") {
    //     Some(value) => value[0].clone(),
    //     None => panic!("Property Error"),
    // });
    //
    // print!("{}", show.seasons()[0].data.pretty());

    // Set up the number of workers.
    // let args:Vec<String> = env::args().collect();
    // let mut num_workers:usize = 5;
    //
    // if args.len() > 1 {
    //     num_workers = args[1].parse::<usize>().unwrap();
    // }
    //
    // println!("Running with {} workers in each pool...", num_workers);
    //
    // // Set up the database connection.
    // let client = Client::connect("localhost", 27017).ok().expect("Failed to initialize client.");
    // let db = client.db(config::DB_NAME);
    // db.auth(config::DB_USERNAME, config::DB_PASSWORD)
    //     .ok().expect("Failed to authorize user.");
    //
    // // Get things going.  After the first run it will look for only updates.
    // let mut first_time = true;
    // let mut start_time = time::now();
    //
    // loop {
    //     video::ingest(first_time, &db, num_workers);
    //     first_time = false;
    //     let end_time = time::now();
    //     println!("Ingest took {} seconds", end_time - start_time);
    //     start_time = end_time;
    // }
}
