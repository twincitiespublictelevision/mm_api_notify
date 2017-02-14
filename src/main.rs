//! # mm_api_import

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
#[macro_use]
extern crate serde_json;

mod api;
mod config;
mod error;
mod objects;
mod runtime;
mod storage;
mod types;

use app_dirs::{AppDataType, AppInfo, get_app_dir};
use clap::{Arg, App};
use chrono::{Duration, NaiveDateTime, UTC};
use mm_client::Client as APIClient;
use mm_client::MMCResult;
use serde_json::error::Result as JsonResult;
use serde_json::Value as Json;

use std::sync::Arc;

use config::{DBConfig, MMConfig, parse_config};
use error::{IngestError, IngestResult};
use objects::{Collection, Importable};
use runtime::Runtime;
use storage::{MongoStore, Storage};
use types::{RunResult, StorageEngine, ThreadedAPI};

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
                    let store = get_store(&config.db);
                    let api = get_api_client(&config.mm);

                    let runtime = Runtime {
                        api: api,
                        config: config,
                        store: store,
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
                                compute_update_start_time(runtime.store.updated_at(),
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

fn get_store(config: &DBConfig) -> MongoStore {
    MongoStore::new(Some(config)).expect("Failed to connect to storage")
}

fn get_api_client(config: &MMConfig) -> ThreadedAPI {
    Arc::new(APIClient::staging(config.key.as_str(), config.secret.as_str())
        .expect("Failed to initalize network client"))
}

fn run_build<T: StorageEngine>(runtime: &Runtime<T>,
                               run_start_time: i64)
                               -> IngestResult<RunResult> {

    println!("Starting build run from {} : {}",
             run_start_time,
             NaiveDateTime::from_timestamp(run_start_time, 0));

    let result = import_response(runtime.api.shows(vec![]), runtime, run_start_time);
    print_runtime("Create", &result);

    result
}

fn compute_update_start_time(last_updated_at: Option<i64>, max_timespan: i64) -> i64 {
    let newest_db_timestamp = last_updated_at.unwrap_or(0);
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

fn run_update_loop<T: StorageEngine>(runtime: &Runtime<T>, run_start_time: i64, delta: i64) {
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

fn run_update<T: StorageEngine>(runtime: &Runtime<T>,
                                run_start_time: i64)
                                -> IngestResult<RunResult> {

    let date_string = NaiveDateTime::from_timestamp(run_start_time, 0)
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    import_response(runtime.api.changelog(vec![("since", date_string.as_str())]),
                    runtime,
                    run_start_time)
}

fn import_response<T: StorageEngine>(response: MMCResult<String>,
                                     runtime: &Runtime<T>,
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
