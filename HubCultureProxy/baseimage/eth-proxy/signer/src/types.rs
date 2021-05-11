//! Misc types
pub use mimir_common::types::Never;
pub use mimir_common::types::Bytes;
pub use mimir_common::types::U256;
pub use mimir_common::types::H256;

use std::ops::{Deref,DerefMut};
use util::unix_time;

/// A randomly generated 256 bit token
pub type Token = H256;

/// A bytes-like uuid
pub type UUID = U256;

/// A 256 bit unix timestamp
#[derive(Debug,Copy,Clone,PartialOrd,Ord,PartialEq,Eq,Serialize,Deserialize)]
pub struct Timestamp(U256);


impl Timestamp {

    /// Get the current time
    pub fn now() -> Self { Timestamp(unix_time().into()) }
}


impl Deref for Timestamp {

    type Target = U256;

    fn deref(&self) -> &Self::Target { &self.0 }
}


impl DerefMut for Timestamp {

    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}




