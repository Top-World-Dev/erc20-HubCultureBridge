//! asynchronous websocket communications
//!
//! Simple websocket based communication facilities for remote Mimir Bridge
//! client connections.
//!
//! ## server
//!
//! ```
//! extern crate tokio;
//! extern crate proxy;
//! 
//! use tokio::prelude::*;
//! use proxy::ws;
//! 
//! # fn example() {
//!
//! let addr = "127.0.0.1:8888".parse().unwrap();
//!
//! let echo_server = ws::listener(&addr).map_err(|e| println!("listener error: {:?}",e))
//!     .for_each(|conn| {
//!         let (tx,rx) = conn.split();
//!         let echo_all = rx.inspect(|msg| println!("echoing {:?}",msg))
//!             .forward(tx).then(|_| Ok(()));
//!         tokio::spawn(echo_all);
//!         Ok(())
//!     });
//!
//! tokio::run(echo_server);
//! # }
//! # fn main() { }
//! ```
//!
//! ## client
//!
//! ```
//! extern crate tokio;
//! extern crate proxy;
//! 
//! use tokio::prelude::*;
//! use proxy::ws;
//! 
//! # fn example() {
//!
//! let url = "ws://127.0.0.1:8888".parse().unwrap();
//!
//! let say_hello = ws::connect(url)
//!     .and_then(|conn| conn.send("hello".into()))
//!     .then(|_|Ok(()));
//!
//! tokio::run(say_hello);
//! # }
//! # fn main() { }
//! ```

pub(crate) mod util;
mod types;

pub use self::types::Message;

use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::{
    connect_async,
    accept_async,
};
use tokio::io::Error as IoError;
use tokio::net::TcpListener;
use tokio::prelude::*;
use url::Url;
use self::util::Connection;
use std::time::Duration;
use std::net::SocketAddr;
use std::{fmt,error};


#[derive(Debug)]
pub enum Error {
    Msg(String),
    ParseError,
    Ws(WsError),
    Io(IoError),
}


impl Error {

    pub fn message(msg: impl Into<String>) -> Self {
        Error::Msg(msg.into())
    }
}


impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Msg(msg) => f.write_str(msg),
            Error::ParseError => f.write_str("unable to parse websocket message"),
            Error::Ws(err) => err.fmt(f),
            Error::Io(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {

    fn description(&self) -> &str {
        match self {
            Error::Msg(msg) => msg,
            Error::ParseError => "unable to parse websocket message",
            Error::Ws(err) => err.description(),
            Error::Io(err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            Error::Msg(_) => None,
            Error::ParseError => None,
            Error::Ws(err) => Some(err),
            Error::Io(err) => Some(err),
        }
    }
}


impl From<WsError> for Error {

    fn from(err: WsError) -> Self { Error::Ws(err) }
}


impl From<IoError> for Error {

    fn from(err: IoError) -> Self { Error::Io(err) }
}

/// a websocket connection
///
pub trait WebSocketConn: Stream<Item=Message,Error=Error> + Sink<SinkItem=Message,SinkError=Error> + Send { }

impl<T> WebSocketConn for T where T: Stream<Item=Message,Error=Error> + Sink<SinkItem=Message,SinkError=Error> + Send { }


/// asynchronously connect to server (`ws://*` or `wss://*`)
///
pub fn connect(url: Url) -> impl Future<Item=impl WebSocketConn, Error=Error> + Send {
    connect_async(url).from_err()
        .map(|(ws_conn,_)| {
            let max_idle = Duration::from_secs(60);
            Connection::new(ws_conn,max_idle)
        })
}


/// listen for incoming websocket handshakes
///
pub fn listener(addr: &SocketAddr) -> impl Stream<Item=impl WebSocketConn, Error=Error> + Send {
    TcpListener::bind(addr).map(|listener| listener.incoming())
        .into_future().flatten_stream().from_err::<Error>()
        .map(|tcp_stream| {
            // TODO: add timeout (see `accept`)
            accept(tcp_stream).then(|rslt| {
                // log & then discard failed upgrade attempts...
                match rslt {
                    Ok(ws_conn) => Ok(Some(ws_conn)),
                    Err(error) => {
                        error!("ws upgrade failed with {:?}",error);
                        Ok(None)
                    },
                }
            })
        })
        // buffer pending upgrades (TODO: make configurable)
        .buffer_unordered(64)
        // filter out failed upgrades...
        .filter_map(|rslt: Option<_>| rslt)
}



/// attempt to upgrade tcp stream (or similar) to websocket
///
// TODO: add (optional?) timeout
fn accept<T>(tcp_stream: T) -> impl Future<Item=impl WebSocketConn, Error=Error> + Send where T: AsyncRead + AsyncWrite + Send {
    accept_async(tcp_stream).from_err()
        .map(|ws_conn| {
            let max_idle = Duration::from_millis(2048);
            Connection::new(ws_conn,max_idle)
        })
}
