//! Specialized transaction-signing handler.
//!
use transact::nonce::{self,NonceStore};
use transact::price::{self,PriceStore};
use base::Error as SignerError;
use base::BaseSigner;
use tokio_channel::{mpsc,oneshot};
use rpc::TxRequest;
use tokio::prelude::*;
use tokio;
use ethrpc::types::{Never,Bytes,H256,U256};
use ethrpc::{self,api,Url};
use std::{fmt,error};
use util;


/// Spawn handler to event-loop.
///
pub fn spawn<S>(signer: S, node: Url) -> impl Future<Item=impl TxHandler<Error=Error<S::Error>> + Clone,Error=Error<S::Error>>
        where S: BaseSigner + Send + 'static {
    init_base_handler(signer,node).map(|base| {
        let handle = spawn_as_remote(base);
        let tx_handler = move |req| { handle.call(req) };
        tx_handler
    })
}


/// Sets up transaction sender.
///
fn tx_sender(node: Url) -> impl TxSender<Error=api::Error> {
    let sender = move |bytes: Bytes| {
        let node = node.clone();
        util::retry(3,move || {
            let bytes = bytes.clone();
            ethrpc::connect(node.clone()).and_then(move |api| {
                api.eth().send_raw_tx(bytes)
            })
        })
    };
    sender
}


/// Sets up basic handler instance.
///
fn init_base_handler<S: BaseSigner>(signer: S, node: Url) -> impl Future<Item=impl BaseHandler<Error=Error<S::Error>>,Error=Error<S::Error>> {
    let work = signer.api().get_address().from_err().map(move |addr| {
        let tx_sender = tx_sender(node.clone());
        let nonce_store = nonce::store(node.clone(),addr);
        let price_store = price::store(node);
        Handler::new(signer,tx_sender,nonce_store,price_store)
    });
    work
}


/// Sets up remote handler instance.
///
fn spawn_as_remote<T>(handler: T) -> RemoteHandle<T::Error>
        where T: BaseHandler + Send + 'static, T::Error: Send + 'static {
    let remote = RemoteHandler::new(handler);
    let (tx,rx) = mpsc::unbounded();
    let work = rx.forward(remote.sink_map_err(|e| e.into::<()>()));
    tokio::spawn(work.map(drop));
    RemoteHandle::new(tx)
}


/// Wraps `TxRequest` for handling by dedicated task.
///
#[derive(Debug)]
struct RemoteRequest<E> {
    req: TxRequest,
    rsp: oneshot::Sender<Result<H256,E>>,
}


/// Handle to a dedicated transaction-handling task.
///
#[derive(Debug)]
struct RemoteHandle<E> {
    inner: mpsc::UnboundedSender<RemoteRequest<E>>,
}


impl<E> Clone for RemoteHandle<E> {

    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        Self { inner }
    }
}

impl<E> RemoteHandle<E> {

    fn new(inner: mpsc::UnboundedSender<RemoteRequest<E>>) -> Self {
        Self { inner }
    }
}

impl<E> RemoteHandle<E> where E: From<oneshot::Canceled> + From<mpsc::SendError<RemoteRequest<E>>> {

    fn call(&self, request: TxRequest) -> impl Future<Item=H256,Error=E> {
        let (tx,rx) = oneshot::channel();
        let remote_req = RemoteRequest { req: request, rsp: tx };
        self.inner.unbounded_send(remote_req).into_future().from_err::<E>()
            .and_then(move |()| rx.from_err())
            .flatten()
    }
}


/// Wraps base transaction-handler, allowing it to runs in a separate task.
///
/// This handler implements `Sink` and is intended to act as the consumer of
/// a request queue; responses are returned via the channel provided by the
/// request.
///
struct RemoteHandler<T,E> {
    /// Inner handler instance
    inner: T,
    /// Response channel for pending result
    rsp: Option<oneshot::Sender<Result<H256,E>>>,
}


impl<T,E> RemoteHandler<T,E> {

    fn new(inner: T) -> Self {
        let rsp = Default::default();
        Self { inner, rsp }
    }
}


impl<T> RemoteHandler<T,T::Error> where T: BaseHandler {

    /// Start processing a request.
    ///
    fn start_work(&mut self, request: RemoteRequest<T::Error>) -> Async<()> {
        debug_assert!(
            self.rsp.is_none() && !self.inner.is_working(),
            "cannot start new job while another is pending"
            );
        let RemoteRequest { req, rsp } = request;
        self.inner.push_work(req);
        self.rsp = Some(rsp);
        self.poll_work()
    }

    /// Drive existing request-handling work to completion.
    ///
    fn poll_work(&mut self) -> Async<()> {
        if self.inner.is_working() {
            let result = match self.inner.poll_work() {
                Ok(Async::Ready(tx_hash)) => Ok(tx_hash),
                Ok(Async::NotReady) => { return Async::NotReady; },
                Err(err) => Err(err),
            };
            let _ = self.rsp.take().expect("response channel must exist for pending work")
                .send(result);
            Async::Ready(())
        } else {
            debug_assert!(self.rsp.is_none(),"response channel must not be orphaned");
            Async::Ready(())
        }
    }
}


