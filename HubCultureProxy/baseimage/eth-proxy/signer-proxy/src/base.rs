use tokio::prelude::*;
use signer::options::SignerOptions;
use signer::{self,rpc};
use ethrpc::crypto::Address;
use rpc::{
    BaseRequest,
    BaseResponse,
};
use std::{fmt,error};

/// Configure local signer instance.
///
pub fn configure_local(opt: &SignerOptions) -> Result<impl BaseSigner<Error=rpc::Error> + Clone,signer::Error> {
    let local_signer = signer::Signer::from_options(opt)?;
    Ok(move |req| local_signer.serve(req))
}


pub trait BaseSigner {

    /// Error-type of the signer instance
    type Error: Send + 'static;

    /// Future yielded by this signer.
    type Future: Future<Item=BaseResponse,Error=Self::Error> + Send + 'static;

    /// Call signer with a request.
    fn call(&self, req: BaseRequest) -> Self::Future;

    /// Access signer api helper.
    fn api(&self) -> Api<Self> { Api::new(&self) }
}


impl<T,F> BaseSigner for T where T: Fn(BaseRequest) -> F, F: IntoFuture<Item=BaseResponse>,
        F::Error: Send + 'static, F::Future: Send + 'static {

    type Error = <F as IntoFuture>::Error;

    type Future = <F as IntoFuture>::Future;

    fn call(&self, req: BaseRequest) -> Self::Future {
        (self)(req).into_future()
    }
}


/// Helper namespace for calling singer apis.
///
pub struct Api<'a,S: 'a + ?Sized> {
    signer: &'a S
}

impl<'a,S> Api<'a,S> where S: 'a + ?Sized {

    fn new(signer: &'a S) -> Self { Self { signer } }
}


impl<'a,S> Api<'a,S> where S: BaseSigner {

    // TODO: implement all signer methods.

    /// Get current signer address.
    ///
    pub fn get_address(&self) -> impl Future<Item=Address,Error=Error<S::Error>> {
        let req = BaseRequest::GetAddress { };
        self.signer.call(req).from_err().and_then(|rsp| {
            rsp.to_addr().map_err(Error::expecting_addr)
        })
    }
}


/// Token indicating expected response type
///
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum Token {
    Address,
    Hash,
    Signature,
    Bytes,
}


impl Token {

    fn as_str(&self) -> &'static str {
        match self {
            Token::Address => "address",
            Token::Hash => "bytes32",
            Token::Signature => "signature",
            Token::Bytes => "bytes",
        }
    }
}


impl fmt::Display for Token {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}


#[derive(Debug,Clone)]
pub enum Error<E> {
    /// Signer returned error
    Signer(E),
    /// Signer returned unexpected response
    Unexpected {
        expecting: Token,
        got: BaseResponse
    }
}


impl<E> Error<E> {

    pub fn expecting_addr(other: BaseResponse) -> Self {
        Error::Unexpected {
            expecting: Token::Address,
            got: other,
        }
    }

    pub fn expecting_hash(other: BaseResponse) -> Self {
        Error::Unexpected {
            expecting: Token::Hash,
            got: other,
        }
    }

    pub fn expecting_sig(other: BaseResponse) -> Self {
        Error::Unexpected {
            expecting: Token::Signature,
            got: other,
        }
    }

    pub fn expecting_bytes(other: BaseResponse) -> Self {
        Error::Unexpected {
            expecting: Token::Bytes,
            got: other,
        }
    }
}


impl<E> From<E> for Error<E> {

    fn from(err: E) -> Self { Error::Signer(err) }
}

impl<E> fmt::Display for Error<E> where E: fmt::Display {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Signer(err) => err.fmt(f),
            Error::Unexpected { expecting, got } => {
                write!(f,"unexpected response type (expecting {}, got {:?})",expecting,got)
            }
        }
    }
}


impl<E> error::Error for Error<E> where E: error::Error {

    fn description(&self) -> &str {
        match self {
            Error::Signer(err) => err.description(),
            Error::Unexpected { .. } => "unexpected response type",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            Error::Signer(err) => Some(err),
            Error::Unexpected { .. } => None,
        }
    }
}
