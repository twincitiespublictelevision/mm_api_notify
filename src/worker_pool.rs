use std::thread;
use std::thread::JoinHandle;

use super::wp::WP;
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
        println!("Ingesting...");

        for show in wp.get_shows() {
            println!("Show: {}", show.id);

            self.join_handles.push( 
                thread::spawn(move || {
                    println!("Launching thread to get videos...");
                    
                    for video in video::get_videos(show) {
                        println!("Received video {}: ", video.tp_media_id);
                    }
                })
            )
        }
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