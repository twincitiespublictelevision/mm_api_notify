extern crate time;
extern crate serde;
extern crate serde_json;
extern crate mongodb;
extern crate bson;

use super::worker_pool;
use super::cove;

use std::thread;
use std::sync::{Arc, Mutex};
use self::serde_json::Value;
use self::mongodb::db::{Database, ThreadedDatabase};
use self::mongodb::coll::options::FindOneAndUpdateOptions;
use self::bson::Document;
use self::bson::Bson;

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
pub fn ingest(first_time: bool, db: &Database, num_workers: usize) {
    let total_programs = get_total_programs();
    let mut updated_date = Arc::new(String::from(""));
    
    if !first_time {
        let current_time = time::now_utc();
        updated_date = Arc::new(format!("{}-{}-{}", 1900 + current_time.tm_year, current_time.tm_mon, current_time.tm_mday));
    }

    let mut worker_pool = worker_pool::WorkerPool::new(num_workers);
      
    for i in (0..total_programs).step_by(200) {
        let shared_updated_date = updated_date.clone();
        let shared_db = db.clone();
        
        worker_pool.add_worker(thread::spawn(move || {
            let programs = get_programs(i);
            let mut worker_pool = worker_pool::WorkerPool::new(num_workers);
            let coll = shared_db.collection("programs");
            
            for program in programs {
                let video_ids = Arc::new(Mutex::new(vec![]));
                let program_id = program.program_id;

                let filter = doc! {
                    "program_id" => program_id
                };

                // Can't use doc! macro because it escapes the JSON data.
                let mut update = Document::new();
                update.insert("program_id", program_id);
                update.insert("data", Bson::JavaScriptCode(program.data.clone()));

                let mut options = FindOneAndUpdateOptions::new();
                options.upsert = true;
                coll.find_one_and_replace(filter, update, Some(options)).expect("Can't insert program into database!");

                let shared_updated_date = shared_updated_date.clone();
                let shared_db = shared_db.clone();

                // We need this final shared DB because shared_db is moved into the thread so it can't be used to delete
                // at the end.
                let final_shared_db = shared_db.clone();

                let shared_video_ids = video_ids.clone();
               
                worker_pool.add_worker(thread::spawn(move || {
                    let total_videos = get_video_count_for_program(&shared_updated_date, &program);
                    let mut worker_pool = worker_pool::WorkerPool::new(num_workers);
                    let program = Arc::new(program);
                    let shared_video_ids = shared_video_ids.clone();
                    
                    for j in (0..total_videos).step_by(200) {
                        let shared_program = program.clone();
                        let shared_updated_date = shared_updated_date.clone();
                        let shared_db = shared_db.clone();
                        let shared_video_ids = shared_video_ids.clone();
                        
                        worker_pool.add_worker(thread::spawn(move || {
                            get_videos(&shared_updated_date, j, &shared_program, &shared_db, &shared_video_ids);
                        }));
                    }

                    worker_pool.wait_for_children();
                }));

                let video_ids_str = format!("[{}]", video_ids.lock().unwrap().join(","));
                let filter = doc! {
                    "tp_media_object_id" => video_ids_str,
                    "cond" => "$nin"
                };

                let video_coll = final_shared_db.collection("videos");
                video_coll.delete_many(filter, None).unwrap();
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
fn get_videos(updated_date: &Arc<String>, start_index: u64, program: &Arc<Program>, db: &Database, video_ids: &Arc<Mutex<Vec<String>>>) {
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
    let mut page_video_ids = vec![];

    for video in videos {
        
        //
        // Not everything has a tp_media_object_id.  Ignore the ones that don't.
        //
        match video.as_object().unwrap().get("tp_media_object_id").unwrap().as_u64() {
            Some(tp_media_object_id) => {
                page_video_ids.push(tp_media_object_id.to_string());
               
                let filter = doc! {
                    "tp_media_object_id" => tp_media_object_id
                };

                // Can't use doc! macro because it escapes the data.
                let mut update = Document::new();
                update.insert("tp_media_object_id", tp_media_object_id);
                update.insert("program_id", program.program_id);
                update.insert("data", Bson::JavaScriptCode(video.to_string()));

                let mut options = FindOneAndUpdateOptions::new();
                options.upsert = true;
                
                coll.find_one_and_replace(filter, update, Some(options)).expect("Can't insert video into database!");
            },
            None => {}
        };
    }

    let shared_video_ids = video_ids.clone();
    let mut locked_video_ids = shared_video_ids.lock().unwrap();
    locked_video_ids.extend(page_video_ids);
}