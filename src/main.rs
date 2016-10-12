#[macro_use]
extern crate chan;
extern crate chan_signal;

extern crate video_ingest;

use chan_signal::Signal;

///
/// Starts processing
/// 
fn main() {

    // Signal gets a value when the OS sent a INT or TERM signal.
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    
    // When our work is complete, send a sentinel value on `sdone`.
    let (sdone, rdone) = chan::sync(0);
    
    // Run work.
    ::std::thread::spawn(move || run(sdone));

    // Wait for a signal or for work to be done.
    chan_select! {
        signal.recv() -> signal => {
            println!("received signal: {:?}", signal);
            video_ingest::terminate();
        },
        rdone.recv() => {
            println!("Program completed normally.");
        }
    }
}

///
/// Runs the ingest
///
fn run(_sdone: chan::Sender<()>) {
    video_ingest::ingest();
}