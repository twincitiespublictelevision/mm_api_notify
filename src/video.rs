extern crate mongodb;
extern crate time;
extern crate serde;
extern crate serde_json;

use super::worker_pool;
use std::thread;
use std::sync::Arc;
use self::serde_json::Value;

use super::cove;

///
/// Holds a show object
///
pub struct Program {
    pub data: String,
    pub program_id: u64
}

///
/// Does the actual ingestion
///
pub fn ingest(first_time: bool) {
    let total_programs = get_total_programs();
    let mut updated_date = Arc::new(String::from(""));

    if !first_time {
        let current_time = time::now_utc();
        updated_date = Arc::new(format!("{}-{}-{}", current_time.tm_year, current_time.tm_mon, current_time.tm_mday));
    }
      
    for i in (0..total_programs).step_by(200) {
        let shared_updated_date = updated_date.clone();
        println!("Processing program set {} of {}", i, total_programs);
        let mut worker_pool = worker_pool::WorkerPool::new();
               
        worker_pool.join_handles.push(thread::spawn(move || {
            let programs = get_programs(i);
           
            for program in programs {
                let total_videos = get_video_count_for_program(&program);
                let mut worker_pool = worker_pool::WorkerPool::new();
                let program = Arc::new(program);

                for j in (0..total_videos).step_by(200) {
                    println!("Processing video set {} of {}", j, total_videos);
                    let shared_program = program.clone();
                    let shared_updated_date = shared_updated_date.clone();

                    worker_pool.join_handles.push(thread::spawn(move || {
                        get_videos(&shared_updated_date, j, &shared_program);
                    }));
                }

                worker_pool.wait_for_children();
                break;
            }
        }));

        worker_pool.wait_for_children();
        break;
    }
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
fn get_video_count_for_program<'a>(program: &Program) -> u64 {
    cove::video_api("videos", vec![["program_id", program.program_id.to_string().as_str()]]).as_object().unwrap().get("count").unwrap().as_u64().unwrap()
}

///
/// Gets all videos from COVE for a program, 200 at a time
///
fn get_videos<'a>(updated_date: &Arc<String>, start_index: u64, program: &Arc<Program>) {
    let program_id = program.program_id.to_string();
    let str_start_index = start_index.to_string();
    let mut params = vec![["program_id", program_id.as_str()], ["limit_start", str_start_index.as_str()]];
    
    if updated_date.as_str() == "" {
        params.push(["filter_record_last_updated_datetime__gt", updated_date.as_str()]);
    }

    let cove_data = cove::video_api("videos", params);
    let videos:&Vec<Value> = cove_data.as_object().unwrap().get("results").unwrap().as_array().unwrap();

    for video in videos {

        // Do the mongo insert.
    }
}