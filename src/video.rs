extern crate mongodb;

use super::wp;

///
/// Holds a video object
///
pub struct Video<'a> {
    tp_media_id: &'a str
}

///
/// Gets all videos from COVE for a show.
///
pub fn get_videos<'a>(show: &wp::WPShow) -> Vec<Video<'a>> {
    println!("Getting videos for show: {}", show.id);

    return vec![
        Video {tp_media_id: "1"},
        Video {tp_media_id: "2"},
        Video {tp_media_id: "3"},
    ];
}