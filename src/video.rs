extern crate mongodb;
extern crate curl;

///
/// Holds a video object
///
pub struct Video<'a> {
    pub data: &'a str,
    pub shows: Vec<Show<'a>>
} 

///
/// Holds a show object
///
pub struct Show<'a> {
    pub data: &'a str,
    pub videos: Vec<Video<'a>>
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
/// Gets all the shows from COVE
///
pub fn get_shows<'a>() -> Vec<Show<'a>> {
    return vec![
        Show {data: "1", videos: vec![]}
    ];
}

///
/// Gets all videos from COVE for a show.
///
pub fn get_videos<'a>(show: Show) -> Vec<Video<'a>> {
    return vec![
        Video {data: "1", shows: vec![]},
        Video {data: "2", shows: vec![]},
        Video {data: "3", shows: vec![]},
    ];
}