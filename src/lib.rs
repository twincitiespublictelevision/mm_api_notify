#[macro_use]
extern crate mysql;

pub mod video;
pub mod wp;

pub fn ingest() {
  video::ingest();
}