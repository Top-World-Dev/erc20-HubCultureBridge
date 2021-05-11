use ethrpc::abi::{self,Value,Token};
use types::Bytes;
use functions;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::{fmt,error};


/// Request for token signing
pub type Request = functions::Call;

/// Token parameters
pub type Params = abi::FunctionParams;


/// Packed token specification
///
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct EthToken {
    /// Name of token
    pub name: String,
    /// Token inputs
    pub inputs: Params,
}


impl EthToken {

    /// Iterate across all function inputs.
    ///
    pub fn iter_inputs(&self) -> impl Iterator<Item=(&str,Token)> {
        self.inputs.iter().map(|param| {
            (param.name.as_ref(),param.kind)
        })
    }
}

/// A collection of function specifications
///
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct EthTokens {
    inner: HashMap<String,EthToken>,
}


impl EthTokens {

    /// Attempt to encode an ethtoken request
    ///
    pub fn try_encode(&self, request: Request) -> Result<Bytes,Error> {
        let Request { name, mut inputs } = request;
        if let Some(ethtoken) = self.inner.get(&name) {
            let mut args = Vec::with_capacity(inputs.len());
            for (index,(name,kind)) in ethtoken.iter_inputs().enumerate() {
                if let Some(value) = inputs.remove(name) {
                    match value.try_cast(kind) {
                        Ok(expected) => args.push(expected),
                        Err(other) => {
                            return Err(Error::WrongType {
                                expecting: kind,
                                got: other,
                                position: index
                            });
                        },
                    }
                } else {
                    return Err(Error::MissingVal {
                        name: name.to_owned(),
                        kind: kind,
                    });
                }
            }
            if let Some((name,value)) = inputs.into_iter().next() {
                Err(Error::UnknownVal { name, value })
            } else {
                let encoded = abi::packed(&args);
                Ok(encoded)
            }
        } else {
            Err(Error::NoSuchToken { name })
        }
    }

    pub fn get(&self, name: &str) -> Option<&EthToken> { self.inner.get(name) }

    pub fn iter(&self) -> impl Iterator<Item=(&str,&EthToken)> {
        self.inner.iter().map(|(name,ethtoken)| {
            (name.as_ref(),ethtoken)
        })
    }
}



impl FromIterator<EthToken> for EthTokens {

    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item=EthToken> {
        let inner = iter.into_iter().map(|ethtoken| {
            (ethtoken.name.clone(),ethtoken)
        }).collect();
        Self { inner }
    }
}

impl<'a> FromIterator<&'a EthToken> for EthTokens {

    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item=&'a EthToken> {
        iter.into_iter().cloned().collect()
    }
}



#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Error {
    WrongType {
        expecting: Token,
        got: Token,
        position: usize,
    },
    MissingVal {
        name: String,
        kind: Token,
    },
    UnknownVal {
        name: String,
        value: Value,
    },
    NoSuchToken {
        name: String,
    },
}


impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::WrongType { expecting, got, position } => {
                write!(f,"Invalid type; expecting {} got {} (position {})",expecting,got,position)
            },
            Error::MissingVal { name, kind } => {
                write!(f,"Missing required value `{}` ({})",name,kind)
            },
            Error::UnknownVal { name, value } => {
                write!(f,"Unexpected value `{}` ({})",name,value)
            },
            Error::NoSuchToken { name } => {
                write!(f,"Unable to locate token spec `{}`",name)
            }
        }
    }
}


impl error::Error for Error {

    fn description(&self) -> &str {
        match self {
            Error::WrongType { .. } => "invalid value type",
            Error::MissingVal { .. } => "missing required value",
            Error::UnknownVal { .. } => "got unexpected value",
            Error::NoSuchToken { .. } => "token spec does not exist",
        }
    }
}