impl<T> Sink for RemoteHandler<T,T::Error> where T: BaseHandler {

    type SinkItem = RemoteRequest<T::Error>;

    type SinkError = Never;

    fn start_send(&mut self, item: Self::SinkItem) -> Result<AsyncSink<Self::SinkItem>,Self::SinkError> {
        match self.poll_work() {
            Async::Ready(()) => {
                let _ = self.start_work(item);
                Ok(AsyncSink::Ready)
            },
            Async::NotReady => Ok(AsyncSink::NotReady(item)),
        }
    }

    fn poll_complete(&mut self) -> Poll<(),Self::SinkError> {
        Ok(self.poll_work())
    }
}

/// Specialized signer for handling transaction-generating requests.
///
pub trait TxHandler {

    /// Error produced by this handler.
    type Error: Send + 'static;

    /// Type of future yielded by this handler.
    type Future: Future<Item=H256,Error=Self::Error> + Send + 'static;

    /// Handle a transaction-generating request.
    fn handle_tx(&self, req: TxRequest) -> Self::Future;
}


impl<T,F> TxHandler for T
        where T: Fn(TxRequest) -> F, F: IntoFuture<Item=H256>,
              F::Future: Send + 'static, F::Error: Send + 'static {

    type Error = <F as IntoFuture>::Error;

    type Future = <F as IntoFuture>::Future;

    fn handle_tx(&self, req: TxRequest) -> Self::Future {
        (self)(req).into_future()
    }
}


/// Service responsible for sending signed transactions to the
/// blockchain.
///
/// *NOTE*: The existence of this trait is a workaround to allow
/// convenient usage of functions which return `impl Future` in contexts
/// where the associated future will be held inside of struct/enum bodies.
///
trait TxSender {

    type Error;

    type Future: Future<Item=H256,Error=Self::Error> + Send + 'static;

    fn send_raw(&self, tx: Bytes) -> Self::Future;
}


impl<T,F> TxSender for T where T: Fn(Bytes) -> F, F: IntoFuture<Item=H256>, F::Future: Send + 'static {

    type Error = <F as IntoFuture>::Error;

    type Future = <F as IntoFuture>::Future;

    fn send_raw(&self, tx: Bytes) -> Self::Future {
        (self)(tx).into_future()
    }
}


/// Base transaction handler.
///
/// Implementations are allowed to panic if `push_work` is called
/// while `is_working` yields `true`, *or* if `poll_work` is
/// called while `is_working` yields `false`.
///
trait BaseHandler {

    type Error;

    fn push_work(&mut self, req: TxRequest);

    fn poll_work(&mut self) -> Poll<H256,Self::Error>;

    fn is_working(&self) -> bool;
}


impl<S,T,N,P> BaseHandler for Handler<S,T,N,P,S::Future,T::Future>
        where S: BaseSigner, T: TxSender, N: NonceStore, P: PriceStore {

    type Error = Error<S::Error,T::Error,N::Error,P::Error>;

    fn push_work(&mut self, req: TxRequest) { self.push_work(req) }

    fn poll_work(&mut self) -> Poll<H256,Self::Error> { self.poll_work() }

    fn is_working(&self) -> bool { self.work.is_some() }
}


/// Core state-machine for handling transaction-generating requests.
///
/// *NOTE*: The excessive use of generics is, for the most part, a
/// workaround to allow convenient usage of methods which return
/// `impl Future`.
///
struct Handler<S,T,N,P,A,B> {
    signer: S,
    tx_sender: T,
    nonce_store: N,
    price_store: P,
    work: Option<Work<A,B>>,
}


impl<S,T,N,P,A,B> Handler<S,T,N,P,A,B> {

    pub fn new(signer: S, tx_sender: T, nonce_store: N, price_store: P) -> Self {
        let work = Default::default();
        Self { signer, tx_sender, nonce_store, price_store, work }
    }
}


