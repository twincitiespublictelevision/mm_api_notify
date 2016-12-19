#![feature(proc_macro)]
extern crate mongodb;
extern crate time;
#[macro_use]
extern crate bson;
extern crate core_data_client;
extern crate rustc_serialize;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate chrono;

use self::chrono::DateTime;
use self::chrono::UTC;
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

    // Create a new API instance
    let api: ThreadedAPI = Arc::new(APIClient::new("", ""));

    let mut import_start_time = UTC::now().timestamp();
    let mut import_completion_time = UTC::now().timestamp();
    let mut next_run_time = 0;

    println!("Start run");
    while true {

        if UTC::now().timestamp() > next_run_time {
            let label = if next_run_time == 0 {
                "Initial"
            } else {
                "Update"
            };

            next_run_time = UTC::now().timestamp() + config::MIN_RUNTIME_DELTA;

            let run_time = show_runner::run(&api, &db, import_start_time);
            // let run_time = show_runner::run_show("antiques-roadshow", &api, &db, import_start_time);

            match run_time {
                Ok(time) => println!("{} run took {} seconds", label, time),
                Err(_) => println!("Failed to run to completion"),
            }

            import_start_time = import_completion_time;
            import_completion_time = UTC::now().timestamp();
        }
    }
}
