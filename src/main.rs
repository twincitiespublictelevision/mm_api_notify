// extern crate video_ingest;
// extern crate mongodb;
// extern crate time;
//
// use video_ingest::video;
// use video_ingest::config;
// use std::env;
//
// use mongodb::{Client, ThreadedClient};
// use mongodb::db::ThreadedDatabase;

extern crate core_data_client;
extern crate rustc_serialize;

mod error;
mod objects;

use objects::Object;
use objects::Ref;
use core_data_client::Client;
use self::rustc_serialize::json::Json;
use std::thread;
use error::IngestResult;

///
/// Starts processing
///
fn main() {

    let client = Client::new("", "");
    let response = client.shows().get("41970cd4-dd4b-4f0e-b6ce-621f3ec32e04");

    // let mut shows: Vec<Object> = Json::from_str(response.unwrap().as_str())
    //     .unwrap()
    //     .find("data")
    //     .unwrap()
    //     .as_array()
    //     .unwrap()
    //     .into_iter()
    //     .map(|obj| Object::new(obj))
    //     .collect();

    let show = Object::new(Json::from_str(response.unwrap().as_str())
        .unwrap()
        .find("data")
        .unwrap());

    let show_title =
        show.data.find("attributes").unwrap().find("title").unwrap().as_string().unwrap();
    let seasons = show.seasons().unwrap_or(Vec::new());
    let season_count = seasons.len();

    println!("{}: Has {} seasons.", show_title, season_count);

    let season_handles: Vec<thread::JoinHandle<_>> = seasons.into_iter()
        .map(|season| {
            thread::spawn(move || {
                let s = season.to_object(&Client::new("", "")).unwrap();
                let episodes = s.episodes().unwrap_or(Vec::new());
                let episode_count = episodes.len();
                println!("\tSeason {} has {} episodes.",
                         s.attributes().unwrap().find("ordinal").unwrap(),
                         episode_count);

                let episode_handles: Vec<thread::JoinHandle<_>> = episodes.into_iter()
                    .map(|episode| {
                        thread::spawn(move || {
                            println!("Make request for {}", episode.value("id").unwrap());
                            let e = episode.to_object(&Client::new("", "")).unwrap();
                            let assets = e.assets().unwrap_or(Vec::new());
                            let asset_count = assets.len();

                            println!("\t\t{} has {} assets.",
                                     e.attributes()
                                         .unwrap()
                                         .find("title")
                                         .unwrap()
                                         .as_string()
                                         .unwrap(),
                                     asset_count);
                        })
                    })
                    .collect();

                for episode_handle in episode_handles {
                    episode_handle.join();
                }
            })
        })
        .collect();

    for season_handle in season_handles {
        season_handle.join();
    }

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
