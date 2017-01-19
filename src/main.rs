#![feature(proc_macro)]

#[macro_use]
extern crate bson;
extern crate chrono;
extern crate core_data_client;
extern crate mongodb;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod config;
mod error;
mod objects;
mod show_runner;
mod types;

use self::chrono::UTC;
use self::serde_json::Value as Json;

use core_data_client::Client as APIClient;
use core_data_client::CDCResult;
use core_data_client::Endpoints;
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;

use std::sync::Arc;

use error::IngestError;
use error::IngestResult;
use objects::Importable;
use objects::Object;
use types::ThreadedAPI;


///
/// Starts processing
///
fn main() {

    // // Set up the database connection.
    let client = Client::connect("localhost", 27017).ok().expect("Failed to initialize client.");
    let db = client.db(config::DB_NAME);
    db.auth(config::DB_USERNAME, config::DB_PASSWORD)
        .ok()
        .expect("Failed to authorize user.");

    // Create a new API instance
    let api: ThreadedAPI = Arc::new(APIClient::new(config::MM_KEY, config::MM_SECRET));

    let obj = Object::from_json(&parse_response(api.get(Endpoints::Show, "edf533df-1057-4223-8b05-4fead0c8dedd")).unwrap()).unwrap();
    obj.import(&api, &db, true, &vec![], 0);

    // let mut import_start_time = 0;
    // let mut import_completion_time = UTC::now().timestamp();
    // let mut next_run_time = 0;
    //
    // println!("Start run");
    // loop {
    //
    //     if UTC::now().timestamp() > next_run_time {
    //         let label = if next_run_time == 0 {
    //             "Initial"
    //         } else {
    //             "Update"
    //         };
    //
    //         next_run_time = UTC::now().timestamp() + config::MIN_RUNTIME_DELTA;
    //
    //         let run_time = show_runner::run(&api, &db, import_start_time);
    //         // let run_time = show_runner::run_show("antiques-roadshow", &api, &db, import_start_time);
    //
    //         match run_time {
    //             Ok(time) => println!("{} run took {} seconds", label, time),
    //             Err(_) => println!("Failed to run to completion"),
    //         }
    //
    //         import_start_time = import_completion_time;
    //         import_completion_time = UTC::now().timestamp();
    //     }
    // }
}

fn parse_response(response: CDCResult<String>) -> IngestResult<Json> {
    match response.map_err(IngestError::API) {
        Ok(json_string) => serde_json::from_str(json_string.as_str()).map_err(IngestError::Parse),
        Err(err) => Err(err),
    }
}