impl<S,T,N,P> Handler<S,T,N,P,S::Future,T::Future>
        where S: BaseSigner, T: TxSender, N: NonceStore, P: PriceStore {


    pub fn cancel(&mut self) {
        let _ = self.work.take();
        self.nonce_store.cancel();
        self.price_store.cancel();
    }

    /// Push a new request for processing.
    ///
    pub fn push_work(&mut self, req: TxRequest) {
        debug_assert!(self.work.is_none(),"must not push new work while job is pending");
        let job = Work::SeedTx { tx: Some(req), nonce: None, price: None };
        self.work = Some(job);
    }

    /// Drive current request-process work to completion.
    ///
    pub fn poll_work(&mut self) -> Poll<H256,Error<S::Error,T::Error,N::Error,P::Error>> {
        match self.poll_work_inner() {
            Ok(Async::Ready(tx_hash)) => {
                self.cancel();
                Ok(Async::Ready(tx_hash))
            },
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(err) => {
                self.cancel();
                Err(err)
            },
        }
    }

    /// Core state-machine logic for driving request-processing.
    fn poll_work_inner(&mut self) -> Poll<H256,Error<S::Error,T::Error,N::Error,P::Error>> {
        loop {
            let next_step = match self.work.as_mut().expect("cannot poll empty work cache") {
                // Drive nonce & gas-price loading ops to completions and begin
                // sigining process on success.
                Work::SeedTx { ref mut tx, ref mut nonce, ref mut price } => {
                    if nonce.is_none() {
                        match self.nonce_store.poll_nonce().map_err(Error::nonce)? {
                            Async::Ready(n) => { *nonce = Some(n); },
                            Async::NotReady => { }
                        }
                    }
                    if price.is_none() {
                        match self.price_store.poll_price().map_err(Error::price)? {
                            Async::Ready(p) => { *price = Some(p); },
                            Async::NotReady => { }
                        }
                    }
                    match (nonce,price) {
                        (Some(nonce),Some(price)) => {
                            let tx_request = tx.take().expect("tx request must exist");
                            let seeded_request = tx_request.seed(*nonce,*price);
                            let work = self.signer.call(seeded_request);
                            Work::SignTx { work }
                        },
                        _ => { return Ok(Async::NotReady); }
                    }
                },
                // Drive tx signing to completion and begin tx-submission
                // process on success.
                Work::SignTx { ref mut work } => {
                    let response = try_ready!(work.poll().map_err(Error::signer));
                    let encoded = response.to_bytes().map_err(|e| e.into())
                        .map_err(SignerError::expecting_bytes)?;
                    let work = self.tx_sender.send_raw(encoded);
                    Work::SendTx { work }
                },
                // Drive tx-submission to completion and increment nonce
                // cache on success.
                Work::SendTx { ref mut work } => {
                    let tx_hash = try_ready!(work.poll().map_err(Error::tx_sender));
                    self.nonce_store.increment();
                    return Ok(Async::Ready(tx_hash));
                },
            };
            self.work = Some(next_step);
        }
    }
}


/// Stores the current state of request-processing.
/// 
enum Work<S,T> {
    SeedTx {
        tx: Option<TxRequest>,
        nonce: Option<U256>,
        price: Option<U256>,
    },
    SignTx { work: S },
    SendTx { work: T },
}


/// Indicates failure to serve a transaction-generating request.
///
#[derive(Debug)]
pub enum Error<S,T=api::Error,N=nonce::Error,P=price::Error> {
    /// Singning failed
    Signer(SignerError<S>),
    /// Transaction submission failed
    TxSender(T),
    /// Nonce lookup failed
    Nonce(N),
    /// Gas-price lookup failed
    Price(P),
    /// Response channel canceled
    Canceled,
    /// Tx-handling task dropped
    Dropped,
}


impl<S,T,N,P> From<SignerError<S>> for Error<S,T,N,P> {

    fn from(err: SignerError<S>) -> Self { Error::Signer(err) }
}

impl<S,T,N,P> Error<S,T,N,P> {

    fn signer(err: S) -> Self { Error::Signer(err.into()) }

    fn tx_sender(err: T) -> Self { Error::TxSender(err) }

    fn nonce(err: N) -> Self { Error::Nonce(err) }

    fn price(err: P) -> Self { Error::Price(err) }
}


impl<S,T,N,P> From<oneshot::Canceled> for Error<S,T,N,P> {

    fn from(_: oneshot::Canceled) -> Self { Error::Canceled }
}


impl<S,T,N,P,I> From<mpsc::SendError<I>> for Error<S,T,N,P> {

    fn from(_: mpsc::SendError<I>) -> Self { Error::Dropped }
}


impl<S,T,N,P> fmt::Display for Error<S,T,N,P>
        where S: fmt::Display, T: fmt::Display, N: fmt::Display, P: fmt::Display {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Signer(err) => err.fmt(f),
            Error::TxSender(err) => err.fmt(f),
            Error::Nonce(err) => err.fmt(f),
            Error::Price(err) => err.fmt(f),
            Error::Canceled => f.write_str("response channel canceled"),
            Error::Dropped => f.write_str("tx-handler task dropped"),
        }
    }
}


impl<S,T,N,P> error::Error for Error<S,T,N,P>
        where S: error::Error, T: error::Error, N: error::Error, P: error::Error {

    fn description(&self) -> &str {
        match self {
            Error::Signer(err) => err.description(),
            Error::TxSender(err) => err.description(),
            Error::Nonce(err) => err.description(),
            Error::Price(err) => err.description(),
            Error::Canceled => "response channel canceled",
            Error::Dropped => "tx-handler task dropped",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            Error::Signer(err) => Some(err),
            Error::TxSender(err) => Some(err),
            Error::Nonce(err) => Some(err),
            Error::Price(err) => Some(err),
            Error::Canceled => None,
            Error::Dropped => None,
        }
    }
}

