extern crate bson;
extern crate mongo_driver;

use bson::Bson;
use mongo_driver::CommandAndFindOptions;
use mongo_driver::client::{ClientPool, Uri};
use mongo_driver::collection::UpdateOptions;
use mongo_driver::flags::UpdateFlag;

use config::DBConfig;
use objects::{utils, Object};
use storage::error::{StoreError, StoreResult};
use storage::storage::{Storage, StorageStatus};

pub struct MongoStore {
    config: DBConfig,
    pool: ClientPool,
}

impl MongoStore {
    pub fn new(config: &DBConfig) -> StoreResult<MongoStore> {
        let conn = MongoStore::conn(
            config.username.as_str(),
            config.password.as_str(),
            config.host.as_str(),
            config.port,
        );
        let uri = Uri::new(conn.clone()).ok_or(StoreError::UriParseError(conn))?;

        let pool = ClientPool::new(uri, None);

        Ok(MongoStore {
            config: config.clone(),
            pool: pool,
        })
    }

    fn conn(user: &str, pass: &str, host: &str, port: u16) -> String {
        format!("mongodb://{}:{}@{}:{}", user, pass, host, port)
    }
}

impl Storage<Object> for MongoStore {
    fn get(&self, id: &str, obj_type: &str) -> Option<StoreResult<Object>> {
        let query = doc!{
            "_id" => id
        };

        let client = self.pool.pop();
        let coll = client.get_collection(self.config.name.as_str(), obj_type);

        let res = coll.find(&query, None).ok().and_then(|mut cursor| {
            cursor.next().map(|res| {
                res.or_else(|err| {
                    error!("Failed to get {} from the Mongo store due to {}", id, err);
                    Err(StoreError::StorageFindError)
                }).and_then(|doc| {
                    Object::from_bson(utils::map_bson_dates_to_string(Bson::Document(doc)))
                        .map_err(StoreError::InvalidItemError)
                })
            })
        });

        res
    }

    fn put(&self, item: &Object) -> StoreResult<StorageStatus> {
        item.as_document()
            .map_err(StoreError::InvalidItemError)
            .and_then(|doc| {
                let client = self.pool.pop();
                let coll =
                    client.get_collection(self.config.name.as_str(), item.object_type.as_str());

                let id = item.id.as_str();

                let filter = doc! {
                    "_id" => id
                };

                let mut opts = UpdateOptions::default();
                opts.update_flags.add(UpdateFlag::Upsert);

                coll.update(&filter, &doc, Some(&opts))
                    .map(|_| StorageStatus::Available)
                    .or_else(|_| Err(StoreError::StorageWriteError))
            })
    }

    fn updated_at(&self) -> Option<i64> {
        let collections = vec!["asset", "episode", "season", "show", "special"];
        let mut opts = CommandAndFindOptions::default();
        opts.limit = 1;
        let client = self.pool.pop();

        collections
            .iter()
            .filter_map(|coll_name| {
                let coll = client.get_collection(self.config.name.as_str(), *coll_name);

                let query = doc! {
                    "$query" => {},
                    "$orderby" => {
                        "attributes.updated_at" => -1
                    }
                };

                let res = coll.find(&query, Some(&opts))
                    .ok()
                    .and_then(|mut cursor| cursor.next())
                    .and_then(|result| result.ok())
                    .and_then(|mut document| {
                        document
                            .remove("attributes")
                            .and_then(|attributes| match attributes {
                                bson::Bson::Document(mut attr) => match attr.remove("updated_at") {
                                    Some(bson::Bson::UtcDatetime(datetime)) => {
                                        Some(datetime.timestamp())
                                    }
                                    _ => None,
                                },
                                _ => None,
                            })
                    });

                res
            })
            .fold(None, |max, cur| match max {
                Some(val) => Some(::std::cmp::max(val, cur)),
                None => Some(cur),
            })
    }
}
