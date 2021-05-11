//! Misc types
pub use mimir_common::types::Bytes;
pub use mimir_common::types::U256;
pub use mimir_common::types::H256;
pub use tokio_util::Never;

pub mod helpers;
mod transaction;
mod filter;
mod block;
mod uint8;
mod log;

pub use self::uint8::Uint8;
pub use self::transaction::{
    Transaction,
    TxInfo,
    TxCall,
    Receipt,
    Status, 
};
pub use self::filter::{
    Filter,
    FilterBuilder,
    Topic,
    Topics,
    Origin,
};
pub use self::block::Block;
pub use self::log::Log;

use serde::de::{Deserialize,Deserializer};
use serde::ser::{Serialize,Serializer};
use proxy::util::serde_str;
use std::str::FromStr;
use std::{fmt,error};


/// Block identifier for Ethereum RPC
///
#[derive(Debug,Copy,Clone)]
pub enum BlockId {
    /// Specifies earliest available block (e.g. genesis)
    Earliest,
    /// Specifies most recently mined block
    Latest,
    /// Specifies pending values (not yet "mined", possibly never will be) 
    Pending,
    /// Specifies a block by its specific number
    Number(U256),
}


impl Default for BlockId {

    fn default() -> Self { BlockId::Latest }
}


impl From<U256> for BlockId {

    fn from(num: U256) -> Self { BlockId::Number(num) }
}


impl Serialize for BlockId {

    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok,S::Error> {
        serde_str::serialize(self,serializer)
    }
}

impl<'de> Deserialize<'de> for BlockId {

    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self,D::Error> {
        serde_str::deserialize(deserializer)
    }
}


impl fmt::Display for BlockId {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockId::Earliest => f.write_str("earliest"),
            BlockId::Latest => f.write_str("latest"),
            BlockId::Pending => f.write_str("pending"),
            BlockId::Number(num) => num.fmt(f),
        }
    }
}


impl FromStr for BlockId {

    type Err = ParseBlockError;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        match s {
            "earliest" => Ok(BlockId::Earliest),
            "latest" => Ok(BlockId::Latest),
            "pending" => Ok(BlockId::Pending),
            other => {
                let num = other.parse().map_err(|_|ParseBlockError)?;
                Ok(BlockId::Number(num))
            }
        }
    }
}

#[derive(Debug,Copy,Clone)]
pub struct ParseBlockError;


impl fmt::Display for ParseBlockError {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid block id")
    }
}


impl error::Error for ParseBlockError {

    fn description(&self) -> &str { "invalid block id" }
}
