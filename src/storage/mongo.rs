extern crate bson;
extern crate mongodb;

use bson::Bson;
use mongodb::{Client, ThreadedClient};
use mongodb::coll::options::{FindOptions, FindOneAndUpdateOptions};
use mongodb::common::WriteConcern;
use mongodb::db::{Database, ThreadedDatabase};

use config::DBConfig;
use objects::{Object, utils};
use storage::error::{StoreError, StoreResult};
use storage::storage::{Storage, StorageStatus};

pub struct MongoStore {
    db: Database,
}

impl Storage<Object> for MongoStore {
    fn new(config: Option<&DBConfig>) -> StoreResult<MongoStore> {
        config
            .ok_or(StoreError::ConfigError)
            .and_then(|conf| {

                // Set up the database connection.
                Client::connect(conf.host.as_str(), conf.port)
                    .or_else(|_| Err(StoreError::InitializationError))
                    .and_then(|client| Ok(client.db(conf.name.as_str())))
                    .and_then(|db| {
                                  db.auth(conf.username.as_str(), conf.password.as_str())
                                      .or_else(|_| Err(StoreError::AuthorizationError))
                                      .and_then(|_| Ok(MongoStore { db: db }))
                              })
            })
    }

    fn get(&self, id: &str, obj_type: &str) -> Option<StoreResult<Object>> {
        let query = doc!{
            "_id" => id
        };

        let coll = self.db.collection(obj_type);

        coll.find(Some(query), None)
            .ok()
            .and_then(|mut cursor| {
                cursor
                    .next()
                    .map(|res| {
                        res.or_else(|err| {
                            error!("Failed to get {} from the Mongo store due to {}", id, err);
                            Err(StoreError::StorageFindError)
                        })
                        .and_then(|doc| {
                            Object::from_bson(utils::map_bson_dates_to_string(Bson::Document(doc)))
                                .map_err(StoreError::InvalidItemError)
                        })
                    })
            })
    }

    fn put(&self, item: &Object) -> StoreResult<StorageStatus> {
        item.as_document()
            .map_err(StoreError::InvalidItemError)
            .and_then(|doc| {
                let coll = self.db.collection(item.object_type.as_str());
                let id = item.id.as_str();

                let filter = doc! {
                "_id" => id
            };

                let mut options = FindOneAndUpdateOptions::new();
                options.upsert = Some(true);
                options.write_concern = Some(WriteConcern {
                                                 w: 1,
                                                 w_timeout: 60000,
                                                 j: true,
                                                 fsync: true,
                                             });

                coll.find_one_and_replace(filter, doc, Some(options))
                    .map_err(|e| {
                        print!("{:?}", e);
                        StoreError::StorageWriteError
                    })
                    .and_then(|opt| {
                                  Ok(match opt {
                                         Some(_) => StorageStatus::Available,
                                         None => StorageStatus::NotAvailable,
                                     })
                              })
            })
    }

    fn updated_at(&self) -> Option<i64> {
        let collections = vec!["asset", "episode", "season", "show", "special"];

        collections
            .iter()
            .filter_map(|coll_name| {
                let coll = self.db.collection(coll_name);
                let mut query_options = FindOptions::new();
                query_options.limit = Some(1);
                query_options.sort = Some(doc! {
                    "attributes.updated_at" => (-1)
                });

                coll.find(None, Some(query_options))
                    .ok()
                    .and_then(|mut cursor| cursor.next())
                    .and_then(|result| result.ok())
                    .and_then(|mut document| {
                        document
                            .remove("attributes")
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
