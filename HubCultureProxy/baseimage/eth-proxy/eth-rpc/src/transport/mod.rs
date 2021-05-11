//! For when where u is ain't where u tryin' to be
//!
pub(crate) mod helpers;
pub mod plex;
pub mod ws;


/// Generic transport error
///
/// *note*: This will become its own independent error type once more
/// transports are implemented.  Prefer working with `ws::Error` if you
/// are using the websocket transport directly.
///
pub type Error = ws::Error;

