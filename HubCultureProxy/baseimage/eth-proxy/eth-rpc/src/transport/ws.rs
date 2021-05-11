//! WebSocket based transport
//!
//! ## Example
//!
//! ```
//! extern crate ethrpc;
//! extern crate tokio;
//!
//! use tokio::prelude::*;
//! use ethrpc::transport::ws;
//! use ethrpc::types::U256;
//! use ethrpc::Transport;
//! 
//! # fn example() {
//! 
//! let uri = "ws://127.0.0.1:8546".parse().unwrap();
//!
//! let work = ws::connect(uri).map_err(drop).and_then(|handle| {
//!     let req = ("eth_getBalance",["0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"]);
//!     handle.call(req).map(|rslt: ethrpc::Result<U256>| {
//!         println!("Got Balance: {}",rslt.unwrap());
//!     }).map_err(drop)
//! });
//!
//! tokio::run(work);
//!
//! # }
//!
//! # fn main() { }
//! ```
//!
use rpc::{self,Request,Response};
use transport::{helpers,plex};
use tokio::prelude::*;
use proxy::ws;
use url::Url;
use std::fmt;
use serde_json::{self,Value};


wrap_errs!(
    Json => serde_json::Error,
    Ws => ws::Error,
);


pub type Handle<Req,Rsp> = plex::TimeoutHandle<Req,rpc::Result<Rsp>>;


/// Initialize a JSONRPC style multiplexed websocket connection.
///
/// See module-level docs for example usage.
///
pub fn connect<Req,Rsp>(url: Url) -> impl Future<Item=impl rpc::Transport<Req,Rsp>,Error=Error> where
        Req: Request + fmt::Debug + Send + 'static,
        Rsp: Response + fmt::Debug + Send + 'static {
    
    future::lazy(move || {
        ws::connect(url).from_err().and_then(|conn| {
            let conn = conn.sink_from_err::<Error>().with(|(id,req): (u64,Req)| -> Result<_,Error> {
                let request = helpers::Request::new(
                    req.method(),
                    req.params(),
                    id
                    );
                debug!("Sending {:?}",request);
                let ser = ws::Message::encode_json(&request)?;
                Ok(ser.into())
            });
            let conn = conn.from_err().and_then(|msg| -> Result<(u64,rpc::Result<Rsp>),Error> {
                let rsp: helpers::Response<Rsp> = msg.parse_json().map_err(|err| {
                    match msg.parse_json::<Value>() {
                        Ok(other) => { warn!("Unexpected json: {}",other); err },
                        Err(err) => { warn!("Non-json message: {}",err); err },
                    }
                })?;
                debug!("Got {:?}",rsp);
                let id = rsp.id();
                Ok((id,rsp.as_result()))
            });
            plex::spawn(conn).map_err(|e| e.into()).map(|handle| {
                let transport = move |req| { handle.call(req) };
                transport
            })
        })
    })
}

