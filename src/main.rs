#[macro_use]
extern crate bson;
extern crate chrono;
extern crate clap;
extern crate core_data_client;
extern crate mongodb;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod config;
mod error;
mod objects;
mod show_runner;
mod types;
mod worker_pool;

use clap::{Arg, App};
use chrono::{Duration, UTC};
use core_data_client::Client as APIClient;
use core_data_client::CDCResult;
use mongodb::{Client, ThreadedClient};
use mongodb::db::{Database, ThreadedDatabase};
use serde_json::error::Result as JsonResult;
use serde_json::Value as Json;

use std::sync::Arc;

use objects::Collection;
use objects::Importable;
use error::{IngestError, IngestResult};
use types::ThreadedAPI;


///
/// Starts processing
///
fn main() {

    let matches = App::new("Video Ingest")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("create")
            .short("c")
            .long("create")
            .takes_value(false)
            .help("Performs a full create run prior to entering the update loop"))
        .arg(Arg::with_name("skip-update")
            .short("k")
            .long("skip-update")
            .takes_value(false)
            .help("Prevents update loop from running"))
        .arg(Arg::with_name("start-time").long("start-time").short("t").takes_value(true))
        .get_matches();

    // Initialize the thread pools
    rayon::initialize(rayon::Configuration::new().set_num_threads(64));

    let db = get_db_connection();
    let api = get_api_client();

    let time_arg = matches.value_of("start-time").map_or(0, |arg| {
        arg.parse::<i64>().expect("Could not parse start time")
    });

    // panic!("{:?} {:?}", matches, time_arg);

    let create_res = if matches.is_present("create") {
        run_create(&api, &db, time_arg).ok()
    } else {
        None
    };

    if !matches.is_present("skip-update") {
        run_update_loop(&api,
                        &db,
                        time_arg + create_res.map_or(0, |dur| dur.num_seconds()))
    }
}

fn get_db_connection() -> Database {
    // // Set up the database connection.
    let client = Client::connect(config::DB_NAME, config::DB_PORT)
        .ok()
        .expect("Failed to initialize client.");
    let db = client.db(config::DB_NAME);
    db.auth(config::DB_USERNAME, config::DB_PASSWORD)
        .ok()
        .expect("Failed to authorize user.");

    panic!("authed");
    db
}

fn get_api_client() -> ThreadedAPI {
    Arc::new(APIClient::qa(config::MM_KEY, config::MM_SECRET)
        .expect("Failed to initalize network client"))
}

fn run_create(api: &ThreadedAPI, db: &Database, run_start_time: i64) -> IngestResult<Duration> {

    println!("Starting create run from {}", run_start_time);

    let result = import_response(api.shows(vec![]), api, db, run_start_time);
    match result {
        Ok(ref time) => output_sucess("Create", time),
        Err(ref err) => output_failure("Create", err),
    };

    result
}

fn compute_update_start_time() -> i64 {
    0
}

fn run_update_loop(api: &ThreadedAPI, db: &Database, run_start_time: i64) {
    let mut import_start_time = 0;
    let mut import_completion_time = UTC::now().timestamp();
    let mut next_run_time = 0;
    let label = "Update";

    println!("Starting update loop from {}", run_start_time);

    loop {
        if UTC::now().timestamp() > next_run_time {

            next_run_time = UTC::now().timestamp() + config::MIN_RUNTIME_DELTA;

            let run_time = run_update(api, db, import_start_time);

            match run_time {
                Ok(ref time) => output_sucess(label, time),
                Err(ref err) => output_failure(label, err),
            }

            import_start_time = import_completion_time;
            import_completion_time = UTC::now().timestamp();
        }
    }
}

fn run_update(api: &ThreadedAPI, db: &Database, run_start_time: i64) -> IngestResult<Duration> {
    import_response(api.changelog(vec![("since", run_start_time.to_string().as_str())]),
                    api,
                    db,
                    run_start_time)
}

fn import_response(response: CDCResult<String>,
                   api: &ThreadedAPI,
                   db: &Database,
                   run_start_time: i64)
                   -> IngestResult<Duration> {
    let start_time = UTC::now();

    let collection = match response {
        Ok(response_string) => {
            let full_json: JsonResult<Json> = serde_json::from_str(response_string.as_str());
            match full_json {
                Ok(mut json) => Collection::from_json(&json),
                Err(err) => Err(IngestError::Parse(err)),
            }
        }
        Err(err) => Err(IngestError::API(err)),
    };

    collection.and_then(|coll| {
            coll.import(api, db, true, &vec![], run_start_time);
            Ok(UTC::now() - start_time)
        })
        .or(Ok(Duration::seconds(0)))
}

fn output_sucess(label: &str, duration: &Duration) {
    println!("{} run took {} seconds", label, duration)
}

fn output_failure(label: &str, error: &IngestError) {
    println!("{} run failed to run to completion. {}", label, error)
}
