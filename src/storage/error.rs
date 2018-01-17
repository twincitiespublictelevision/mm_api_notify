use std::result::Result;
use std::fmt;

use error::IngestError;

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug)]
pub enum StoreError {
    UriParseError(String),
    InvalidItemError(IngestError),
    StorageFindError,
    StorageWriteError,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StoreError::UriParseError(ref conn) => {
                write!(f, "Failed to parse connection string {:?}", conn)
            }
            StoreError::InvalidItemError(ref err) => err.fmt(f),
            StoreError::StorageFindError => write!(f, "Failed to return a document"),
            StoreError::StorageWriteError => write!(f, "Failed to write to storage"),
        }
    }
}
