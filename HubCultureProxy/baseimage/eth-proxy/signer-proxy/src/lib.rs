extern crate signer;
extern crate ethrpc;
extern crate proxy;
extern crate tokio_channel;
extern crate tokio;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate log;

macro_rules! try_ready {
    ($e:expr) => (match $e {
        Ok(Async::Ready(t)) => t,
        Ok(Async::NotReady) => return Ok(Async::NotReady),
        Err(e) => return Err(From::from(e)),
    })
}
pub mod extension;
pub mod transact;
pub mod base;
pub mod options;
pub mod util;
pub mod rpc;

use tokio::prelude::*;
use signer::rpc::Error as SignerError;
use signer::Error as SetupError;
use rpc::{Request,Response};
use extension::ExtHandler;
use transact::TxHandler;
use options::SignerProxyOptions;
use ethrpc::Url;
use base::BaseSigner;
use std::{fmt,error};


pub type BoxHandler<E> = Box<RequestHandler<Future=Box<Future<Item=Response,Error=Error<E>> + Send>,Error=Error<E>> + Send>;

pub fn spawn_local(opt: &SignerProxyOptions) -> Result<Box<Future<Item=BoxHandler<SignerError>,Error=Error<SignerError>> + Send>,SetupError> {
    let local_signer = base::configure_local(&opt.signer)?;
    let node = opt.node_addr.clone();
    let work = Box::new(spawn(local_signer,node));
    Ok(work)
}


/// Spawn a `RequestHandler` instance.
///
pub fn spawn<S>(signer: S, node: Url) -> impl Future<Item=BoxHandler<S::Error>,Error=Error<S::Error>>
        where S: BaseSigner + Clone + Send + 'static {
    let work = transact::spawn(signer.clone(),node.clone()).map_err(Error::tx)
        .map(move |tx_handler| -> BoxHandler<S::Error> {
            let ext_handler = extension::spawn(node);
            Box::new(ProxySigner {
                base_handler: signer,
                tx_handler,
                ext_handler,
            })
        });
    work
}


/// Top-level request handler.
///
pub trait RequestHandler {

    type Error: Send + 'static;

    type Future: Future<Item=Response,Error=Self::Error> + Send + 'static;

    fn handle_request(&self, req: Request) -> Self::Future;
}


impl<B,T,E> RequestHandler for ProxySigner<B,T,E> where B: BaseSigner, T: TxHandler, E: ExtHandler {

    type Error = Error<B::Error,T::Error,E::Error>;

    type Future = Box<Future<Item=Response,Error=Self::Error> + Send>;

    fn handle_request(&self, req: Request) -> Self::Future {
        Box::new(self.handle_request(req))
    }
}


/*
impl<B,T,E> RequestHandler for ProxySigner<B,T,E> where B: BaseSigner, T: TxHandler, E: ExtHandler {

    type Error = Error<B::Error,T::Error,E::Error>;

    type Future = ProxyFuture<B::Future,T::Future,E::Future>;

    fn handle_request(&self, req: Request) -> ProxyFuture<B::Future,T::Future,E::Future> {
        self.handle_request(req)
    }
}
*/

#[derive(Debug,Clone)]
pub struct ProxySigner<B,T,E> {
    base_handler: B,
    tx_handler: T,
    ext_handler: E,
}



impl<B,T,E> ProxySigner<B,T,E> where B: BaseSigner, T: TxHandler, E: ExtHandler {

    pub fn handle_request(&self, req: Request) -> ProxyFuture<B::Future,T::Future,E::Future> {
        match req {
            Request::Call(req) => ProxyFuture::Base(self.base_handler.call(req.into_base())),
            Request::Transact(req) => ProxyFuture::Tx(self.tx_handler.handle_tx(req)),
            Request::Ext(req) => ProxyFuture::Ext(self.ext_handler.handle_ext(req)),
        }
    }
}


#[derive(Debug)]
pub enum ProxyFuture<B,T,E> {
    Base(B),
    Tx(T),
    Ext(E),
}



impl<B,T,E> Future for ProxyFuture<B,T,E>
        where B: Future, T: Future, E: Future,
              Response: From<B::Item> + From<T::Item> + From<E::Item> {

    type Item = Response;

    type Error = Error<B::Error,T::Error,E::Error>;

    fn poll(&mut self) -> Poll<Self::Item,Self::Error> {
        match self {
            ProxyFuture::Base(work) => {
                let item = try_ready!(work.poll().map_err(Error::base));
                Ok(Async::Ready(item.into()))
            },
            ProxyFuture::Tx(work) => {
                let item = try_ready!(work.poll().map_err(Error::tx));
                Ok(Async::Ready(item.into()))
            },
            ProxyFuture::Ext(work) => {
                let item = try_ready!(work.poll().map_err(Error::ext));
                Ok(Async::Ready(item.into()))
            },
        }
    }
}


#[derive(Debug)]
pub enum Error<B,T=transact::Error<B>,E=extension::Error> {
    Base(B),
    Tx(T),
    Ext(E),
}


impl<B,T,E> Error<B,T,E> {

    fn base(err: B) -> Self { Error::Base(err) }

    fn tx(err: T) -> Self { Error::Tx(err) }

    fn ext(err: E) -> Self { Error::Ext(err) }
}

impl<B,T,E> fmt::Display for Error<B,T,E>
        where E: fmt::Display, T: fmt::Display, B: fmt::Display {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Ext(err) => err.fmt(f),
            Error::Tx(err) => err.fmt(f),
            Error::Base(err) => err.fmt(f),
        }
    }
}


impl<B,T,E> error::Error for Error<B,T,E>
        where E: error::Error, T: error::Error, B: error::Error {

    fn description(&self) -> &str {
        match self {
            Error::Ext(err) => err.description(),
            Error::Tx(err) => err.description(),
            Error::Base(err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            Error::Ext(err) => Some(err),
            Error::Tx(err) => Some(err),
            Error::Base(err) => Some(err),
        }
    }
}
