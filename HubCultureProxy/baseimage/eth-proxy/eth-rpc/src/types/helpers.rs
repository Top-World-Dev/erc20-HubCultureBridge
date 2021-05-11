//! Misc helper types.
//!
use std::fmt;


/// Value which may be either singular or sequential
///
/// ## Example
///
/// ```
/// extern crate serde_json;
/// extern crate ethrpc;
///
/// use ethrpc::types::helpers::ValOrSeq;
///
/// # fn main() {
/// 
/// assert_eq!(ValOrSeq::Val(123),serde_json::from_str("123").unwrap());
///
/// assert_eq!(ValOrSeq::Seq(vec![456,789]),serde_json::from_str("[456,789]").unwrap());
///
/// # }
///
/// ```
///
#[derive(Hash,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(untagged)]
pub enum ValOrSeq<T> {
    Val(T),
    Seq(Vec<T>),
}

impl<T> ValOrSeq<T> {

    pub fn iter(&self) -> impl Iterator<Item=&T> {
        let base: &[T] = self.as_seq().unwrap_or(&[]);
        base.iter().chain(self.as_val())
    }

    pub fn into_iter(self) -> impl Iterator<Item=T> {
        match self {
            ValOrSeq::Val(val) => Some(val).into_iter().chain(Vec::new()),
            ValOrSeq::Seq(seq) => None.into_iter().chain(seq),
        }
    }

    pub fn compress(self) -> Option<Self> {
        match self {
            ValOrSeq::Val(v) => Some(ValOrSeq::Val(v)),
            ValOrSeq::Seq(mut s) => {
                match s.len() {
                    0 => None,
                    1 => s.pop().map(From::from),
                    _other => Some(ValOrSeq::Seq(s)),
                }
            }
        }
    }

    pub fn to_vec(self) -> Vec<T> {
        match self {
            ValOrSeq::Val(val) => vec![val],
            ValOrSeq::Seq(seq) => seq,
        }
    }

    fn as_val(&self) -> Option<&T> {
        match self {
            ValOrSeq::Val(v) => Some(v),
            _other => None,
        }
    }

    fn as_seq(&self) -> Option<&[T]> {
        match self {
            ValOrSeq::Seq(s) => Some(s),
            _other => None,
        }
    }
}


impl<T> fmt::Debug for ValOrSeq<T> where T: fmt::Debug {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValOrSeq::Val(val) => val.fmt(f),
            ValOrSeq::Seq(seq) => seq.fmt(f),
        }
    }
}


impl<T> From<T> for ValOrSeq<T> {

    fn from(val: T) -> Self { ValOrSeq::Val(val) }
}


impl<T> From<Vec<T>> for ValOrSeq<T> {

    fn from(seq: Vec<T>) -> Self { ValOrSeq::Seq(seq) }
}
