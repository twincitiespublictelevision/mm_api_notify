extern crate mongodb;
extern crate curl;

///
/// Holds a video object
///
pub struct Video<'a> {
    pub tp_media_object_id: &'a str,
    pub data: &'a str
}

impl Video<'a> {

    ///
    /// Does a Mongo upsert off the tp_media_object_id
    ///
    pub fn save(&self) {

    }

    ///
    /// Deletes all records where not in the list passed in
    pub fn delete_where_not_in(ids_to_save: Vec<&str>) {

    }
} 

///
/// Holds a show object
///
pub struct Program<'a> {
    pub program_id: &'a str,
    pub data: &'a str
}

impl Program<'a> {

    ///
    /// Does a Mongo upsert off the tp_media_object_id
    ///
    pub fn save(&self) {

    }

    ///
    /// Deletes all records where not in the list passed in
    pub fn delete_where_not_in(ids_to_save: Vec<&str>) {

    }
}

///
/// Makes an API call
///
fn video_api<'a>(endpoint: &str, filters: Vec<[&str; 2]>, fields: Vec<&str>) -> &'a str {
    let mut url = format!("http://api.pbs.org/cove/v1/{}", endpoint);

    for filter in filters {
    url = format!("{}&{}={}", url, filter[0], filter[1]);
    }

    return "";
}

/// 
/// Gets the total programs to break them up
///
pub fn get_total_programs() -> u64 {
    1000
}

///
/// Gets all the shows from COVE
///
pub fn get_programs<'a>(start_index: u64) -> Vec<Program<'a>> {
    vec![
        Program {program_id: "1", data: "1"},
        Program {program_id: "2", data: "2"},
        Program {program_id: "3", data: "3"}
    ]
}

///
/// Gets the total videos for a program so they can be chunked
///
pub fn get_video_count_for_program<'a>(program: &Program) -> u64 {
    1000
}

///
/// Gets all videos from COVE for a program, 200 at a time
///
pub fn get_videos<'a>(program: &Program, start_index: u64) -> Vec<Video<'a>> {
    vec![
        Video {tp_media_object_id: "1", data: "1"},
        Video {tp_media_object_id: "2", data: "2"},
        Video {tp_media_object_id: "3", data: "3"}
    ]
}