extern crate rustc_serialize;

use self::rustc_serialize::json::Json;
use error::IngestError;
use std::fmt;

pub struct Season {
    data: Json,
}

impl Season {
    pub fn new(data: Json) -> Season {
        Season { data: data }
    }
}

impl fmt::Display for Season {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.data.fmt(f)
    }
}

pub trait Seasons {
    fn seasons(&self) -> Vec<Season>;
}
