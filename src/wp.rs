use mysql as my;

pub struct WPShow<'a> {
    pub id: u32,
    pub video_id: &'a str
}

pub struct WP {
    pool: my::Pool
}

impl WP {

    // Gets the pool ready
    pub fn new() -> Self {
        WP {
            pool: my::Pool::new("mysql://root:root@localhost:3306").unwrap()
        }
    }

    ///
    /// Gets all shows from the WordPress database
    ///
    pub fn get_shows<'a>(&self) -> Vec<WPShow<'a>> {
        println!("Getting shows...");

        return vec![
            WPShow { id: 1, video_id: "1"},
            WPShow { id: 2, video_id: "2"},
            WPShow { id: 3, video_id: "3"}
        ];
    }
}