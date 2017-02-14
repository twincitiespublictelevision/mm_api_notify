extern crate bson;
extern crate mongodb;

use bson::Bson;
use mongodb::{Client, ThreadedClient};
use mongodb::db::{Database, ThreadedDatabase};
use mongodb::coll::options::{FindOptions, FindOneAndUpdateOptions};

use config::DBConfig;
use objects::{Object, utils};
use storage::error::{StoreError, StoreResult};

pub struct Store {
    db: Database,
}

impl Store {
    pub fn new(config: &DBConfig) -> StoreResult<Store> {

        // Set up the database connection.
        Client::connect(config.host.as_str(), config.port)
            .or_else(|_| Err(StoreError::InitializationError))
            .and_then(|client| Ok(client.db(config.name.as_str())))
            .and_then(|db| {
                db.auth(config.username.as_str(), config.password.as_str())
                    .or_else(|_| Err(StoreError::AuthorizationError))
                    .and_then(|_| Ok(Store { db: db }))
            })
    }

    pub fn get(&self, id: &str, obj_type: &str) -> Option<Object> {
        let query = doc!{
            "_id" => id
        };

        let coll = self.db.collection(obj_type);

        coll.find(Some(query), None).ok().and_then(|mut cursor| match cursor.next() {
            Some(Ok(doc)) => {
                bson::from_bson(utils::map_bson_dates_to_string(Bson::Document(doc))).ok()
            }
            _ => None,
        })
    }

    pub fn put(&self, object: &Object) -> StoreResult<Object> {
        object.as_document().map_err(StoreError::InvalidObjectError).and_then(|doc| {
            let coll = self.db.collection(object.object_type.as_str());
            let id = object.id.as_str();

            let filter = doc! {
                "_id" => id
            };

            let mut options = FindOneAndUpdateOptions::new();
            options.upsert = true;

            coll.find_one_and_replace(filter, doc, Some(options))
                .map_err(StoreError::StorageWriteError)
                .and_then(|opt| match opt {
                    Some(doc) => {
                        Object::from_bson(Bson::Document(doc))
                            .map_err(StoreError::InvalidObjectError)
                    }
                    None => Err(StoreError::StorageFindError),
                })
        })
    }

    pub fn updated_at(&self) -> Option<i64> {
        let collections = vec!["asset", "episode", "season", "show", "special"];

        collections.iter()
            .filter_map(|coll_name| {
                let coll = self.db.collection(coll_name);
                let mut query_options = FindOptions::new().with_limit(1);
                query_options.sort = Some(doc! {
                    "attributes.updated_at" => (-1)
                });

                coll.find(None, Some(query_options))
                    .ok()
                    .and_then(|mut cursor| cursor.next())
                    .and_then(|result| result.ok())
                    .and_then(|mut document| {
                        document.remove("attributes")
                            .and_then(|attributes| match attributes {
                                bson::Bson::Document(mut attr) => {
                                    match attr.remove("updated_at") {
                                        Some(bson::Bson::UtcDatetime(datetime)) => {
                                            Some(datetime.timestamp())
                                        }
                                        _ => None,
                                    }
                                }
                                _ => None,
                            })
                    })
            })
            .fold(None, |max, cur| match max {
                Some(val) => Some(::std::cmp::max(val, cur)),
                None => Some(cur),
            })
    }
}
