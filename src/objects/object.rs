extern crate rustc_serialize;

use self::rustc_serialize::json::Json;
use std::fmt;
use objects::reference::Ref;

pub struct Object {
    pub data: Json,
}

impl Object {
    pub fn new(data: &Json) -> Object {
        Object { data: data.clone() }
    }

    // TODO: Handle the rest of the references

    pub fn assets(&self) -> Option<Vec<Ref>> {
        self.references("assets")
    }

    pub fn collection(&self) -> Option<Ref> {
        self.reference("collection")
    }

    pub fn collections(&self) -> Option<Vec<Ref>> {
        self.references("collections")
    }

    pub fn episode(&self) -> Option<Ref> {
        self.reference("episode")
    }

    pub fn episodes(&self) -> Option<Vec<Ref>> {
        self.references("episodes")
    }

    pub fn franchise(&self) -> Option<Ref> {
        self.reference("franchise")
    }

    pub fn season(&self) -> Option<Ref> {
        self.reference("season")
    }

    pub fn seasons(&self) -> Option<Vec<Ref>> {
        self.references("seasons")
    }

    pub fn show(&self) -> Option<Ref> {
        self.reference("show")
    }

    pub fn shows(&self) -> Option<Vec<Ref>> {
        self.references("shows")
    }

    pub fn special(&self) -> Option<Ref> {
        self.reference("special")
    }

    pub fn specials(&self) -> Option<Vec<Ref>> {
        self.references("specials")
    }

    pub fn attributes(&self) -> Option<&Json> {
        self.data.find("attributes")
    }

    pub fn value(&self, property: &str) -> Option<&str> {
        self.data.find(property).map_or(None, |type_value| type_value.as_string())
    }

    fn reference(&self, ref_name: &str) -> Option<Ref> {

        self.attributes().map_or(None, |attributes| {
            attributes.find(ref_name).map_or(None, |object| Some(Ref::new(object)))
        })
    }

    fn references(&self, ref_name: &str) -> Option<Vec<Ref>> {

        self.attributes()
            .map_or(None, |attributes| {
                attributes.find(ref_name)
                    .map_or(None, |objects| {
                        objects.as_array().map_or(None, |array| {
                            Some(array.into_iter().map(|object| Ref::new(object)).collect())
                        })
                    })
            })

    }
}


impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.data.fmt(f)
    }
}
