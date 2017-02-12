extern crate app_dirs;
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

mod api;
mod config;
mod error;
mod objects;
mod runtime;
mod types;

use app_dirs::{AppDataType, AppInfo, get_app_dir};
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

use config::{DBConfig, MMConfig, parse_config};
use error::{IngestError, IngestResult};
use objects::Collection;
use objects::Importable;
use runtime::Runtime;
use types::RunResult;
use types::ThreadedAPI;

///
/// Starts processing
///
fn main() {

    let matches = App::new("Video Ingest")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("build")
            .short("b")
            .long("build")
            .takes_value(false)
            .help("Performs a full build run prior to entering the update loop"))
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .takes_value(true)
            .help("Path to configuration file"))
        .arg(Arg::with_name("skip-update")
            .short("k")
            .long("skip-update")
            .takes_value(false)
            .help("Prevents update loop from running"))
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .takes_value(false)
            .help("Provides feedback on import during processing"))
        .arg(Arg::with_name("start-time").long("start-time").short("t").takes_value(true))
        .get_matches();

    let config_path = if !matches.is_present("config") {
            let info = AppInfo {
                name: env!("CARGO_PKG_NAME"),
                author: env!("CARGO_PKG_AUTHORS"),
            };

            let path = get_app_dir(AppDataType::UserConfig, &info, "/")
                .and_then(|mut path| {
                    path.push("config.toml");
                    Ok(path)
                })
                .expect("Failed to run. Unable to determine path default config location.");

            path.to_str().map(|str| str.to_string())
        } else {
            matches.value_of("config").map(|str| str.to_string())
        }
        .expect("Failed to run. Unable to parse path to default config location.");

    parse_config(config_path.as_str())
        .ok_or(IngestError::InvalidConfig)
        .and_then(|config| {

            // Initialize the thread pools
            rayon::initialize(rayon::Configuration::new().set_num_threads(config.thread_pool_size))
                .map_err(IngestError::ThreadPool)
                .or_else(|err| {
                    println!("Failed to initalize the configured thread pool size. Unable to \
                              start.");
                    Err(err)
                })
                .and_then(|_| {
                    let db = get_db_connection(&config.db);
                    let api = get_api_client(&config.mm);

                    let runtime = Runtime {
                        api: api,
                        config: config,
                        db: db,
                        verbose: matches.is_present("verbose"),
                    };

                    let time_arg = matches.value_of("start-time").map_or(0, |arg| {
                        arg.parse::<i64>().expect("Could not parse start time")
                    });

                    let build_res = if matches.is_present("build") {
                        run_build(&runtime, time_arg).ok()
                    } else {
                        None
                    };

                    if !matches.is_present("skip-update") {

                        let now = UTC::now().timestamp();

                        let update_start_time =
                            if time_arg < (now - runtime.config.mm.changelog_max_timespan) {
                                compute_update_start_time(&runtime.db,
                                                          runtime.config.mm.changelog_max_timespan)
                            } else {
                                time_arg + build_res.map_or(0, |(dur, _)| dur.num_seconds())
                            };

                        run_update_loop(&runtime,
                                        update_start_time,
                                        runtime.config.min_runtime_delta)
                    };

                    Ok(())
                })
        });
}

fn get_db_connection(config: &DBConfig) -> Database {
    // // Set up the database connection.
    let client = Client::connect(config.host.as_str(), config.port)
        .expect("Failed to initialize MongoDB client.");
    let db = client.db(config.name.as_str());
    db.auth(config.username.as_str(), config.password.as_str())
        .expect("Failed to authorize MongoDB user.");
    db
}

fn get_api_client(config: &MMConfig) -> ThreadedAPI {
    Arc::new(APIClient::qa(config.key.as_str(), config.secret.as_str())
        .expect("Failed to initalize network client"))
}

fn run_build(runtime: &Runtime, run_start_time: i64) -> IngestResult<RunResult> {

    println!("Starting build run from {} : {}",
             run_start_time,
             NaiveDateTime::from_timestamp(run_start_time, 0));

    let result = import_response(runtime.api.shows(vec![]), runtime, run_start_time);
    print_runtime("Create", &result);

    result
}

fn compute_update_start_time(db: &Database, max_timespan: i64) -> i64 {
    let newest_db_timestamp = get_most_recent_update_timestamp(db);
    let now = UTC::now().timestamp();

    if newest_db_timestamp < (now - max_timespan) {
        println!("Newest database record exceeds the maximum threshold for updates. Performing \
                  update run with the maximum threshold. Consider performing a build (-b) run to \
                  fully update the database.");
        now - max_timespan
    } else {
        newest_db_timestamp
    }
}

fn get_most_recent_update_timestamp(db: &Database) -> i64 {
    get_timestamps_from_db(db)
        .into_iter()
        .fold(dawn_of_time(), std::cmp::max)
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
                        .and_then(|attributes| match attributes {
                            bson::Bson::Document(mut attr) => {
                                match attr.remove("updated_at") {
                                    Some(bson::Bson::UtcDatetime(datetime)) => Some(datetime),
                                    _ => None,
                                }
                            }
                            _ => None,
                        })
                })
        })
        .collect::<Vec<DateTime<UTC>>>()
}

fn dawn_of_time() -> DateTime<UTC> {
    UTC.ymd(1970, 1, 1).and_hms(0, 0, 0)
}

fn run_update_loop(runtime: &Runtime, run_start_time: i64, delta: i64) {
    let mut import_start_time = run_start_time;
    let mut import_completion_time = UTC::now().timestamp();
    let mut next_run_time = run_start_time;
    let label = "Update";

    println!("Starting update loop from {} : {}",
             run_start_time,
             NaiveDateTime::from_timestamp(run_start_time, 0));

    loop {
        if UTC::now().timestamp() > next_run_time {

            next_run_time = UTC::now().timestamp() + delta;

            let run_time = run_update(runtime, import_start_time);

            print_runtime(label, &run_time);

            import_start_time = import_completion_time;
            import_completion_time = UTC::now().timestamp();
        }
    }
}

fn run_update(runtime: &Runtime, run_start_time: i64) -> IngestResult<RunResult> {

    let date_string = NaiveDateTime::from_timestamp(run_start_time, 0)
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    import_response(runtime.api.changelog(vec![("since", date_string.as_str())]),
                    runtime,
                    run_start_time)
}

fn import_response(response: MMCResult<String>,
                   runtime: &Runtime,
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
            let res = coll.import(runtime, true, run_start_time);
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
