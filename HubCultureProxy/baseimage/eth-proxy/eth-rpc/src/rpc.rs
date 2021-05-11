use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::{fmt,error};
use serde_json::Value;

use tokio::prelude::*;


pub type Result<T> = ::std::result::Result<T,Error>;

pub trait Transport<Req,Rsp>: Clone {

    type Error: fmt::Display;

    type Future: Future<Item=Result<Rsp>,Error=Self::Error> + Send + 'static;

    fn call(&self, request: Req) -> Self::Future;
}


impl<T,F,Req,Rsp> Transport<Req,Rsp> for T where
        T: Fn(Req) -> F + Clone, F: IntoFuture<Item=Result<Rsp>>,
        <F as IntoFuture>::Future: Send + 'static,
        <F as IntoFuture>::Error: fmt::Display {

    type Error = <F as IntoFuture>::Error;

    type Future = <F as IntoFuture>::Future;

    fn call(&self, request: Req) -> Self::Future {
        (self)(request).into_future()
    }
}


/// An RPC request payload (method & params)
///
/// *note*: it is the responsibility of the implementer to ensure that
/// `Params` serializes as a sequence.
///
pub trait Request {

    type Params: ?Sized + Serialize + fmt::Debug;

    fn method(&self) -> &str;

    fn params(&self) -> Option<&Self::Params>;
}


impl<M,P> Request for (M,P) where M: AsRef<str>, P: Serialize + fmt::Debug {

    type Params = P;

    fn method(&self) -> &str { self.0.as_ref() }

    fn params(&self) -> Option<&Self::Params> { Some(&self.1) }
}


/// An RPC response payload (non-error)
pub trait Response: DeserializeOwned {}

impl<T> Response for T where T: DeserializeOwned { }


/// Its all gone terribly wrong, but in a way we kinda expected...
///
/// Represent a jsonrpc error object.
/// 
/// ```
/// # extern crate ethrpc;
/// # extern crate serde_json;
/// # use ethrpc::Error;
///
/// let rpc_error = r#"{
///     "code":-32010,
///     "message":"Insufficient funds. The account you tried to send transaction from does not have enough funds. Required 270000 and got: 209850."
/// }"#;
///
/// let err: Error = serde_json::from_str(rpc_error).unwrap();
///
/// assert_eq!(err.code,-32010);
///
/// assert!(err.data.is_none());
///
/// ```
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Error {
    pub code: i64,
    pub message: String,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{} (code: {})",self.message,self.code)
    }
}


impl error::Error for Error {

    fn description(&self) -> &str { &self.message }
}
