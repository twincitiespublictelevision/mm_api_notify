#[macro_use]
extern crate mysql;

pub mod video;
pub mod wp;

///
/// Does the actual ingestion
///
pub fn ingest() {
    video::ingest();
}

///
/// Recovers on termination
///
pub fn terminate() {
    video::terminate();
}