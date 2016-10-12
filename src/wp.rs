use mysql as my;

pub struct WPShow<'a> {
    pub id: u32,
    pub video_id: &'a str
}

pub fn initialize_pool() -> my::Pool {
    return my::Pool::new("mysql://root:root@localhost:3306").unwrap();
}

///
/// Gets all shows from the WordPress database
///
pub fn get_shows(pool: my::Pool, wp_shows: & mut Vec<WPShow>) {
    *wp_shows = vec![
         WPShow { id: 1, video_id: "1"}
     ];
}

#[cfg(test)]
mod test {

    #[test]
    pub fn test_get_shows() {
    }
}