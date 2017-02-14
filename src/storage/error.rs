extern crate mongodb;

use std::result::Result;
use std::fmt;

use error::IngestError;

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug)]
pub enum StoreError {
    ConfigError,
    InitializationError,
    AuthorizationError,
    InvalidItemError(IngestError),
    StorageFindError,
    StorageWriteError,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StoreError::ConfigError => write!(f, "Invalid config was supplied"),
            StoreError::InitializationError => {
                write!(f, "Failed to initialize the storage mechanism")
            }
            StoreError::AuthorizationError => {
                write!(f,
                       "Failed to initialize the authorization against the storage mechanism")
            }
            StoreError::InvalidItemError(ref err) => err.fmt(f),
            StoreError::StorageFindError => write!(f, "Failed to return a document"),
            StoreError::StorageWriteError => write!(f, "Failed to write to storage"),
        }
    }
}
