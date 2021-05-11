//! The Ethereum Api
//!
//! ## Example
//!
//! ```
//! extern crate ethrpc;
//! extern crate tokio;
//!
//! use tokio::prelude::*;
//!
//!
//! # fn example() {
//!
//! let uri = "ws://127.0.0.1:8546".parse().unwrap();
//!
//! let work = ethrpc::connect(uri).map_err(drop).and_then(|api| {
//!     // Load the current block-number
//!     api.eth().block_number().and_then(move |block_number| {
//!         println!("The current block number is: {}",block_number);
//!         // Load account addresses
//!         api.eth().accounts().and_then(move |accounts| {
//!             println!("Available accounts: {:?}",accounts);
//!             let block = block_number.into();
//!             // Generate a future to load each account's current balance
//!             let get_balances = accounts.iter().map(|addr| {
//!                 api.eth().get_balance(*addr,block)
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
//!     }).map_err(drop)
//! });
//!
//! tokio::run(work);
//!
//! # }
//! # fn main() { }
//! ```
//!
use types::{Filter,Log,Bytes,U256,H256,BlockId,Block,Transaction,TxInfo,TxCall,Receipt,Never};
use crypto::Address;
use rpc;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::fmt;
use serde_json::Value;


pub mod error;
mod util;
mod eth;

pub use self::error::Error;
pub use self::util::{Util,LatestLogs};
pub use self::eth::Eth;

use self::error::{Unexpected,TransportFailed};
use tokio::prelude::*;
use transport;
use url::Url;


/// Connect to specified node
///
pub fn connect(url: Url) -> impl Future<Item=Api<impl rpc::Transport<Request,Response>>,Error=Error> {
    transport::ws::connect(url).map(Api::new).map_err(|e| -> Error {
        error!("During connect: {:?}",e);
        Error::from(error::TransportFailed)
    })
}


/// Connect to default endpoint
///
pub fn autoconnect() -> impl Future<Item=Api<impl rpc::Transport<Request,Response>>,Error=Error> {
    let url = "ws://127.0.0.1:8546".parse().unwrap();
    connect(url)
}


/// Top-level API handle
///
/// See module-level docs for example usage.
///
#[derive(Debug,Clone)]
pub struct Api<T> {
    transport: T,
}


impl<T> Api<T> {

    /// Wrap an existing transport.
    ///
    pub fn new(transport: T) -> Self { Self { transport } }

    /// Access the `eth_*` namespace.
    ///
    pub fn eth(&self) -> Eth<T> { Eth::new(&self.transport) }

    /// Access extra utility functions.
    ///
    pub fn util(&self) -> Util<T> { Util::new(&self.transport) }
}


/// An Ethereum JSON-RPC request payload
///
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(untagged)]
pub enum Request {
    /// Equivalent to `eth_getLogs`
    GetLogs([Filter;1]),
    /// Equivalent to `eth_getBlockByNumber`
    GetBlockByNumber(BlockId,bool),
    /// Equivalent to `eth_getTransactionByHash`
    GetTxByHash([H256;1]),
    /// Equivalent to `eth_getTransactionReceipt`
    GetTxReceipt([H256;1]),
    /// Equivalent to `eth_getBalance`
    GetBalance(Address,BlockId),
    /// Equivalent to `eth_getTransactionCount`
    GetTxCount(Address,BlockId),
    /// Equivalent to `eth_estimateGas`
    EstimateGas(Transaction,BlockId),
    /// Equivalten to `eth_call`
    Call(TxCall,BlockId),
    /// Equivalent to `eth_sendRawTransaction`
    SendRawTx([Bytes;1]),
    /// Equivalent to `eth_blockNumber`
    BlockNumber,
    /// Equivalent to `eth_gasPrice`
    GasPrice,
    /// Equivalent to `eth_accounts`
    Accounts,
}


impl Request {

    /// Construct a request for the `eth_getLogs` method.
    ///
    pub fn get_logs(filter: Filter) -> Self { Request::GetLogs([filter]) }

    /// Construct a request for the `eth_getBlockByNumber` method.
    ///
    pub fn get_block_by_number(block: BlockId, full: bool) -> Self { Request::GetBlockByNumber(block,full) }

    /// Construct a request for the `eth_getTransactionByHash` method.
    ///
    pub fn get_tx_by_hash(hash: H256) -> Self { Request::GetTxByHash([hash]) }

    /// Construct a request for the `eth_getTransactionReceipt` method.
    ///
    pub fn get_tx_receipt(hash: H256) -> Self { Request::GetTxReceipt([hash]) }

