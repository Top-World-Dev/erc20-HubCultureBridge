use ethrpc::abi::{Function,Value,Token,EncodeError};
use types::Bytes;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::{fmt,error};


/// Description of a function call
///
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct Call {
    /// Name of the function to be called
    pub name: String,
    /// Inputs as a mapping from name to value
    #[serde(default,skip_serializing_if = "HashMap::is_empty")]
    pub inputs: HashMap<String,Value>,
}


/// A collection of function specifications
///
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Functions {
    inner: HashMap<String,Function>,
}


impl Functions {

    /// Attempt to encode a function call
    ///
    pub fn try_encode(&self, call: Call) -> Result<Bytes,Error> {
        let Call { name, mut inputs } = call;
        if let Some(function) = self.inner.get(&name) {
            let mut args = Vec::with_capacity(inputs.len());
            for (name,kind) in function.iter_inputs() {
                if let Some(value) = inputs.remove(name) {
                    args.push(value);
                } else {
                    return Err(Error::MissingArg {
                        name: name.to_owned(),
                        kind: kind,
                    });
                }
            }
            if let Some((name,value)) = inputs.into_iter().next() {
                Err(Error::UnknownArg { name, value })
            } else {
                let encoded = function.encode(&args)?;
                Ok(encoded)
            }
        } else {
            Err(Error::NoSuchFunction { name })
        }
    }

    pub fn get(&self, name: &str) -> Option<&Function> { self.inner.get(name) }

    pub fn iter(&self) -> impl Iterator<Item=(&str,&Function)> {
        self.inner.iter().map(|(name,function)| {
            (name.as_ref(),function)
        })
    }
}



impl FromIterator<Function> for Functions {

    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item=Function> {
        let inner = iter.into_iter().map(|function| {
            (function.name.clone(),function)
        }).collect();
        Self { inner }
    }
}

impl<'a> FromIterator<&'a Function> for Functions {

    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item=&'a Function> {
        iter.into_iter().cloned().collect()
    }
}



#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Error {
    Encode(EncodeError),
    MissingArg {
        name: String,
        kind: Token,
    },
    UnknownArg {
        name: String,
        value: Value,
    },
    NoSuchFunction {
        name: String,
    },
}


impl From<EncodeError> for Error {

    fn from(err: EncodeError) -> Self { Error::Encode(err) }
}


impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Encode(err) => err.fmt(f),
            Error::MissingArg { name, kind } => {
                write!(f,"Missing required argument `{}` ({})",name,kind)
            },
            Error::UnknownArg { name, value } => {
                write!(f,"Unexpected argument `{}` ({})",name,value)
            },
            Error::NoSuchFunction { name } => {
                write!(f,"Unable to locate function `{}`",name)
            }
        }
    }
}


impl error::Error for Error {

    fn description(&self) -> &str {
        match self {
            Error::Encode(err) => err.description(),
            Error::MissingArg { .. } => "missing required argument",
            Error::UnknownArg { .. } => "got unexpected argument",
            Error::NoSuchFunction { .. } => "function does not exist",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            Error::Encode(err) => Some(err),
            _other => None,
        }
    }
}

