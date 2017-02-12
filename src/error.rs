extern crate bson;
extern crate mm_client;
extern crate rayon;
extern crate serde_json;

use std::error::Error;
use std::result::Result;
use std::fmt;

use self::bson::EncoderError;
use self::mm_client::MMCError;
use self::rayon::InitError;
use self::serde_json::error::Error as ParserError;

pub type IngestResult<T> = Result<T, IngestError>;

#[derive(Debug)]
pub enum IngestError {
    InvalidConfig,
    ThreadPool(InitError),
    API(MMCError),
    Parse(ParserError),
    Serialize(EncoderError),
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
            IngestError::API(ref err) => err.fmt(f),
            IngestError::Parse(ref err) => err.fmt(f),
            IngestError::Serialize(ref err) => err.fmt(f),
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
            IngestError::API(ref err) => err.description(),
            IngestError::Parse(ref err) => err.description(),
            IngestError::Serialize(ref err) => err.description(),
            _ => "",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            IngestError::ThreadPool(ref err) => Some(err),
            IngestError::API(ref err) => Some(err),
            IngestError::Parse(ref err) => Some(err),
            IngestError::Serialize(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<InitError> for IngestError {
    fn from(err: InitError) -> IngestError {
        IngestError::ThreadPool(err)
    }
}

impl From<MMCError> for IngestError {
    fn from(err: MMCError) -> IngestError {
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
