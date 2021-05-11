//! Gas-price loading.
//!
use std::time::{Duration,Instant};
use ethrpc::types::U256;
use ethrpc::util::bufmath;
use ethrpc::{self,api,Url};
use tokio::prelude::*;
use std::fmt;
use util;


/// Indicates failure to load gas-price.
///
pub type Error = api::Error;


/// Initialze a price store.
///
pub fn store(node: Url) -> impl PriceStore<Error=Error> {
    debug!("Initializing price store with node {}",node);
    let loader = move || {
        let node = node.clone();
        util::retry(3,move || {
            ethrpc::connect(node.clone()).and_then(move |api| {
                api.eth().gas_price()
            })
        })
    };
    PriceCache::new(loader)
}


/// Caching gas-price lookup/storage service.
///
pub trait PriceStore {

    /// Indicates failure to load gas-price.
    ///
    type Error;

    /// Poll for current gas-price.
    ///
    fn poll_price(&mut self) -> Poll<U256,Self::Error>;

    /// Cancel pending work (if any).
    ///
    /// This function must be called if `poll_price` was called since the
    /// last time an item or error was returned, but the caller no longer
    /// cares about the result.  This ensures that future calls to `poll_price`
    /// will not get a stale result.
    ///
    fn cancel(&mut self);
}


impl<L> PriceStore for PriceCache<L,L::Future> where L: PriceLoader {

    type Error = <L as PriceLoader>::Error;

    fn poll_price(&mut self) -> Poll<U256,Self::Error> { self.poll_price() }

    fn cancel(&mut self) { self.cancel() }
}


trait PriceLoader {

    type Error: fmt::Display;

    type Future: Future<Item=U256,Error=Self::Error>;

    fn load_price(&self) -> Self::Future;
}


impl<T,F> PriceLoader for T where T: Fn() -> F, F: IntoFuture<Item=U256>, F::Error: fmt::Display {

    type Error = <F as IntoFuture>::Error;

    type Future = <F as IntoFuture>::Future;

    fn load_price(&self) -> Self::Future {
        (self)().into_future()
    }
}


struct PriceCache<L,F> {
    // Last time price was updated
    last_update: Instant,
    max_age: Duration,
    loader: L,
    current: Option<U256>,
    work: Option<F>,
}


impl<L,F> PriceCache<L,F> {

    pub fn new(loader: L) -> Self {
        let last_update = Instant::now();
        let max_age = Duration::from_secs(41);
        let (current,work) = Default::default();
        Self { last_update, max_age, loader, current, work }
    }

    pub fn cancel(&mut self) {
        let _ = self.work.take();
    }

    fn set_current(&mut self, price: U256) {
        self.last_update = Instant::now();
        self.current = Some(price);
    }

    fn get_current(&mut self) -> Option<U256> {
        if self.current.is_some() {
            debug_assert!(self.work.is_none(),"expired work must be cleared");
            let now = Instant::now();
            match self.current {
                Some(current) if self.last_update + self.max_age > now => {
                    Some(current)
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


impl<L> PriceCache<L,L::Future> where L: PriceLoader {

    fn poll_work(&mut self) -> Poll<U256,L::Error> {
        debug_assert!(self.current.is_none(),"expired price must be cleared");
        let Self { work, loader, .. } = self;
        let poll = work.get_or_insert_with(|| {
            loader.load_price()
        }).poll();
        let price = try_ready!(poll.map_err(|err| {
            let _ = work.take();
            warn!("Failed to load price: {}",err);
            err
        }));
        let _ = work.take();
        Ok(Async::Ready(price))
    }

    pub fn poll_price(&mut self) -> Poll<U256,L::Error> {
        if let Some(price) = self.get_current() {
            debug!("Yielding current price from cache {:?}",price);
            Ok(Async::Ready(price))
        } else {
            let mut price = try_ready!(self.poll_work());
            increment(&mut price);
            debug!("Yielding fresh price from node {:?}",price);
            self.set_current(price);
            Ok(Async::Ready(price))
        }
    }
}


impl<L> Stream for PriceCache<L,L::Future> where L: PriceLoader {

    type Item = U256;

    type Error = L::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>,Self::Error> {
        let price = try_ready!(self.poll_price());
        Ok(Async::Ready(Some(price)))
    }
}


fn increment(price: &mut U256) {
    let one = U256::from(1u64);
    let overflow = bufmath::add(price,&one);
    assert!(overflow == false,"256-bit integer overflow during increment");
}