    /// Construct a request for the `eth_getBalance` method.
    ///
    pub fn get_balance(addr: Address, block: BlockId) -> Self { Request::GetBalance(addr,block) }

    /// Construct a request for the `eth_getTransactionCount` method.
    ///
    pub fn get_tx_count(addr: Address, block: BlockId) -> Self { Request::GetTxCount(addr,block) }

    /// Construct a request for the `eth_estimateGas` method.
    ///
    pub fn estimate_gas(tx: Transaction, block: BlockId) -> Self { Request::EstimateGas(tx,block) }

    /// Construct a request for the `eth_call` method.
    ///
    pub fn call(tx: TxCall, block: BlockId) -> Self { Request::Call(tx,block) }
    
    /// Construct a request for the `eth_sendRawTransaction` method.
    ///
    pub fn send_raw_tx(bytes: Bytes) -> Self { Request::SendRawTx([bytes]) }

    /// Construct a request for the `eth_blockNumber` method.
    ///
    pub fn block_number() -> Self { Request::BlockNumber }

    /// Construct a request for the `eth_gasPrice` method.
    ///
    pub fn gas_price() -> Self { Request::GasPrice }

    /// Contruct a request for the `eth_accounts` method.
    ///
    pub fn accounts() -> Self { Request::Accounts }
}


impl rpc::Request for Request {

    type Params = Self;

    fn method(&self) -> &str {
        match self {
            Request::GetLogs(_) => "eth_getLogs",
            Request::GetBlockByNumber(_,_) => "eth_getBlockByNumber",
            Request::GetTxByHash(_) => "eth_getTransactionByHash",
            Request::GetTxReceipt(_) => "eth_getTransactionReceipt",
            Request::GetBalance(_,_) => "eth_getBalance",
            Request::GetTxCount(_,_) => "eth_getTransactionCount",
            Request::EstimateGas(_,_) => "eth_estimateGase",
            Request::Call(_,_) => "eth_call",
            Request::SendRawTx(_) => "eth_sendRawTransaction",
            Request::BlockNumber => "eth_blockNumber",
            Request::GasPrice => "eth_gasPrice",
            Request::Accounts => "eth_accounts",
        }
    }

    fn params(&self) -> Option<&Self::Params> {
        match self {
            Request::GetLogs(_) => Some(self),
            Request::GetBlockByNumber(_,_) => Some(self),
            Request::GetTxByHash(_) => Some(self),
            Request::GetTxReceipt(_) => Some(self),
            Request::GetBalance(_,_) => Some(self),
            Request::GetTxCount(_,_) => Some(self),
            Request::EstimateGas(_,_) => Some(self),
            Request::Call(_,_) => Some(self),
            Request::SendRawTx(_) => Some(self),
            Request::BlockNumber => None,
            Request::GasPrice => None,
            Request::Accounts => None,
        }
    }
}


/// An Ethereum JSON-RPC response payload
///
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(untagged)]
pub enum Response {
    /// A null value
    NullValue(()),
    /// Empty sequence
    EmptySeq([Never;0]),
    /// Empty mapping
    EmptyMap(HashMap<Never,Never>),
    /// A block datastructure
    Block(Block<H256>),
    /// Pending/mined transaction info
    TxInfo(TxInfo),
    /// Transaction execution receipt
    TxReceipt(Receipt),
    /// Sequence of log objects
    Logs(Vec<Log>),
    /// Sequence of addresses
    Addrs(Vec<Address>),
    /// A single address
    Addr(Address),
    /// 256-bit arbitrary bytearray
    Hash(H256),
    /// Unsigned 256-bit integer (hex encoded)
    Uint(U256), 
    /// Arbitrary byte-array
    Bytes(Bytes),
    /// Arbitrary JSON
    Other(Value),
}


impl Response {

    pub fn expect_block(self) -> Result<Option<Block<H256>>,Unexpected> {
        match self {
            Response::Block(block) => Ok(Some(block)),
            Response::NullValue(()) => Ok(None),
            other => Err(Unexpected {
                expecting: "block or null",
                got: other,
            }),
        }
    }

    pub fn expect_tx_info(self) -> Result<Option<TxInfo>,Unexpected> {
        match self {
            Response::TxInfo(tx) => Ok(Some(tx)),
            Response::NullValue(()) => Ok(None),
            other => Err(Unexpected {
                expecting: "transaction or null",
                got: other,
            }),
        }
    }

    pub fn expect_tx_receipt(self) -> Result<Option<Receipt>,Unexpected> {
        match self {
            Response::TxReceipt(receipt) => Ok(Some(receipt)),
            Response::NullValue(()) => Ok(None),
            other => Err(Unexpected {
                expecting: "receipt or null",
                got: other,
            })
        }
    }

