use std::thread::JoinHandle;

/// 
/// Implements a worker pool for threads
///
pub struct WorkerPool { 
    pub join_handles: Vec<JoinHandle<()>>,
    pub max_handles: usize
}

impl WorkerPool {

    ///
    /// Constructor
    ///
    pub fn new(max_handles: usize) -> WorkerPool {
        WorkerPool {
            join_handles: vec![],
            max_handles: max_handles
        }
    } 

    ///
    /// Waits for handle to free up
    ///
    pub fn wait_for_a_spot(&mut self) {
        while self.join_handles.len() >= self.max_handles {
            let join_handle = self.join_handles.pop().unwrap();
            join_handle.join().expect("Unable to join thread waiting for a spot.");
        }
    }

    ///
    /// Waits for children to finish
    ///
    pub fn wait_for_children(&mut self) {
        for handle in self.join_handles.drain(..) {
            handle.join().expect("Unable to join thread in waiting for children to finish.");
        }
    }
}