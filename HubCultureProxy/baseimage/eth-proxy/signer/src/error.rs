use mimir_common::util::hex::ParseHexError;
use std::{fmt,io,error};
use toml;
use functions;
use crypto;

wrap_errs! {
    Crypto => crypto::Error,
    Toml => toml::de::Error,
    Hex => ParseHexError,
    Io => io::Error,
    Function => functions::Error,
    Msg => ErrorMsg,
}


impl Error {

    pub fn message(msg: impl Into<String>) -> Self {
        Self::from(msg.into())
    }
}


impl From<String> for Error {

    fn from(msg: String) -> Self { Error::Msg(ErrorMsg(msg)) }
}

impl<'a> From<&'a str> for Error {

    fn from(msg: &'a str) -> Self { Error::from(msg.to_string()) }
}


#[derive(Debug,Clone)]
pub struct ErrorMsg(String);


impl fmt::Display for ErrorMsg {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}


impl error::Error for ErrorMsg {

    fn description(&self) -> &str {
        &self.0
    }
}

