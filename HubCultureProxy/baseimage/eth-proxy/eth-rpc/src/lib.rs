//! Ethereum Rpc
//!
//! ## Example
//!
//! ```
//! extern crate ethrpc;
//! extern crate tokio;
//!
//! use tokio::prelude::*;
//!
//! # fn example() {
//!
//! // Connect to default rpc endoint
//! let work = ethrpc::autoconnect().and_then(|api| {
//!     // Load the current block-number
//!     api.eth().block_number().and_then(move |num| {
//!         println!("The current block number is: {}",num);
//!         // Load account addresses
//!         api.eth().accounts().and_then(move |accounts| {
//!             println!("Available accounts: {:?}",accounts);
//!             // Generate a future to load each account's current balance
//!             let get_balances = accounts.iter().map(|addr| {
//!                 api.eth().get_balance(*addr,num.into())
//!             }).collect::<Vec<_>>();
//!             // Load all balances concurrently
//!             future::collect(get_balances).map(move |balances| {
//!                 // `future::collect` preserves ordering, so `zip` will
//!                 // re-associate balances correctly.
//!                 for (addr,bal) in accounts.iter().zip(balances) {
//!                     println!("Address: {} Balance: {}",addr,bal);
//!                 }
//!             })
//!         })
//!     })
//! }).map_err(drop);
//!
//! tokio::run(work);
//!
//! # }
//! # fn main() { }
//! ```
#[macro_use]
extern crate mimir_common;
extern crate mimir_crypto;
#[macro_use]
extern crate proxy;
extern crate rlp as _rlp;
extern crate tokio_util;
extern crate tokio_channel;
extern crate tokio;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
#[macro_use]
extern crate smallvec;
extern crate rand;
#[macro_use]
extern crate log;
extern crate url;

macro_rules! try_ready {
    ($e:expr) => (match $e {
        Ok(Async::Ready(t)) => t,
        Ok(Async::NotReady) => return Ok(Async::NotReady),
        Err(e) => return Err(From::from(e)),
    })
}

pub mod transport;
pub mod transaction;
pub mod crypto;
pub mod types;
pub mod util;
pub mod api;
pub mod abi;


pub(crate) mod rpc;
pub use rpc::*;

pub use url::Url;

pub use api::{
    connect,
    autoconnect,
};
