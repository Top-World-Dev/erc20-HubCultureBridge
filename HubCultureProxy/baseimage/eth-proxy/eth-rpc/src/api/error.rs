use tokio::timer;
use std::{fmt,error};
use api::Response;
use rpc;


wrap_errs!(
    Rpc => rpc::Error,
    Rsp => Unexpected,
    Timer => timer::Error,
    Fatal => TransportFailed,
);


/// Opaque error indicating an unrecoverable transport failure.
///
/// Typically indicates a lost connection or invalid arguments.  Transports
/// are required to produce `ERROR` level logs detailing the reason for
/// their failure.
///
#[derive(Debug,Copy,Clone)]
pub struct TransportFailed;


impl fmt::Display for TransportFailed {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("transport failed (see logs for details)")
    }
}

impl error::Error for TransportFailed {

    fn description(&self) -> &str { "transport failed (see logs for details)" }
}


/// Error indicating unexpected response payload.
///
/// This error specifically indicates that the response was a valid JSON-RPC
/// payload, but was not the *expected* payload.
///
#[derive(Debug,Clone)]
pub struct Unexpected<T=Response> {
    /// Short description of expected payload
    pub expecting: &'static str,
    /// The payload that was actually recieved
    pub got: T
}


impl<T> fmt::Display for Unexpected<T> {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"invalid response (expecting {})",self.expecting)
    }
}


impl error::Error for Unexpected {

    fn description(&self) -> &str { "unexpected response value" }
}

