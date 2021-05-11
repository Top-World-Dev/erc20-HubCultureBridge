//! Solidity abi utilities.
//!
//! ## Examples
//!
//! Parse log topics as solidity event params:
//!
//! ```
//! #[macro_use]
//! extern crate serde_json;
//! extern crate ethrpc;
//!
//! use std::collections::HashMap;
//! use ethrpc::abi::{Event,Value};
//! use ethrpc::crypto;
//! 
//! # fn main() {
//! 
//! let spec = json!({
//!     "name":"MyEvent",
//!     "inputs": [
//!         {"name": "spam", "type": "address", "indexed": true},
//!         {"name": "eggs", "type": "uint256", "indexed": true},
//!         {"name": "cats", "type": "uint8"  , "indexed": true},
//!     ]
//! });
//!
//! let event: Event = serde_json::from_value(spec).unwrap();
//! 
//! assert_eq!(event.signature(),crypto::keccak("MyEvent(address,uint256,uint8)"));
//!
//! let topics = [
//!     "0x00000000000000000000000000000000000000000000000000000000deadbeef".parse().unwrap(),
//!     "0x0000000000000000000000000000000000000000000000000000000000123456".parse().unwrap(),
//!     "0x00000000000000000000000000000000000000000000000000000000000000ff".parse().unwrap(),
//! ];
//! 
//! let decoded: HashMap<_,_> = event.decode(&topics).collect();
//! 
//! let spam = decoded.get("spam").unwrap();
//! let eggs = decoded.get("eggs").unwrap();
//! let cats = decoded.get("cats").unwrap();
//!
//! assert!(spam.as_addr().is_some() && eggs.as_uint().is_some() && cats.as_uint8().is_some());
//! assert_eq!(spam.to_string(),"0x00000000000000000000000000000000deadbeef");
//! assert_eq!(eggs.to_string(),"0x123456");
//! assert_eq!(cats.to_string(),"0xff");
//!
//! let numbers = [
//!     Value::Uint("0xabc".parse().unwrap()),
//!     Value::Uint("0x123".parse().unwrap()),
//! ];
//!
//! let filter_topics = event.encode_topics::<&[_]>(&[&[],&numbers,&[]]).unwrap();
//!
//! let expected = json!([
//!     event.signature(),
//!     null,
//!     [
//!         "0x0000000000000000000000000000000000000000000000000000000000000abc",
//!         "0x0000000000000000000000000000000000000000000000000000000000000123",
//!     ],
//!     null
//! ]);
//!
//! assert_eq!(expected,serde_json::to_value(filter_topics).unwrap());
//! # }
//! ```
//!
//! Encode function calldata:
//! ```
//! #[macro_use]
//! extern crate serde_json;
//! extern crate ethrpc;
//!
//! use ethrpc::abi::{Function,Value};
//! use ethrpc::crypto;
//! 
//! # fn main() {
//! 
//! let spec = json!({
//!     "name":"hello",
//!     "inputs": [
//!         {"name": "foo", "type": "address" },
//!         {"name": "bar", "type": "uint256" }
//!     ]
//! });
//!
//! let function: Function = serde_json::from_value(spec).unwrap();
//! 
//! assert_eq!(function.selector().as_ref(),&crypto::keccak("hello(address,uint256)")[0..4]);
//!
//! let args = [
//!     Value::Addr("0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".parse().unwrap()),
//!     Value::Uint("0x123456".parse().unwrap()),
//! ];
//! 
//! let calldata = function.encode(&args).unwrap();
//!
//! let expected = "0xfb129803\
//! 000000000000000000000000deadbeefdeadbeefdeadbeefdeadbeefdeadbeef\
//! 0000000000000000000000000000000000000000000000000000000000123456";
//!
//! assert_eq!(&calldata.to_string(),expected);
//!
//! # }
//! ```
use serde::de::{Deserialize,Deserializer};
use serde::ser::{Serialize,Serializer};
use proxy::util::serde_str;
use types::{Bytes,H256,U256,Uint8};
use crypto::{Keccak256,Address};
use std::borrow::Borrow;
use std::str::FromStr;
use std::fmt;

mod function;
mod event;

pub use self::function::{
    Function,
    Selector,
    EncodeError,
};
pub use self::function::{
    Params as FunctionParams,
    Param as FunctionParam,
};
pub use self::event::{
    Event,
    FilterError,
};
pub use self::event::{
    Params as EventParams,
    Param as EventParam,
};


