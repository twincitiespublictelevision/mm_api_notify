use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use super::wp::WPShow;
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
    pub fn ingest(&mut self, shows: &Vec<WPShow>) {
        self.join_handles.extend(
            shows.iter().map(|show| {
                thread::spawn(move || {
                    video::get_videos(show);
                })
            })
        );
    }

    ///
    /// Joins all threads
    ///
    pub fn terminate(&mut self) {
        for handle in self.join_handles.drain(..) {
            println!("Joining thread...");
            handle.join().expect("Unable to join thread");
        }
    }
}