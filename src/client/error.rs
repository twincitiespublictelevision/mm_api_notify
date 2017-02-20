extern crate mm_client;

use self::mm_client::MMCError;

use std::error::Error;
use std::fmt;
use std::result::Result;

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug)]
pub enum ClientError {
    ConfigError,
    InitializationError,
    API(MMCError),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ClientError::ConfigError => write!(f, "Invalid config was supplied"),
            ClientError::InitializationError => write!(f, "Failed to initialize the API client"),
            _ => write!(f, ""),
        }
    }
}

impl Error for ClientError {
    fn description(&self) -> &str {
        match *self {
            ClientError::API(ref err) => err.description(),
            _ => "",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ClientError::API(ref err) => err.cause(),
            _ => None,
        }
    }
}
