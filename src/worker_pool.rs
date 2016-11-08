use std::thread::JoinHandle;

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
    pub fn new() -> WorkerPool {
        WorkerPool {
            join_handles: vec![]
        }
    } 

    ///
    /// Waits for children to finish
    ///
    pub fn wait_for_children(&mut self) {
        for handle in self.join_handles.drain(..) {
            handle.join().expect("Unable to join thread.");
        }
    }
}