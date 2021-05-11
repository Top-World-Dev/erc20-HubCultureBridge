//! Nonce-calculation
//!
use std::time::{Duration,Instant};
use ethrpc::types::U256;
use ethrpc::crypto::Address;
use ethrpc::util::bufmath;
use ethrpc::{self,api,Url};
use tokio::prelude::*;
use std::fmt;
use util;

/// Indicates failure to load nonce.
///
pub type Error = api::Error;


/// Initialize a nonce store.
///
pub fn store(node: Url, addr: Address) -> impl NonceStore<Error=Error> {
    debug!("Initializing nonce store for {} with node {}",addr,node);
    let loader = move || {
        let node = node.clone();
        util::retry(3,move || {
            ethrpc::connect(node.clone()).and_then(move |api| {
                api.eth().get_tx_count(addr,Default::default())
            })
        })
    };
    NonceCache::new(loader)
}


/// Caching nonce lookup/storage service.
///
pub trait NonceStore {

    /// Indicates failure to load nonce.
    ///
    type Error;

    /// Poll for current nonce target.
    ///
    fn poll_nonce(&mut self) -> Poll<U256,Self::Error>;

    /// Increment nonce (if cached).
    ///
    /// This function must be called any time the yielded nonce is
    /// used in a broadcasted transaction (e.g. `eth_sendTransaction`
    /// or `eth_sendRawTransaction`).
    ///
    fn increment(&mut self);

    /// Cancel pending work (if any).
    ///
    /// This function must be called if `poll_nonce` was called since the
    /// last time an item or error was returned, but the caller no longer
    /// cares about the result.  This ensures that future calls to `poll_nonce`
    /// will not get a stale result.
    ///
    fn cancel(&mut self);
}


impl<L> NonceStore for NonceCache<L,L::Future> where L: NonceLoader {

    type Error = <L as NonceLoader>::Error;

    fn poll_nonce(&mut self) -> Poll<U256,Self::Error> { self.poll_nonce() }

    fn increment(&mut self) { self.increment() }

    fn cancel(&mut self) { self.cancel() }
}


trait NonceLoader {

    type Error: fmt::Display;

    type Future: Future<Item=U256,Error=Self::Error>;

    fn load_nonce(&self) -> Self::Future;
}


impl<T,F> NonceLoader for T where T: Fn() -> F, F: IntoFuture<Item=U256>, F::Error: fmt::Display {

    type Error = <F as IntoFuture>::Error;

    type Future = <F as IntoFuture>::Future;

    fn load_nonce(&self) -> Self::Future {
        (self)().into_future()
    }
}


struct NonceCache<L,F> {
    // Last time nonce was used
    last_use: Instant,
    max_age: Duration,
    loader: L,
    current: Option<U256>,
    work: Option<F>,
}


impl<L,F> NonceCache<L,F> {

    pub fn new(loader: L) -> Self {
        let last_use = Instant::now();
        let max_age = Duration::from_secs(127);
        let (current,work) = Default::default();
        Self { last_use, max_age, loader, current, work }
    }

    pub fn increment(&mut self) {
        if let Some(nonce) = self.current.as_mut() {
            self.last_use = Instant::now();
            increment(nonce);
        }
    }

    pub fn cancel(&mut self) {
        let _ = self.work.take();
    }

    fn set_current(&mut self, nonce: U256) {
        if let Some(previous) = self.current.take() {
            if previous > nonce {
                warn!("Overwriting current nonce with lower value ({:?} => {:?})",previous,nonce);
            }
        }
        self.last_use = Instant::now();
        self.current = Some(nonce);
    }

    fn get_current(&mut self) -> Option<U256> {
        if self.current.is_some() {
            debug_assert!(self.work.is_none(),"expired work must be cleared");
            let now = Instant::now();
            match self.current {
                Some(nonce) if self.last_use + self.max_age > now => {
                    Some(nonce)
                },
                _ => {
                    let _ = self.current.take();
                    None
                }
            }
        } else {
            None
        }
    }
}


impl<L> NonceCache<L,L::Future> where L: NonceLoader {

    fn poll_work(&mut self) -> Poll<U256,L::Error> {
        debug_assert!(self.current.is_none(),"expired nonce must be cleared");
        let Self { work, loader, .. } = self; 
        let poll = work.get_or_insert_with(|| {
            loader.load_nonce()
        }).poll();
        let nonce = try_ready!(poll.map_err(|err| {
            let _ = work.take();
            warn!("Failed to load nonce: {}",err);
            err
        }));
        let _ = work.take();
        Ok(Async::Ready(nonce))
    }

    pub fn poll_nonce(&mut self) -> Poll<U256,L::Error> {
        if let Some(nonce) = self.get_current() {
            debug!("Yielding current nonce from cache {:?}",nonce);
            Ok(Async::Ready(nonce))
        } else {
            let nonce = try_ready!(self.poll_work());
            debug!("Yielding fresh nonce from node {:?}",nonce);
            self.set_current(nonce);
            Ok(Async::Ready(nonce))
        }
    }
}


impl<L> Stream for NonceCache<L,L::Future> where L: NonceLoader {

    type Item = U256;

    type Error = L::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>,Self::Error> {
        let nonce = try_ready!(self.poll_nonce());
        self.increment();
        Ok(Async::Ready(Some(nonce)))
    }
}


fn increment(nonce: &mut U256) {
    let one = U256::from(1u64);
    let overflow = bufmath::add(nonce,&one);
    assert!(overflow == false,"256-bit integer overflow during increment");
}