    pub fn expect_logs(self) -> Result<Vec<Log>,Unexpected> {
        match self {
            Response::Logs(logs) => Ok(logs),
            Response::EmptySeq(_) => Ok(Default::default()),
            other => Err(Unexpected {
                expecting: "array of logs",
                got: other,
            }),
        }
    }

    pub fn expect_addrs(self) -> Result<Vec<Address>,Unexpected> {
        match self {
            Response::Addrs(addrs) => Ok(addrs),
            Response::EmptySeq(_) => Ok(Default::default()),
            other => Err(Unexpected {
                expecting: "array of addresses",
                got: other,
            })
        }
    }

    pub fn expect_hash(self) -> Result<H256,Unexpected> {
        match self {
            Response::Hash(hash) => Ok(hash),
            other => Err(Unexpected {
                expecting: "256-bit bytes-like value",
                got: other,
            })
        }
    }

    pub fn expect_uint(self) -> Result<U256,Unexpected> {
        match self {
            Response::Uint(uint) => Ok(uint),
            Response::Addr(addr) => {
                let mut buf = [0u8;32];
                let offset = buf.len() - addr.len();
                (&mut buf[offset..]).copy_from_slice(&addr);
                Ok(buf.into())
            },
            Response::Hash(hash) => Ok(hash.into_other()),
            other => Err(Unexpected {
                expecting: "256-bit unsigned integer",
                got: other,
            })
        }
    }

    pub fn expect_bytes(self) -> Result<Bytes,Unexpected> {
        match self {
            Response::Bytes(bytes) => Ok(bytes),
            Response::Addr(addr) => {
                let mut buf = Vec::with_capacity(addr.len());
                buf.extend_from_slice(&addr);
                Ok(buf.into())
            },
            Response::Hash(hash) => {
                let mut buf = Vec::with_capacity(hash.len());
                buf.extend_from_slice(&hash);
                Ok(buf.into())
            },
            Response::Uint(uint) => {
                let trimmed = ::util::trim(&uint);
                let mut buf = Vec::with_capacity(trimmed.len());
                buf.extend_from_slice(&trimmed);
                Ok(buf.into())
            },
            other => Err(Unexpected {
                expecting: "arbitrary byte-array",
                got: other,
            })
        }
    }
}


/// A type which may or may not be an expected value.
///
pub trait Expect<T>: Sized {

    fn as_expected(self) -> Result<T,Unexpected<Self>>;
}

macro_rules! impl_expect {
    ($($method:ident => $type:ty,)*) => {
        $(
            impl Expect<$type> for Response {
                
                fn as_expected(self) -> Result<$type,Unexpected> { self.$method() }
            }
        )*
    }
}


impl_expect! {
    expect_block => Option<Block<H256>>, 
    expect_tx_info => Option<TxInfo>,
    expect_tx_receipt => Option<Receipt>,
    expect_logs => Vec<Log>,
    expect_addrs => Vec<Address>,
    expect_hash => H256,
    expect_uint => U256,
    expect_bytes => Bytes,
}


/// An asynchronous rpc operation.
///
pub enum AsyncRpc<F,T> {
    /// Drive inner future to completion
    Work {
        inner: F,
        expect: PhantomData<T>,
    },
    /// Fail immediately with supplied error
    Fail {
        error: Option<Error>,
    },
}


impl<F,T> AsyncRpc<F,T> where F: Future<Item=rpc::Result<Response>>, Response: Expect<T>, F::Error: fmt::Display {

    pub fn new(inner: F) -> Self {
        AsyncRpc::Work { inner, expect: PhantomData }
    }

    pub fn fail(reason: impl Into<Error>) -> Self {
        AsyncRpc::Fail { error: Some(reason.into()) }
    }

    fn poll_inner(&mut self) -> Poll<T,Error> {
        match self {
            AsyncRpc::Work { ref mut inner, .. } => {
                let item = try_ready!(inner.poll().map_err(|e| {
                    warn!("Transport failed: {}",e);
                    Error::from(TransportFailed)
                }))?;
                let expected = item.as_expected()?;
                Ok(Async::Ready(expected))
            },
            AsyncRpc::Fail { ref mut error } => {
                let err = error.take().expect("No polling past completion");
                Err(err)
            },
        }
    }
}


impl<F,T> Future for AsyncRpc<F,T> where F: Future<Item=rpc::Result<Response>>, Response: Expect<T>, F::Error: fmt::Display {
    
    type Item = T;

    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item,Self::Error> {
        self.poll_inner()
    }
}

