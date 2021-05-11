use tokio::timer;
use ethrpc::api;
use signer::rpc;
use std::{fmt,error};


/// Error indicating failure in rpc signer
///
#[derive(Debug)]
pub enum Error {
    /// Error during signer initialization
    Signer(rpc::Error),
    /// Error originating from event-loop timer
    Timer(timer::Error),
    /// Error during communication with local node
    Node(api::Error), 
}


impl From<rpc::Error> for Error {

    fn from(err: rpc::Error) -> Self { Error::Signer(err) }
}


impl From<timer::Error> for Error {

    fn from(err: timer::Error) -> Self { Error::Timer(err) }
}

impl From<api::Error> for Error {

    fn from(err: api::Error) -> Self { Error::Node(err) }
}


impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Signer(err) => err.fmt(f),
            Error::Timer(err) => err.fmt(f),
            Error::Node(err) => err.fmt(f),
        }
    }
}


impl error::Error for Error {

    fn description(&self) -> &str {
        match self {
            Error::Signer(err) => err.description(),
            Error::Timer(err) => err.description(),
            Error::Node(err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            Error::Signer(err) => Some(err),
            Error::Timer(err) => Some(err),
            Error::Node(err) => Some(err),
        }
    }
}

