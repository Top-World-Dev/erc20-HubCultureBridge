//! Misc internal helper types
//!
use rpc;


/// Generic JSON-RPC request serialization target
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Request<'a,P> {
    method: &'a str,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    params: Option<P>,
    id: u64,
    jsonrpc: Version,
}


impl<'a,P> Request<'a,P> {

    pub fn new(method: &'a str, params: Option<P>, id: u64) -> Self {
        let jsonrpc = Default::default();
        Self { method, params, id, jsonrpc }
    }
}


/// Generic JSON-RPC response deserialization target
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(untagged)]
pub enum Response<T> {
    Okay {
        id: u64,
        result: T,
    },
    Error {
        id: u64,
        error: rpc::Error,
    }
}


impl<T> Response<T> {

    pub fn id(&self) -> u64 {
        match self {
            Response::Okay { id, .. } => *id,
            Response::Error { id, .. } => *id,
        }
    }

    pub fn as_result(self) -> rpc::Result<T> {
        match self {
            Response::Okay { result, .. } => Ok(result),
            Response::Error { error, .. } => Err(error),
        }
    }
}


/// JSON-RPC protocol version
#[derive(Debug,Copy,Clone,Serialize,Deserialize)]
pub  enum Version {
    #[serde(rename = "2.0")]
    Two
}

impl Default for Version {

    fn default() -> Self { Version::Two }
}

