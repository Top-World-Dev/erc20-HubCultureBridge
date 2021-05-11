use serde::de::{Deserialize,Deserializer};
use serde::ser::{Serialize,Serializer};
use proxy::util::serde_str;
use std::num::ParseIntError;
use std::ops::{Deref,DerefMut};
use std::str::FromStr;
use std::fmt;


/// Unsigned 8-bit integer.
///
/// ```
/// extern crate ethrpc;
/// use ethrpc::types::Uint8;
///
/// let bytestr = "0x10";
/// 
/// let byte: Uint8 = bytestr.parse().unwrap();
/// 
/// // Hexadecimal string conversions
/// assert_eq!(byte.to_string(),bytestr);
/// assert_eq!(byte,Uint8::from(0x10u8));
/// assert!("0x100".parse::<Uint8>().is_err());
///
/// // Usabe as slice
/// fn use_as_bytes(_: &[u8]) {  }
/// use_as_bytes(&byte);
///
/// ```
///
#[derive(Hash,Default,Copy,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub struct Uint8(pub [u8;1]);

impl Serialize for Uint8 {

    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok,S::Error> {
        serde_str::serialize(self,serializer)
    }
}

impl<'de> Deserialize<'de> for Uint8 {

    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self,D::Error> {
        serde_str::deserialize(deserializer)
    }
}


impl Uint8 {

    pub fn into_inner(self) -> u8 { self.into() }

    pub fn into_other<T: From<u8>>(self) -> T { self.into_inner().into() }
}

impl Deref for Uint8 {

    type Target = [u8];

    fn deref(&self) -> &Self::Target { &self.0 }
}


impl DerefMut for Uint8 {

    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}


impl From<u8> for Uint8 {

    fn from(byte: u8) -> Self { Uint8([byte]) }
}

impl Into<u8> for Uint8 {

    fn into(self) -> u8 { self.0[0] }
}

impl AsRef<[u8]> for Uint8 {

    fn as_ref(&self) -> &[u8] { self.0.as_ref() }
}

impl AsMut<[u8]> for Uint8 {

    fn as_mut(&mut self) -> &mut [u8] { self.0.as_mut() }
}

impl fmt::Debug for Uint8 {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{:#x}",self.0[0])
    }
}

impl fmt::Display for Uint8 {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self,f)
    }
}


impl FromStr for Uint8 {

    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        let byte = u8::from_str_radix(s.trim_left_matches("0x"),16)?;
        Ok(Uint8([byte]))
    }
}

