extern crate video_ingest;
extern crate mongodb;
extern crate time;

use video_ingest::video;
use video_ingest::config;
use std::env;

use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;

///
/// Starts processing
/// 
fn main() {

    // Set up the number of workers.
    let args:Vec<String> = env::args().collect();
    let mut num_workers:usize = 5;

    if args.len() > 1 {
        num_workers = args[1].parse::<usize>().unwrap();
    }

    println!("Running with {} workers in each pool...", num_workers);
   
    // Set up the database connection.
    let client = Client::connect("localhost", 27017).ok().expect("Failed to initialize client.");
    let db = client.db(config::DB_NAME);
    db.auth(config::DB_USERNAME, config::DB_PASSWORD)
        .ok().expect("Failed to authorize user.");

    // Get things going.  After the first run it will look for only updates.    
    let mut first_time = true;
    let mut start_time = time::now();

    loop {
        video::ingest(first_time, &db, num_workers);
        first_time = false;
        let end_time = time::now();
        println!("Ingest took {} seconds", end_time - start_time);
        start_time = end_time;
    }
}