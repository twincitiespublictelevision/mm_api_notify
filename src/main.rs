#[macro_use]
extern crate bson;
extern crate chrono;
extern crate clap;
extern crate mm_client;
extern crate mongodb;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod config;
mod error;
mod objects;
mod types;

use clap::{Arg, App};
use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, UTC};
use mm_client::Client as APIClient;
use mm_client::MMCResult;
use mongodb::{Client, ThreadedClient};
use mongodb::db::{Database, ThreadedDatabase};
use mongodb::coll::options::FindOptions;
use serde_json::error::Result as JsonResult;
use serde_json::Value as Json;

use std::sync::Arc;

use error::{IngestError, IngestResult};
use objects::Collection;
use objects::Importable;
use types::RunResult;
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
    let pool_result = rayon::initialize(rayon::Configuration::new()
        .set_num_threads(config::THREAD_POOL_SIZE));

    if pool_result.is_err() {
        println!("Failed to initalize the configured thread pool size. Unable to start.");
        return;
    }

    let db = get_db_connection();
    let api = get_api_client();

    let time_arg = matches.value_of("start-time").map_or(0, |arg| {
        arg.parse::<i64>().expect("Could not parse start time")
    });

    let create_res = if matches.is_present("create") {
        run_create(&api, &db, time_arg).ok()
    } else {
        None
    };

    if !matches.is_present("skip-update") {

        let now = UTC::now().timestamp();

        let update_start_time = if time_arg < (now - config::MM_CHANGELOG_MAX_TIMESPAN) {
            compute_update_start_time(&db)
        } else {
            time_arg + create_res.map_or(0, |(dur, _)| dur.num_seconds())
        };

        run_update_loop(&api, &db, update_start_time)
    }
}

fn get_db_connection() -> Database {
    // // Set up the database connection.
    let client = Client::connect(config::DB_HOST, config::DB_PORT)
        .ok()
        .expect("Failed to initialize client.");
    let db = client.db(config::DB_NAME);
    db.auth(config::DB_USERNAME, config::DB_PASSWORD)
        .ok()
        .expect("Failed to authorize user.");
    db
}

fn get_api_client() -> ThreadedAPI {
    Arc::new(APIClient::qa(config::MM_KEY, config::MM_SECRET)
        .expect("Failed to initalize network client"))
}

fn run_create(api: &ThreadedAPI, db: &Database, run_start_time: i64) -> IngestResult<RunResult> {

    println!("Starting create run from {} : {}",
             run_start_time,
             NaiveDateTime::from_timestamp(run_start_time, 0));

    let result = import_response(api.shows(vec![]), api, db, run_start_time);
    print_runtime("Create", &result);

    result
}

fn compute_update_start_time(db: &Database) -> i64 {
    let newest_db_timestamp = get_most_recent_update_timestamp(db);
    let now = UTC::now().timestamp();

    if newest_db_timestamp < (now - config::MM_CHANGELOG_MAX_TIMESPAN) {
        println!("Newest database record exceeds the maximum threshold for updates. Performing \
                  update run with the maximum threshold. Consider performing a create run to \
                  fully update the database.");
        now - config::MM_CHANGELOG_MAX_TIMESPAN
    } else {
        newest_db_timestamp
    }
}

fn get_most_recent_update_timestamp(db: &Database) -> i64 {
    get_timestamps_from_db(db)
        .into_iter()
        .fold(dawn_of_time(),
              |max_datetime, datetime| std::cmp::max(max_datetime, datetime))
        .timestamp()
}

fn get_timestamps_from_db(db: &Database) -> Vec<DateTime<UTC>> {
    let collections = vec!["asset", "episode", "season", "show", "special"];

    collections.iter()
        .filter_map(|coll_name| {
            let coll = db.collection(coll_name);
            let mut query_options = FindOptions::new().with_limit(1);
            query_options.sort = Some(doc! {
            "attributes.updated_at" => (-1)
        });

            coll.find(None, Some(query_options))
                .ok()
                .and_then(|mut cursor| cursor.next())
                .and_then(|result| result.ok())
                .and_then(|mut document| {
                    document.remove("attributes")
                        .and_then(|attributes| {
                            match attributes {
                                bson::Bson::Document(mut attr) => {
                                    match attr.remove("updated_at") {
                                        Some(bson::Bson::UtcDatetime(datetime)) => Some(datetime),
                                        _ => None,
                                    }
                                }
                                _ => None,
                            }
                        })
                })
        })
        .collect::<Vec<DateTime<UTC>>>()
}

fn dawn_of_time() -> DateTime<UTC> {
    UTC.ymd(1970, 1, 1).and_hms(0, 0, 0)
}

fn run_update_loop(api: &ThreadedAPI, db: &Database, run_start_time: i64) {
    let mut import_start_time = run_start_time;
    let mut import_completion_time = UTC::now().timestamp();
    let mut next_run_time = run_start_time;
    let label = "Update";

    println!("Starting update loop from {} : {}",
             run_start_time,
             NaiveDateTime::from_timestamp(run_start_time, 0));

    loop {
        if UTC::now().timestamp() > next_run_time {

            next_run_time = UTC::now().timestamp() + config::MIN_RUNTIME_DELTA;

            let run_time = run_update(api, db, import_start_time);

            print_runtime(label, &run_time);

            import_start_time = import_completion_time;
            import_completion_time = UTC::now().timestamp();
        }
    }
}

fn run_update(api: &ThreadedAPI, db: &Database, run_start_time: i64) -> IngestResult<RunResult> {

    let date_string = NaiveDateTime::from_timestamp(run_start_time, 0)
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    import_response(api.changelog(vec![("since", date_string.as_str())]),
                    api,
                    db,
                    run_start_time)
}

fn import_response(response: MMCResult<String>,
                   api: &ThreadedAPI,
                   db: &Database,
                   run_start_time: i64)
                   -> IngestResult<RunResult> {
    let start_time = UTC::now();

    let collection = match response {
        Ok(response_string) => {
            let full_json: JsonResult<Json> = serde_json::from_str(response_string.as_str());
            match full_json {
                Ok(json) => Collection::from_json(&json),
                Err(err) => Err(IngestError::Parse(err)),
            }
        }
        Err(err) => Err(IngestError::API(err)),
    };

    collection.and_then(|coll| {
            let res = coll.import(api, db, true, &vec![], run_start_time);
            Ok((UTC::now() - start_time, res))
        })
        .or(Ok((Duration::seconds(0), (0, 1))))
}

fn print_runtime(label: &str, run_time: &IngestResult<RunResult>) {
    match *run_time {
        Ok(ref results) => print_sucess(label, results),
        Err(ref err) => print_failure(label, err),
    }
}

fn print_sucess(label: &str, &(dur, (pass, fail)): &RunResult) {
    println!("{} run took {} seconds with {} successes and {} failures.",
             label,
             dur.num_seconds(),
             pass,
             fail)
}

fn print_failure(label: &str, error: &IngestError) {
    println!("{} run failed to run to completion. {}", label, error)
}
