#[macro_use]
extern crate mysql;

pub mod video;
pub mod wp;

use std::thread::JoinHandle;
use std::thread::Thread;

pub struct WorkerPool {
    pub join_handles: Vec<JoinHandle<()>>
}

impl WorkerPool {
  
    ///
    /// Does the actual ingestion
    ///
    pub fn ingest(&mut self) {
        let handles = video::ingest();

        handles.into_iter().map(|handle| self.join_handles.push(handle));
    }

    ///
    /// Recovers on termination
    ///
    pub fn terminate(self) {
        video::terminate(self.join_handles);
    }
}