/// Encode abi values in packed format
///
/// ```
/// # extern crate ethrpc;
/// # use ethrpc::abi::{self,Value};
/// # fn main() {
/// 
/// let values = [
///     Value::Addr("0x00000000000000000000000000000000deadbeef".parse().unwrap()),
///     Value::Uint("0x12345".parse().unwrap()),
///     Value::Hash(
///         "0xaaaaffffaaaaffffaaaaffffaaaaffffaaaaffffaaaaffffaaaaffffaaaaffff".parse().unwrap()
///         ),
/// ];
///
/// let packed = abi::packed(&values);
///
/// let expect = "0x00000000000000000000000000000000deadbeef\
/// 0000000000000000000000000000000000000000000000000000000000012345\
/// aaaaffffaaaaffffaaaaffffaaaaffffaaaaffffaaaaffffaaaaffffaaaaffff";
///
/// assert_eq!(packed,expect.parse().unwrap());
/// # }
/// ```
///
pub fn packed<V>(values: impl IntoIterator<Item=V>) -> Bytes where V: Borrow<Value> {
    let mut buff = Vec::new();
    for val in values.into_iter() {
        buff.extend_from_slice(val.borrow().as_ref());
    }
    Bytes::from(buff)
}


/// Cast 256 bit evm word to the `Value` corresponding to
/// `Token`.
///
fn cast_word(token: Token, word: [u8;32]) -> Value {
    match token {
        Token::Addr => {
            let mut addr = Address::default();
            let offset = word.len() - addr.len();
            let truncated = &word[offset..];
            addr.copy_from_slice(truncated);
            addr.into()
        },
        Token::Uint8 => {
            let mut uint8 = Uint8::default();
            let offset = word.len() - uint8.len();
            let truncated = &word[offset..];
            uint8.copy_from_slice(truncated);
            uint8.into()
        },
        Token::Hash => H256::from(word).into(),
        Token::Uint => U256::from(word).into(),
    }
}


/// Calculate abi signature
///
pub(crate) fn signature(name: &str, tokens: impl IntoIterator<Item=Token>) -> H256 {
    let mut hasher = Keccak256::default();
    hasher.absorb(name.as_bytes());
    hasher.absorb(b"(");
    for (index,token) in tokens.into_iter().enumerate() {
        if index > 0 { hasher.absorb(b","); }
        hasher.absorb(token.as_bytes());
    }
    hasher.absorb(b")");
    hasher.finish().into()
}


/// An abi value.
///
#[derive(Hash,Debug,Copy,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(untagged)]
pub enum Value {
    Addr(Address), 
    Hash(H256),
    Uint8(Uint8),
    Uint(U256),
}


impl Value {

    /// Attempt to cast this value to the type indicated by `target`.
    ///
    /// If this value cannot be interpreted as `target`, the token
    /// indicating this value's type is returned.
    ///
    /// ```
    /// # extern crate ethrpc;
    /// # use ethrpc::abi::{Token,Value};
    /// # fn main() {
    /// let value = Value::Addr("0x00a329c0648769a73afac7f9381e08fb43dbea72".parse().unwrap());
    /// 
    /// // Casting to the current type produces unmodified value
    /// assert_eq!(value.try_cast(Token::Addr),Ok(value));
    ///
    /// // Invalid casts yield token indicating the current type
    /// assert_eq!(value.try_cast(Token::Hash),Err(Token::Addr));
    /// 
    /// // Casting to a uint from a value of equal or lesser size is OK.
    /// assert!(value.try_cast(Token::Uint).is_ok());
    /// # }
    /// ```
    ///
    pub fn try_cast(&self, target: Token) -> Result<Self,Token> {
        match target {
            Token::Addr => {
                let addr = self.as_addr().ok_or(self.token())?;
                Ok(Value::Addr(addr))
            },
            Token::Hash => {
                let hash = self.as_hash().ok_or(self.token())?;
                Ok(Value::Hash(hash))
            },
            Token::Uint8 => {
                let uint8 = self.as_uint8().ok_or(self.token())?;
                Ok(Value::Uint8(uint8))
            },
            Token::Uint => {
                let uint = self.as_uint().ok_or(self.token())?;
                Ok(Value::Uint(uint))
            },
        }
    }

    /// Get token indicating the type of this value.
    ///
    /// *note*: Some encodings are ambiguous; prefer using `try_cast`
    /// or one of the `as_*` methods rather than checking the
    /// type token directly.
    ///
    pub fn token(&self) -> Token {
        match self {
            Value::Addr(_) => Token::Addr,
            Value::Hash(_) => Token::Hash,
            Value::Uint8(_) => Token::Uint8,
            Value::Uint(_) => Token::Uint,
        }
    }

