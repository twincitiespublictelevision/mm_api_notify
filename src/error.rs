extern crate bson;
extern crate mm_client;
extern crate rayon;
extern crate serde_json;

use self::bson::{DecoderError, EncoderError};
use self::rayon::InitError;
use self::serde_json::error::Error as ParserError;

use std::error::Error;
use std::result::Result;
use std::fmt;

use client::ClientError;

pub type IngestResult<T> = Result<T, IngestError>;

#[derive(Debug)]
pub enum IngestError {
    InvalidConfig,
    ThreadPool(InitError),
    Client(ClientError),
    Parse(ParserError),
    Serialize(EncoderError),
    Deserialize(DecoderError),
    InvalidDocumentDataError,
    InvalidObjDataError,
    InvalidRefDataError,
}

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IngestError::InvalidConfig => {
                write!(f,
                       "Supplied config.toml could not be understood. Try checking for a \
                        misspelled or missing property.")
            }
            IngestError::ThreadPool(ref err) => err.fmt(f),
            IngestError::Client(ref err) => err.fmt(f),
            IngestError::Parse(ref err) => err.fmt(f),
            IngestError::Serialize(ref err) => err.fmt(f),
            IngestError::Deserialize(ref err) => err.fmt(f),
            _ => write!(f, ""),
        }
    }
}

impl Error for IngestError {
    fn description(&self) -> &str {
        match *self {
            IngestError::InvalidConfig => {
                "Supplied config.toml could not be understood. Try checking for a misspelled or \
                 missing property."
            }
            IngestError::ThreadPool(ref err) => err.description(),
            IngestError::Client(ref err) => err.description(),
            IngestError::Parse(ref err) => err.description(),
            IngestError::Serialize(ref err) => err.description(),
            IngestError::Deserialize(ref err) => err.description(),
            _ => "",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            IngestError::ThreadPool(ref err) => Some(err),
            IngestError::Client(ref err) => Some(err),
            IngestError::Parse(ref err) => Some(err),
            IngestError::Serialize(ref err) => Some(err),
            IngestError::Deserialize(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<InitError> for IngestError {
    fn from(err: InitError) -> IngestError {
        IngestError::ThreadPool(err)
    }
}

impl From<ClientError> for IngestError {
    fn from(err: ClientError) -> IngestError {
        IngestError::Client(err)
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

impl From<DecoderError> for IngestError {
    fn from(err: DecoderError) -> IngestError {
        IngestError::Deserialize(err)
    }
}
