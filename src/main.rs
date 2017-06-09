//! # mm_api_notify
#[cfg(test)]
extern crate mockito;

extern crate app_dirs;
#[macro_use]
extern crate bson;
extern crate chrono;
extern crate clap;
extern crate fern;
#[macro_use]
extern crate log;
extern crate mm_client;
extern crate mongodb;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

mod client;
mod config;
mod error;
mod hooks;
mod objects;
mod runtime;
mod storage;
mod types;

use app_dirs::{AppDataType, AppInfo, get_app_dir};
use clap::{Arg, App};
use chrono::{Duration, NaiveDateTime, UTC};
use serde_json::error::Result as JsonResult;
use serde_json::Value as Json;

use std::str::FromStr;
use std::{thread, time};

use client::{APIClient, ClientResult, MMClient};
use config::{DBConfig, APIConfig, parse_config};
use error::{IngestError, IngestResult};
use hooks::Payload;
use objects::{Collection, Importable};
use runtime::Runtime;
use storage::{MongoStore, Storage};
use types::{RunResult, StorageEngine, ThreadedAPI};

///
/// Starts processing
///
fn main() {

    let matches = App::new("Media Manager Notifier")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .takes_value(true)
            .help("Path to configuration file"))
        .arg(Arg::with_name("build")
            .short("b")
            .long("build")
            .takes_value(false)
            .help("Performs a full build run prior to entering the update loop"))
        .arg(Arg::with_name("skip-update")
            .short("k")
            .long("skip-update")
            .takes_value(false)
            .help("Prevents update loop from running"))
        .arg(Arg::with_name("start-time")
            .long("start-time")
            .short("t")
            .takes_value(true)
            .help("Defines the timestamp to start building or updating from"))
        .arg(Arg::with_name("log-level")
            .long("log-level")
            .short("l")
            .takes_value(true)
            .help("Defines the log level to run at. Defaults to WARN"))
        .arg(Arg::with_name("query")
            .long("query")
            .short("q")
            .takes_value(true)
            .number_of_values(2)
            .value_names(&["type", "id"])
            .conflicts_with_all(&["build", "skip-update", "start-time"])
            .help("Queries the cache with a type and id pair and displays the payload that the \
                   runner will emit"))
        .get_matches();

    let config_path =
        if !matches.is_present("config") {
                let info = AppInfo {
                    name: env!("CARGO_PKG_NAME"),
                    author: env!("CARGO_PKG_AUTHORS"),
                };

                let path =
                    get_app_dir(AppDataType::UserConfig, &info, "/")
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

    let conf_res = parse_config(config_path.as_str())
        .ok_or(IngestError::InvalidConfig)
        .and_then(|config| {

            // Initialize logging
            if let &Some(ref log_location) = &config.log.location {

                let config_log_level_filter = match config.log.level {
                    Some(ref level) => log::LogLevelFilter::from_str(level.as_str()).ok(),
                    None => None,
                };

                let log_level = matches
                    .value_of("log-level")
                    .and_then(|level| log::LogLevelFilter::from_str(level).ok())
                    .or(config_log_level_filter)
                    .unwrap_or(log::LogLevelFilter::Warn);

                fern::Dispatch::new()
                    .format(|out, message, record| {
                                out.finish(format_args!("[{}][{}] {}",
                                                UTC::now().format("%Y-%m-%d][%H:%M:%S"),
                                                record.level(),
                                                message))
                            })
                    .level(log_level)
                    .chain(std::io::stdout())
                    .chain(fern::log_file(log_location.as_str()).expect("Failed to open log file"))
                    .apply()
                    .expect("Failed to initialize logger");
            }

            // Initialize the thread pools
            rayon::initialize(rayon::Configuration::new().num_threads(config.thread_pool_size))
                .or_else(|err| {
                    panic!("Failed to initalize the configured thread pool size. Unable to \
                              start. {}",
                           err);
                })
                .and_then(|_| {
                    let store = get_store(&config.db);
                    let api = get_api_client(&config.mm);

                    let runtime = Runtime {
                        api: api,
                        config: config,
                        store: store,
                    };

                    if let Some(query) = matches.values_of("query") {
                        let query_args = query.collect::<Vec<&str>>();
                        match runtime.store.get(query_args[1], query_args[0]) {
                            Some(Ok(obj)) => {
                                match Payload::from_object(&obj, &runtime.store) {
                                    Some(payload) => {
                                        println!("{}",
                                                 serde_json::to_string_pretty(&payload).unwrap())
                                    }
                                    None => error!("Failed to generate payload from object."),
                                }
                            }
                            _ => println!("Could not find the requested object in the cache."),
                        };
                    } else {
                        let time_arg =
                            matches
                                .value_of("start-time")
                                .map_or(0, |arg| {
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

                            run_update_loop(&runtime, update_start_time)
                        };
                    }

                    Ok(())
                })
        });

    conf_res.expect("Failed to parse config.")
}

fn get_store(config: &DBConfig) -> MongoStore {
    MongoStore::new(Some(config)).expect("Failed to connect to storage")
}

fn get_api_client(config: &APIConfig) -> MMClient {
    MMClient::new(Some(config)).expect("Failed to initalize network client")
}

fn run_build<T: StorageEngine, S: ThreadedAPI>(runtime: &Runtime<T, S>,
                                               run_start_time: i64)
                                               -> IngestResult<RunResult> {

    info!("Starting build run from {} : {}",
          run_start_time,
          NaiveDateTime::from_timestamp(run_start_time, 0));

    let result = import_response(runtime.api.all_shows(), runtime, run_start_time);
    print_runtime("Create", &result);

    result
}

fn compute_update_start_time(last_updated_at: Option<i64>, max_timespan: i64) -> i64 {
    let newest_db_timestamp = last_updated_at.unwrap_or(0);
    let now = UTC::now().timestamp();

    if newest_db_timestamp < (now - max_timespan) {
        warn!("Newest database record exceeds the maximum threshold for updates. Performing \
                  update run with the maximum threshold. Consider performing a build (-b) run to \
                  fully update the database.");
        now - max_timespan
    } else {
        newest_db_timestamp
    }
}

fn run_update_loop<T: StorageEngine, S: ThreadedAPI>(runtime: &Runtime<T, S>,
                                                     run_start_time: i64) {
    let mut import_start_time = run_start_time;
    let mut import_completion_time = UTC::now().timestamp();
    let mut next_run_time = run_start_time;
    let label = "Update";

    info!("Starting update loop from {} : {}",
          run_start_time,
          NaiveDateTime::from_timestamp(run_start_time, 0));

    loop {
        if UTC::now().timestamp() > next_run_time {

            next_run_time = UTC::now().timestamp() + runtime.config.min_runtime_delta -
                            runtime.config.lookback_timeframe;

            let run_time = run_update(runtime, import_start_time);

            print_runtime(label, &run_time);

            import_start_time = import_completion_time;
            import_completion_time = UTC::now().timestamp();

            let diff = next_run_time - import_completion_time;

            if diff > 0 {
                thread::sleep(time::Duration::from_secs(diff as u64));
            }
        }
    }
}

fn run_update<T: StorageEngine, S: ThreadedAPI>(runtime: &Runtime<T, S>,
                                                run_start_time: i64)
                                                -> IngestResult<RunResult> {

    let date_string = NaiveDateTime::from_timestamp(run_start_time, 0)
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    import_response(runtime.api.changes(date_string.as_str()),
                    runtime,
                    run_start_time)
}

fn import_response<T: StorageEngine, S: ThreadedAPI>(response: ClientResult<String>,
                                                     runtime: &Runtime<T, S>,
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
        Err(err) => Err(IngestError::Client(err)),
    };

    collection
        .and_then(|coll| {
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
    info!("{} run took {} seconds with {} successes and {} failures.",
          label,
          dur.num_seconds(),
          pass,
          fail)
}

fn print_failure(label: &str, error: &IngestError) {
    error!("{} run failed to run to completion. {}", label, error)
}
