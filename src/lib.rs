extern crate libc;

use std::thread;
use std::thread::JoinHandle;
use std::os::unix::thread::JoinHandleExt;
use libc::pthread_join;
use libc::c_void;
use std::ptr;

pub struct WorkerPool {
    pub join_handles: Vec<JoinHandle<()>>
}

impl WorkerPool {
  
    ///
    /// Does the actual ingestion
    ///
    pub fn ingest(&mut self) {

        // Use 9 threads for an example.
        for i in 0..10 {
            self.join_handles.push(
                thread::spawn(move || {

                    // Get the videos
                    println!("Getting videos for thread {}", i);
                })
            );
        }
    }

    ///
    /// Joins all threads
    ///
    pub fn terminate(&mut self) {
        for handle in &self.join_handles {
            unsafe {
                let state_ptr: *mut *mut c_void = *ptr::null();
                pthread_join(handle.as_pthread_t(), state_ptr);
            }
        }

        self.join_handles = vec![];
    }
}