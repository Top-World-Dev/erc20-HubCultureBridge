use hyper::client::HttpConnector;
use hyper_tls::{self,HttpsConnector};
use tokio_util::service::{self,Service};
use tokio::prelude::future::Either;
use tokio::prelude::*;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use log::Level;
use std::{fmt,error};
use serde_json::{self,Value};
use hyper;
use _http;

pub use hyper::Error as HyperError;
pub use _http::Error as HttpError;
pub use _http::{Request,Response,Uri};
pub use hyper::Body;

const CONNECTOR_THREADS: usize = 2;


/// Alias for tls enabled http client
///
pub type Client = hyper::Client<HttpsConnector<HttpConnector>,Body>;


/// Configure a tls enabled client with reasonable defaults
///
pub fn client() -> Result<Client,Error> {
    let connector = HttpsConnector::new(CONNECTOR_THREADS)?;
    let client = hyper::Client::builder()
        .build(connector);
    Ok(client)
}


/// Marker trait indicating a task-safe error value
pub trait SafeErr: error::Error + Send + 'static { }

impl<T> SafeErr for T where T: error::Error + Send + 'static { }

/// Marker trait indicating a task-safe I/O source
pub trait SafeIo: AsyncRead + AsyncWrite + Send + 'static { }

impl<T> SafeIo for T where T: AsyncRead + AsyncWrite + Send + 'static { }


pub fn serve_json<Srv,Req,Rsp>(incoming: impl Stream<Item=impl SafeIo,Error=impl SafeErr + Sync>, service: Srv) -> impl Future<Item=(),Error=hyper::Error> 
    where
        Srv: Service<Req,Rsp,Error = ()> + Send + 'static,
        Srv::Future: Send + 'static,
        Req: DeserializeOwned + fmt::Debug + Send + 'static,
        Rsp: Serialize + fmt::Debug + Send + 'static {

    service::spawn(service).map_err(|e| e.into()).and_then(|handle| {

        let new_service = move || -> Result<_,String> {
            let capture_handle = handle.clone();
            let service_fn = hyper::service::service_fn(move |req: Request<Body>| {
                let sub_capture_handle = capture_handle.clone();
                from_json_body(req.into_body()).then(move |rslt| {
                    let rslt = rslt.map_err(|e| e.to_string());
                    match rslt {
                        Ok(req) => {
                            Either::A(sub_capture_handle.call(req).map_err(|e|e.to_string()).and_then(|rsp| {
                                into_json_body(&rsp).map_err(|e|e.to_string()).and_then(|body| {
                                    //Response::new(body)
                                    Response::builder()
                                        .header("Content-Type","application/json")
                                        .status(200)
                                        .body(body)
                                        .map_err(|e|e.to_string())
                                })
                            }))
                        },
                        Err(_) => {
                            Either::B(Response::builder()
                                .header("Content-Type","application/json")
                                .status(400)
                                .body(json!(rslt.map(drop)).to_string().into())
                                .map_err(|e|e.to_string())
                                .into_future()
                            )
                        }
                    }
                })
            });
            Ok(service_fn)
        };

        hyper::Server::builder(incoming).serve(new_service)

    })
}


pub type ParseJsonBody = ParseBody<Value>;

#[derive(Debug)]
pub enum ParseBody<T> {
    Json(T),
    Str(String),
    Raw(Vec<u8>),
}


impl<T> ParseBody<T> where T: DeserializeOwned {

    pub fn parse(body: Body) -> impl Future<Item=Self,Error=Error> {
        body.concat2().from_err().map(|body| {
            Self::parse_bytes(&body.into_bytes())
        })
    }

    pub fn parse_bytes(bytes: &[u8]) -> Self {
        if let Ok(value) = serde_json::from_slice(bytes) {
            ParseBody::Json(value)
        } else {
            match String::from_utf8(bytes.to_owned()) {
                Ok(s) => ParseBody::Str(s),
                Err(e) => ParseBody::Raw(e.into_bytes()),
            }
        }
    }
}



pub fn from_json_body<T>(body: Body) -> impl Future<Item=T,Error=Error> where T: DeserializeOwned {
    body.concat2().from_err().and_then(|body| {
        let bytes = body.into_bytes();
        match serde_json::from_slice(&bytes) {
            Ok(parsed) => Ok(parsed),
            Err(err) => {
                if log_enabled!(Level::Debug) {
                    if let Ok(other) = serde_json::from_slice::<Value>(&bytes) {
                        debug!("Unexpected json: {}",other);
                    }
                }
                Err(err.into())
            }
        }
    })
}


pub fn into_json_body<T>(body: &T) -> impl Future<Item=Body,Error=Error> where T: Serialize {
    serde_json::to_vec(body).map_err(From::from).map(From::from).into_future()
}


#[derive(Debug)]
pub enum Error {
    Hyper(hyper::Error),
    HyperTls(hyper_tls::Error),
    Http(_http::Error),
    Json(serde_json::Error),
    ServiceFailed,
}


impl From<hyper::Error> for Error {

    fn from(err: hyper::Error) -> Self { Error::Hyper(err) }
}


impl From<hyper_tls::Error> for Error {

    fn from(err: hyper_tls::Error) -> Self { Error::HyperTls(err) }
}


impl From<_http::Error> for Error {

    fn from(err: _http::Error) -> Self { Error::Http(err) }
}


impl From<serde_json::Error> for Error {

    fn from(err: serde_json::Error) -> Self { Error::Json(err) }
}

impl<T> From<service::Error<T>> for Error {

    fn from(_err: service::Error<T>) -> Self { Error::ServiceFailed }
}


impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Hyper(err) => err.fmt(f),
            Error::HyperTls(err) => err.fmt(f),
            Error::Http(err) => err.fmt(f),
            Error::Json(err) => err.fmt(f),
            Error::ServiceFailed => f.write_str("service failed (likely fatal)"),
        }
    }
}


impl error::Error for Error {

    fn description(&self) -> &str {
        match self {
            Error::Hyper(err) => err.description(),
            Error::HyperTls(err) => err.description(),
            Error::Http(err) => err.description(),
            Error::Json(err) => err.description(),
            Error::ServiceFailed => "service failed (likely fatal)",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            Error::Hyper(err) => Some(err),
            Error::HyperTls(err) => Some(err),
            Error::Http(err) => Some(err),
            Error::Json(err) => Some(err),
            Error::ServiceFailed => None,
        }
    }
}
