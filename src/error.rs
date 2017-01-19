extern crate core_data_client;
extern crate serde_json;
extern crate bson;

use std::error::Error;
use std::result::Result;
use std::fmt;
use self::serde_json::error::Error as ParserError;
use self::bson::EncoderError;
use self::core_data_client::CDCError;

pub type IngestResult<T> = Result<T, IngestError>;

#[derive(Debug)]
pub enum IngestError {
    API(CDCError),
    Parse(ParserError),
    Serialize(EncoderError),
    InvalidDocumentDataError,
    InvalidObjDataError,
    InvalidRefDataError,
}

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IngestError::API(ref err) => err.fmt(f),
            IngestError::Parse(ref err) => err.fmt(f),
            IngestError::Serialize(ref err) => err.fmt(f),
            IngestError::InvalidDocumentDataError => write!(f, ""),
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
            IngestError::Serialize(ref err) => err.description(),
            IngestError::InvalidDocumentDataError => "",
            IngestError::InvalidObjDataError => "",
            IngestError::InvalidRefDataError => "",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            IngestError::API(ref err) => Some(err),
            IngestError::Parse(ref err) => Some(err),
            IngestError::Serialize(ref err) => Some(err),
            IngestError::InvalidDocumentDataError => None,
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

impl From<EncoderError> for IngestError {
    fn from(err: EncoderError) -> IngestError {
        IngestError::Serialize(err)
    }
}
