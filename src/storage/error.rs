extern crate mongodb;

use mongodb::error::Error as DBError;

use std::result::Result;
use std::fmt;

use error::IngestError;

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug)]
pub enum StoreError {
    InitializationError,
    AuthorizationError,
    InvalidObjectError(IngestError),
    StorageFindError,
    StorageWriteError(DBError),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StoreError::InitializationError => {
                write!(f, "Failed to initialize the storage mechanism")
            }
            StoreError::AuthorizationError => {
                write!(f,
                       "Failed to initialize the authorization against the storage mechanism")
            }
            StoreError::InvalidObjectError(ref err) => err.fmt(f),
            StoreError::StorageFindError => write!(f, "Failed to return a document"),
            StoreError::StorageWriteError(ref err) => err.fmt(f),
        }
    }
}
