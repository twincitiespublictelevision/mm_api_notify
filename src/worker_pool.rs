use std::thread;
use std::thread::JoinHandle;
use std::sync::Arc;

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
    pub fn ingest(&mut self) {
        let mut i = Arc::new(0);

        for i in (0..video::get_total_programs()).step_by(200) {
            let i = i.clone();

            self.join_handles.push(
                thread::spawn(move || {
                    for program in video::get_programs(i) {
                        program.save();
                        let mut j = Arc::new(0);
                        let program_to_share = Arc::new(&program);
                        
                        for j in (0..video::get_video_count_for_program(&program)).step_by(200) {
                            let j = j.clone();
                            let program_to_share = program_to_share.clone();
                            
                            self.join_handles.push( 
                                thread::spawn(move || {
                                    for video in video::get_videos(&program_to_share, j) {
                                        video.save();
                                    }
                                })
                            );
                        }
                    }
                })
            );
        }
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