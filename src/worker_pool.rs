use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use super::video;

/// 
/// Implements a worker pool for threads
///
pub struct WorkerPool {
    pub join_handles: Vec<JoinHandle<()>>,
}

impl WorkerPool {

    ///
    /// Constructor
    ///
    pub fn new() -> Self {
        WorkerPool {
            join_handles: vec![],
        }
    }
  
    ///
    /// Does the actual ingestion
    ///
    pub fn ingest(&mut self, wp: &WP) {
        let programs_to_save:Vec<&str> = vec![];
        let videos_to_save:Vec<&str> = vec![];
        let shared_programs_to_save = Arc::new(Mutex::new(programs_to_save));
        let shared_videos_to_save = Arc::new(Mutex::new(videos_to_save));
        let total_programs = video::get_total_programs();
        let i = 0;
        let shared_i = Arc::new(i);
        let total_videos = HashMap::new();
        let shared_total_videos = Arc::new(Mutex::new(total_videos));
        let video_counters = HashMap::new();
        let shared_video_counters = Arc::new(Mutex::new(video_counters));

        while (i < total_programs) {
            self.join_handles.push(
                thread::spawn(move || {
                    for program in video::get_programs(shared_i) {
                        let shared_program = Arc::new(program);
                        program.save();
                        shared_programs_to_save.lock();
                        shared_programs_to_save.push(program.program_id);
                        shared_programs_to_save.unlock();

                        shared_total_videos.lock();
                        let total_videos = video::get_video_count_for_program(program);
                        shared_total_videos.insert(program.program_id, total_videos);

                        shared_video_counters.lock();
                        shared_video_counters.insert(program.program_id, 0);
                        shared_video_counters.unlock();

                        while (shared_video_counters.get(program.program_id) < shared_total_videos.get(program.program_id)) {
                            self.join_handles.push( 
                                thread::spawn(move || {
                                    for video in video::get_videos(shared_program.clone(), shared_video_counters.get(program.program_id)) {
                                        video.save();

                                        videos_to_save.lock();
                                        videos_to_save.push(video.tp_media_object_id);
                                        videos_to_save.unlock();
                                    }
                                });
                            );

                            shared_video_counters.lock();
                            shared_video_counters.set(program.program_id, shared_video_counters.get(program.program_id) + 200);
                            shared_video_counters.unlock();
                        }
                    }
                });
            );

            i += 200;
        }

        // Get rid of the old stuff.
        video::Program::delete_where_not_in(programs_to_save);
        video::Video::delete_where_not_in(videos_to_save);
    }

    ///
    /// Joins all threads
    ///
    pub fn terminate(&mut self) {
        for handle in self.join_handles.drain(..) {
            handle.join().expect("Unable to join thread.");
        }
    }
}