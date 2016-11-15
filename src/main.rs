extern crate video_ingest;
extern crate mongodb;

use video_ingest::video;
use video_ingest::config;

use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;

///
/// Starts processing
/// 
fn main() {

    // Set up the database connection
    let client = Client::connect("localhost", 27017).ok().expect("Failed to initialize client.");
    let db = client.db(config::DB_NAME);
    db.auth(config::DB_USERNAME, config::DB_PASSWORD)
        .ok().expect("Failed to authorize user.");

    // Get things going.  After the first run it will look for only updates.    
    let mut first_time = true;

    loop {
        video::ingest(first_time, &db);
        first_time = false;
    }
}