#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate video_ingest;

use chan_signal::Signal;
use video_ingest::worker_pool::WorkerPool;
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use wp::WP;

///
/// Starts processing
/// 
fn main() {
    let worker_pool = WorkerPool { join_handles: vec![] };
    
    // Signal gets a value when the OS sent a INT or TERM signal.
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);

    // Used to signal the main thread to stop.
    let please_stop = Arc::new(AtomicBool::new(false));

    // Clone for the running thread.
    let threads_please_stop = please_stop.clone();

    // One WordPress instance.
    let wp = WP::new();

    // Run work.
    let runner = thread::spawn(|| run(threads_please_stop, worker_pool, wp));

    // Wait for a signal or for work to be done.
    chan_select! {
        signal.recv() -> signal => {
            println!("Received signal: {:?}", signal);
            please_stop.store(true, Ordering::SeqCst);
        }
    }

    runner.join().expect("Unable to join runner thread.");
    println!("Program complete.");
}

// 
/// Runs the main thread.
///
fn run(please_stop: Arc<AtomicBool>, mut worker_pool: WorkerPool, wp: WP)  {
    while !please_stop.load(Ordering::SeqCst) {
        let shows = wp.get_shows();

        worker_pool.ingest(shows);
        worker_pool.terminate();
    }
}