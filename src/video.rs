extern crate time;
extern crate serde;
extern crate serde_json;
extern crate mongodb;

use super::worker_pool;
use super::cove;

use std::thread;
use std::sync::Arc;
use self::serde_json::Value;
use self::mongodb::db::{Database, ThreadedDatabase};
use self::mongodb::coll::options::FindOneAndUpdateOptions;

///
/// Holds a program object
///
pub struct Program {
    pub data: String,
    pub program_id: u64
}

///
/// Does the actual ingestion
///
pub fn ingest(first_time: bool, db: &Database) {
    let total_programs = get_total_programs();
    let mut updated_date = Arc::new(String::from(""));
   
    if !first_time {
        let current_time = time::now_utc();
        updated_date = Arc::new(format!("{}-{}-{}", 1900 + current_time.tm_year, current_time.tm_mon, current_time.tm_mday));
    }

    let mut worker_pool = worker_pool::WorkerPool::new(5);
      
    for i in (0..total_programs).step_by(200) {
        let shared_updated_date = updated_date.clone();
        let shared_db = db.clone();
        
        worker_pool.add_worker(thread::spawn(move || {
            let programs = get_programs(i);
            let mut worker_pool = worker_pool::WorkerPool::new(5);
            let coll = shared_db.collection("programs");
            
            for program in programs {
                let program_id = program.program_id;
                let program_data = program.data.clone();

                let filter = doc! {
                    "program_id" => program_id
                };

                let update = doc! {
                    "program_id" => program_id,
                    "data" => program_data
                };

                let mut options = FindOneAndUpdateOptions::new();
                options.upsert = true;
                
                coll.find_one_and_replace(filter, update, Some(options)).expect("Can't insert program into database!");
                let shared_updated_date = shared_updated_date.clone();
                let shared_db = shared_db.clone();
               
                worker_pool.add_worker(thread::spawn(move || {
                    let total_videos = get_video_count_for_program(&shared_updated_date, &program);
                    let mut worker_pool = worker_pool::WorkerPool::new(5);
                    let program = Arc::new(program);

                    for j in (0..total_videos).step_by(200) {
                        let shared_program = program.clone();
                        let shared_updated_date = shared_updated_date.clone();
                        let shared_db = shared_db.clone();
                        
                        worker_pool.add_worker(thread::spawn(move || {
                            get_videos(&shared_updated_date, j, &shared_program, &shared_db);
                        }));
                    }

                    worker_pool.wait_for_children();
                }));
            }

            worker_pool.wait_for_children();
        }));
    }

    worker_pool.wait_for_children();
}

/// 
/// Gets the total programs to break them up
///
fn get_total_programs() -> u64 {
    cove::video_api("programs", vec![]).as_object().unwrap().get("count").unwrap().as_u64().unwrap()
}

///
/// Gets all the shows from COVE
///
fn get_programs(start_index: u64) -> Vec<Program> {
    let cove_data = cove::video_api("programs", vec![["limit_start", start_index.to_string().as_str()]]);
    let programs: &Vec<Value> = cove_data.as_object().unwrap().get("results").unwrap().as_array().unwrap();
    let mut programs_data = vec![];

    for program in programs {
        let program_uri = program.as_object().unwrap().get("resource_uri").unwrap().as_str().unwrap();
        let program_id: u64 = program_uri.split("/").nth(4).unwrap().parse().unwrap();

        // Do the mongo insert.

        programs_data.push(Program {data: program.to_string(), program_id: program_id});
    }

    programs_data
}

///
/// Gets the total videos for a program so they can be chunked
///
fn get_video_count_for_program<'a>(updated_date: &Arc<String>, program: &Program) -> u64 {
    let program_id = program.program_id.to_string();
    let mut params = vec![
        ["filter_program", program_id.as_str()], 
        ["exclude_type", "Other"]
    ];

    if updated_date.as_str() != "" {
        params.push(["filter_record_last_updated_datetime__gt", updated_date.as_str()]);
    }

    cove::video_api("videos", params).as_object().unwrap().get("count").unwrap().as_u64().unwrap()
}

///
/// Gets all videos from COVE for a program, 200 at a time
///
fn get_videos<'a>(updated_date: &Arc<String>, start_index: u64, program: &Arc<Program>, db: &Database) {
    let program_id = program.program_id.to_string();
    let str_start_index = start_index.to_string();
    let mut params = vec![
        ["filter_program", program_id.as_str()], 
        ["limit_start", str_start_index.as_str()],
        ["exclude_type", "Other"]
    ];
    
    if updated_date.as_str() != "" {
        params.push(["filter_record_last_updated_datetime__gt", updated_date.as_str()]);
    }

    let cove_data = cove::video_api("videos", params);
    let videos:&Vec<Value> = cove_data.as_object().unwrap().get("results").unwrap().as_array().unwrap();
    let coll = db.collection("videos");

    for video in videos {
        
        //
        // Not everything has a tp_media_object_id.  Ignore the ones that don't.
        //
        match video.as_object().unwrap().get("tp_media_object_id").unwrap().as_u64() {
            Some(tp_media_object_id) => {
                let video_data = video.to_string();

                let filter = doc! {
                    "tp_media_object_id" => tp_media_object_id
                };

                let update = doc! {
                    "tp_media_object_id" => tp_media_object_id,
                    "data" => video_data
                };

                let mut options = FindOneAndUpdateOptions::new();
                options.upsert = true;
                
                coll.find_one_and_replace(filter, update, Some(options)).expect("Can't insert video into database!");
            },
            None => {}
        };
    }
}