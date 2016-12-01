extern crate rustc_serialize;

use self::rustc_serialize::json::Json;
use error::IngestError;
use objects::season;

pub struct Show {
    data: Json,
}

impl Show {
    pub fn new(data: Json) -> Show {
        Show { data: data }
    }
}

impl season::Seasons for Show {
    fn seasons(&self) -> Vec<season::Season> {
        let attributes = match self.data.find("attributes") {
            Some(value) => value,
            None => panic!("Failed to find attributes"),
        };

        match attributes.find("seasons") {
            Some(objects) => {
                objects.as_array()
                    .unwrap()
                    .into_iter()
                    .map(|object| season::Season::new(object.clone()))
                    .collect()
            }
            None => panic!("Failed to find seasons"),
        }
    }
}