    /// Convert value to its raw 256-bit word representation.
    ///
    pub fn into_word(&self) -> [u8;32] {
        match self {
            Value::Addr(addr) => {
                let mut buf = [0u8;32];
                debug_assert!(addr.len() < buf.len());
                let offset = buf.len() - addr.len();
                (&mut buf[offset..]).copy_from_slice(&addr);
                buf
            },
            Value::Hash(hash) => hash.into_inner(),
            Value::Uint8(uint8) => {
                let mut buf = [0u8;32];
                debug_assert!(uint8.len() < buf.len());
                let offset = buf.len() - uint8.len();
                (&mut buf[offset..]).copy_from_slice(&uint8);
                buf
            },
            Value::Uint(uint) => uint.into_inner(),
        }
    }

    pub fn as_addr(&self) -> Option<Address> {
        match self {
            Value::Addr(addr) => Some(*addr),
            _other => None,
        }
    }

    pub fn as_hash(&self) -> Option<H256> {
        match self {
            Value::Hash(hash) => Some(*hash),
            _other => None,
        }
    }

    pub fn as_uint8(&self) -> Option<Uint8> {
        match self {
            Value::Uint8(uint8) => Some(*uint8),
            _other => None,
        }
    }

    pub fn as_uint(&self) -> Option<U256> {
        // addr & hash are both special cases of uint from
        // a decoded perspective, so allow caller to treat
        // them as such.
        Some(U256::from(self.into_word()))
    }
}


impl AsRef<[u8]> for Value {

    fn as_ref(&self) -> &[u8] {
        match self {
            Value::Addr(addr) => addr.as_ref(),
            Value::Hash(hash) => hash.as_ref(),
            Value::Uint8(uint8) => uint8.as_ref(),
            Value::Uint(uint) => uint.as_ref(),
        }
    }
}


impl fmt::Display for Value {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Addr(addr) => addr.fmt(f),
            Value::Hash(hash) => hash.fmt(f),
            Value::Uint8(uint8) => uint8.fmt(f),
            Value::Uint(uint) => uint.fmt(f),
        }
    }
}


impl From<Address> for Value {

    fn from(addr: Address) -> Self { Value::Addr(addr) }
}

impl From<Uint8> for Value {

    fn from(uint8: Uint8) -> Self { Value::Uint8(uint8) }
}

impl From<U256> for Value {

    fn from(uint: U256) -> Self { Value::Uint(uint) }
}

impl From<H256> for Value {

    fn from(hash: H256) -> Self { Value::Hash(hash) }
}


/// Indicates an expected abi type.
///
#[derive(Hash,Debug,Copy,Clone,PartialEq,Eq)]
pub enum Token {
    Addr,
    Uint8,
    Uint,
    Hash,
}


impl Token {

    /// Cast a 256-bit word-like value into the `Value` matching
    /// this token.
    ///
    /// *note*:  This operation truncates for values smaller than
    /// 256 bits (e.g. `Address`).
    ///
    pub fn cast_word(&self, word: impl Into<[u8;32]>) -> Value {
        cast_word(*self,word.into())
    }

    /// Get the type-string of this `Token`.
    ///
    pub fn as_str(&self) -> &str {
        match self {
            Token::Addr => "address",
            Token::Uint8 => "uint8",
            Token::Uint => "uint256",
            Token::Hash => "bytes32",
        }
    }

    /// Get type-string as bytes.
    ///
    pub fn as_bytes(&self) -> &[u8] { self.as_str().as_bytes() }
}


impl Serialize for Token {

    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok,S::Error> {
        serde_str::serialize(self,serializer)
    }
}

impl<'de> Deserialize<'de> for Token {

    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self,D::Error> {
        serde_str::deserialize(deserializer)
    }
}


/// Indicates failure to parse a `str` as a `Token`.
///
#[derive(Debug,Copy,Clone)]
pub struct ParseError;


impl fmt::Display for ParseError {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("unable to parse string as abi type")
    }
}

impl FromStr for Token {

    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        match s.trim() {
            "address" => Ok(Token::Addr),
            "uint8" => Ok(Token::Uint8),
            "uint" | "uint256" => Ok(Token::Uint),
            "bytes32" => Ok(Token::Hash),
            _other => Err(ParseError),
        }
    }
}


impl fmt::Display for Token {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}


impl AsRef<str> for Token {

    fn as_ref(&self) -> &str { self.as_str() }
}


