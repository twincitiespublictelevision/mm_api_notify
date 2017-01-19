// use std::thread::JoinHandle;
// use std::collections::VecDeque;
//
// ///
// /// Implements a worker pool for threads
// ///
// pub struct WorkerPool {
//     pub join_handles: VecDeque<JoinHandle<()>>,
//     pub max_handles: usize,
// }
//
// impl WorkerPool {
//     ///
//     /// Constructor
//     ///
//     pub fn new(max_handles: usize) -> WorkerPool {
//         WorkerPool {
//             join_handles: VecDeque::new(),
//             max_handles: max_handles,
//         }
//     }
//
//     ///
//     /// Adds a worker
//     ///
//     pub fn add_worker(&mut self, new_handle: JoinHandle<()>) {
//         while self.join_handles.len() >= self.max_handles {
//             let join_handle = self.join_handles.pop_front().unwrap();
//             join_handle.join().expect("Unable to join thread waiting for a spot.");
//         }
//
//         self.join_handles.push_back(new_handle);
//     }
//
//     ///
//     /// Waits for workers to finish
//     ///
//     pub fn wait_for_workers(&mut self) {
//         for handle in self.join_handles.drain(..) {
//             handle.join().expect("Unable to join thread in waiting for workers to finish.");
//         }
//     }
// }
