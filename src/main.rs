#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate video_ingest;

use chan_signal::Signal;
use video_ingest::WorkerPool;
use std::thread;

///
/// Starts processing
/// 
fn main() {
    let mut worker_pool = WorkerPool { join_handles: vec![] };
    
    // Signal gets a value when the OS sent a INT or TERM signal.
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);

    // When our work is complete, send a sentinel value on `sdone`.
    let (sdone, rdone) = chan::sync(0);

    // Run work.
    thread::spawn(|| run(sdone, &mut worker_pool));

    // Wait for a signal or for work to be done.
    chan_select! {
        signal.recv() -> signal => {
            println!("received signal: {:?}", signal);
            worker_pool.terminate();
        },
        rdone.recv() => {
            println!("Program completed normally.");
        }
    }
}

fn run(sdone: chan::Sender<()>, worker_pool: &mut WorkerPool) {
    loop {
        worker_pool.ingest();
        worker_pool.terminate();
    }
}