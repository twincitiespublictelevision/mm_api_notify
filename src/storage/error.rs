use std::result::Result;
use std::fmt;

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug)]
pub enum StoreError {
    InitializationError,
    AuthorizationError,
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
        }
    }
}
