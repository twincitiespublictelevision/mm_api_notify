extern crate bson;
extern crate mongodb;

use super::wp as wp;

///
/// Ingests video
///
pub fn ingest() {
    let mut shows:Vec<wp::WPShow> = vec![];
    
    {
        let mut shows_ptr = & mut shows;
        let pool = wp::initialize_pool();

        wp::get_shows(pool, shows_ptr);
    }
    
    println!("{}{}", shows[0].id, shows[0].video_id);
    println!("Ingesting!");
}