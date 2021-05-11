use tokio::io;
use native_tls;
use std::{fmt,error};


#[derive(Debug)]
pub enum Error {
    Message(String),
    Io(io::Error),
    Tls(native_tls::Error),
}


impl Error {


    pub fn message(msg: impl Into<String>) -> Self { Self::from(msg.into()) }
    
    #[allow(unused)]
    fn assertions() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Self>()
    }
}


impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => msg.fmt(f),
            Error::Io(err) => err.fmt(f),
            Error::Tls(err) => err.fmt(f)
        }
    }
}


impl error::Error for Error {

    fn description(&self) -> &str {
        match self {
            Error::Message(msg) => &msg,
            Error::Io(err) => err.description(),
            Error::Tls(err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            Error::Message(_) => None,
            Error::Io(err) => Some(err),
            Error::Tls(err) => Some(err),
        }
    }
}


impl From<String> for Error {

    fn from(msg: String) -> Self { Error::Message(msg) }
}


impl From<io::Error> for Error {

    fn from(err: io::Error) -> Self { Error::Io(err) }
}

impl From<native_tls::Error> for Error {

    fn from(err: native_tls::Error) -> Self { Error::Tls(err) }
}
