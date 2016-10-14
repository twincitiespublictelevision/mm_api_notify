extern crate bson;
extern crate mongodb;

use super::wp as wp;
use std::thread as thread;
use std::thread::JoinHandle as JoinHandle;
use mysql as my;
use std::time::Duration;

pub struct Video<'a> {
    tp_media_id: &'a str
}

///
/// Ingests video
///
pub fn ingest() -> Vec<JoinHandle<()>> {
    
    // Get all the shows from WordPress.
    let pool = wp::initialize_pool();
    let shows = wp::get_shows(pool);
    
    // Spawn threads to get videos.
    return shows.into_iter().map(|show| thread::spawn(move || {
        get_videos(show);
    })).collect::<Vec<_>>();
}

///
/// Recovers on termination
///
pub fn terminate(join_handles: Vec<JoinHandle<()>>) {
    println!("Terminating...");
    join_handles.into_iter().map(|join_handle| {
        match join_handle.thread().name() {
            Some(name) => println!("Joining thread: {}", name),
            None => {}
        }

        join_handle.join();
    });
}

///
/// Gets all programs from COVE
///
pub fn get_videos<'a>(show: wp::WPShow) -> Vec<Video<'a>> {
    println!("Getting videos for show: {}", show.id);

    return vec![
        Video {tp_media_id: "1"},
        Video {tp_media_id: "2"},
        Video {tp_media_id: "3"},
    ];
}