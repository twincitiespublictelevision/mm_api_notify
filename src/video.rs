extern crate bson;
extern crate mongodb;

use super::wp as wp;
use std::thread as thread;
use std::thread::JoinHandle as JoinHandle;
use mysql as my;

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
pub fn terminate() {
    println!("Waiting for all threads to finish");
}

///
/// Gets all programs from COVE
///
pub fn get_videos<'a>(show: wp::WPShow) -> Vec<Video<'a>> {
    return vec![];
}