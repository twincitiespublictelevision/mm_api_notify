extern crate rustc_serialize;
extern crate core_data_client;

use std::error::Error;
use std::result::Result;
use std::fmt;
use rustc_serialize::json::ParserError;
use self::core_data_client::CDCError;

pub type IngestResult<T> = Result<T, IngestError>;

#[derive(Debug)]
pub enum IngestError {
    API(CDCError),
    Parse(ParserError),
    InvalidObjDataError,
    InvalidRefDataError,
}

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IngestError::API(ref err) => err.fmt(f),
            IngestError::Parse(ref err) => err.fmt(f),
            IngestError::InvalidObjDataError => write!(f, ""),
            IngestError::InvalidRefDataError => write!(f, ""),
        }
    }
}

impl Error for IngestError {
    fn description(&self) -> &str {
        match *self {
            IngestError::API(ref err) => err.description(),
            IngestError::Parse(ref err) => err.description(),
            IngestError::InvalidObjDataError => "",
            IngestError::InvalidRefDataError => "",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            IngestError::API(ref err) => Some(err),
            IngestError::Parse(ref err) => Some(err),
            IngestError::InvalidObjDataError => None,
            IngestError::InvalidRefDataError => None,
        }
    }
}

impl From<CDCError> for IngestError {
    fn from(err: CDCError) -> IngestError {
        IngestError::API(err)
    }
}

impl From<ParserError> for IngestError {
    fn from(err: ParserError) -> IngestError {
        IngestError::Parse(err)
    }
}
