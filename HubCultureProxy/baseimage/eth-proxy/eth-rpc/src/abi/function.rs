use smallvec::SmallVec;
use std::{fmt,error};

use abi::{self,Token,Value};
use types::Bytes;


/// A solidity function specification.
/// 
/// See module-level docs for example usage.
///
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct Function {
    pub name: String,
    pub inputs: Params,
    #[serde(default)]
    pub payable: bool,
}


impl Function {

    /// Get the selector for this function.
    ///
    pub fn selector(&self) -> Selector {
        let types = self.inputs.iter().map(|p| p.kind);
        let sighash = abi::signature(&self.name,types);
        let selector = {
            let mut buf = [0u8;4];
            buf.copy_from_slice(&sighash[..4]);
            Selector::from(buf)
        };
        debug!("calculated selector {} for function {}",selector,self.name);
        selector
    }

    /// Attempt to encode calldata using `args`.
    ///
    pub fn encode(&self, args: &[Value]) -> Result<Bytes,EncodeError> {
        if self.inputs.len() == args.len() {
            let selector = self.selector();
            let bufsize = selector.len() + (32 * args.len());
            let mut calldata = Vec::with_capacity(bufsize);
            calldata.extend_from_slice(selector.as_ref());
            for (index,((_,kind),arg)) in self.iter_inputs().zip(args).enumerate() {
                match arg.try_cast(kind) {
                    Ok(expected) => {
                        let word = expected.into_word();
                        calldata.extend_from_slice(&word);
                    },
                    Err(other) => {
                        return Err(EncodeError::ArgType {
                            expecting: kind,
                            got: other,
                            position: index,
                        });
                    }
                }
            }
            debug_assert!(calldata.len() == bufsize);
            Ok(Bytes::from(calldata))
        } else {
            Err(EncodeError::ArgCount {
                expecting: self.inputs.len(),
                got: args.len(),
            })
        }
    }


    /// Iterate across all function inputs.
    ///
    pub fn iter_inputs(&self) -> impl Iterator<Item=(&str,Token)> {
        self.inputs.iter().map(|param| {
            (param.name.as_ref(),param.kind)
        })
    }
}


impl From<(String,Params)> for Function {

    fn from(parts: (String,Params)) -> Self {
        let (name,inputs) = parts;
        Self { name, inputs, payable: false }
    }
}


/// Indicates failure to encode function arguments.
///
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum EncodeError {
    /// Argument count didn't match
    ArgCount {
        expecting: usize,
        got: usize,
    },
    /// Argument type didn't match
    ArgType {
        expecting: Token,
        got: Token,
        position: usize,
    },
}


impl fmt::Display for EncodeError {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EncodeError::ArgCount { expecting, got } => {
                write!(f,"Invalid arg count; expecting {} got {}",expecting,got)
            },
            EncodeError::ArgType { expecting, got, position } => {
                write!(f,"Invalid arg type; expecting {} got {} (position {})",expecting,got,position)
            },
        }
    }
}


impl error::Error for EncodeError {

    fn description(&self) -> &str {
        match self {
            EncodeError::ArgCount { .. } => "invalid argument count",
            EncodeError::ArgType { .. } => "invalid argument type",
        }
    }
}


/// Solidity function selector.
///
#[derive(Default,Copy,Clone,Hash,PartialEq,Eq,PartialOrd,Ord)]
pub struct Selector(pub [u8;4]);

newtype!(Selector,[u8;4],[u8;4]);

hex_array!(Selector,4);


impl fmt::Display for Selector {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self,f)
    }
}


///  A an ordered collection of function parameters.
///
pub type Params = SmallVec<[Param;4]>;


/// A named/typed function parameter.
///
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Param {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: Token, 
}
