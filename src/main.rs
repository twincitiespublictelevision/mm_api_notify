#[macro_use]
extern crate chan;
extern crate chan_signal;

extern crate video_ingest;

use chan_signal::Signal;
use video_ingest::WorkerPool;

///
/// Starts processing
/// 
fn main() {

    // Signal gets a value when the OS sent a INT or TERM signal.
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    
    // When our work is complete, send a sentinel value on `sdone`.
    let (send, recv) = chan::sync(0);

    // Start the workers
    let mut worker_pool = WorkerPool { join_handles: vec![] };

    // Run work.
    worker_pool = run(send, worker_pool);
    
    // Wait for a signal or for work to be done.
    chan_select! {
        signal.recv() -> signal => {
            println!("received signal: {:?}", signal);
            worker_pool.terminate();
        },
        recv.recv() => {
            println!("Program completed normally.");
        }
    }
}

///
/// Runs the ingest
///
/// We needed to return worker_pool because of borrowing.  TODO: Find a way around this.
///
fn run(send: chan::Sender<()>, mut worker_pool: WorkerPool) -> WorkerPool {
    worker_pool.ingest();

    return worker_pool;
}