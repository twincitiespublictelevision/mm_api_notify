extern crate mongodb;

use super::worker_pool;
use std::thread;
use std::sync::Arc;
use curl::easy::Easy;

///
/// Holds a video object
///
pub struct Video<'a> {
    pub tp_media_object_id: &'a str,
    pub data: &'a str
}

impl<'a> Video<'a> {

    ///
    /// Does a Mongo upsert off the tp_media_object_id
    ///
    pub fn save(&self) {

    }
} 

///
/// Holds a show object
///
pub struct Program<'a> {
    pub program_id: &'a str,
    pub data: &'a str,
}

impl<'a> Program<'a> {

    ///
    /// Does a Mongo upsert off the program_id
    ///
    pub fn save(&self) {

    }
}

///
/// Does the actual ingestion
///
pub fn ingest() {
    let total_programs = get_total_programs();

    for i in (0..total_programs).step_by(200) {
        let mut worker_pool = worker_pool::WorkerPool::new();
        
        worker_pool.join_handles.push(thread::spawn(move || {
            let programs = get_programs(i);
           
            for program in programs {
                let total_videos = get_video_count_for_program(&program);
                let mut worker_pool = worker_pool::WorkerPool::new();
                let program = Arc::new(program);
                    
                for j in (0..total_videos).step_by(200) {
                    let shared_program = program.clone();

                    worker_pool.join_handles.push(thread::spawn(move || {
                        get_videos(j, &shared_program);
                    }));
                }

                worker_pool.wait_for_children();
            }
        }));

        worker_pool.wait_for_children();
    }
}

/// 
/// Gets the total programs to break them up
///
fn get_total_programs() -> u64 {
    1000
}

///
/// Gets all the shows from COVE
///
fn get_programs<'a>(start_index: u64) -> Vec<Program<'a>> {
    vec![
        Program {program_id: "1", data: "1"},
        Program {program_id: "2", data: "2"},
        Program {program_id: "3", data: "3"}
    ]
}

///
/// Gets the total videos for a program so they can be chunked
///
fn get_video_count_for_program<'a>(program: &Program) -> u64 {
    1000
}

///
/// Gets all videos from COVE for a program, 200 at a time
///
fn get_videos<'a>(j: u64, program: &Arc<Program>) -> Vec<Video<'a>> {
     vec![
        Video {tp_media_object_id: "1", data: "1"},
        Video {tp_media_object_id: "2", data: "2"},
        Video {tp_media_object_id: "3", data: "3"}
    ]
}