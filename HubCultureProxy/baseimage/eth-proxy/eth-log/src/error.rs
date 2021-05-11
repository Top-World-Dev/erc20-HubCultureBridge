use proxy::{self,http};
use ethrpc::api;
use tokio::timer;
use tera;
use ignore;
use std::{fmt,io,error};


wrap_errs! {
    Tera => tera::Error,
    Http => http::Error,
    Proxy => proxy::Error,
    Node => api::Error,
    Time => timer::Error,
    Ignore => ignore::Error,
    Io => io::Error,
    Msg => ErrorMsg,
}


impl Error {

    pub fn message(msg: impl Into<String>) -> Self {
        Self::from(msg.into())
    }
}


impl From<http::HyperError> for Error {

    fn from(err: http::HyperError) -> Self { Error::Http(err.into()) }
}

impl From<http::HttpError> for Error {

    fn from(err: http::HttpError) -> Self { Error::Http(err.into()) }
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
