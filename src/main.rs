#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate video_ingest;

use video_ingest::video;

use chan_signal::Signal;
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

///
/// Starts processing
/// 
fn main() {
    
    // Signal gets a value when the OS sent a INT or TERM signal.
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);

    // Used to signal the main thread to stop.
    let please_stop = Arc::new(AtomicBool::new(false));

    // Clone for the running thread.
    let threads_please_stop = please_stop.clone();

    let join_handle = thread::spawn(move || run(threads_please_stop));

    // Wait for a signal or for work to be done.
    chan_select! {
        signal.recv() -> signal => {
            println!("Received signal: {:?}", signal);
            please_stop.store(true, Ordering::SeqCst);
        }
    }

    join_handle.join().expect("Unable to join main thread.");
    println!("Program complete.");
}

// 
/// Runs the main thread.
///
fn run(please_stop: Arc<AtomicBool>)  {
    let mut first_time = true;

    while !please_stop.load(Ordering::SeqCst) {
        video::ingest(first_time);
        first_time = false;
    }
}