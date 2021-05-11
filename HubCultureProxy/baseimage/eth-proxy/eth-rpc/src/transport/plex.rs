//! Spawn multiplexed services
//!
use tokio_channel::{mpsc,oneshot};
pub use tokio_util::Never;
use tokio::prelude::*;
use tokio;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::time::Duration;
use std::fmt;


pub trait Service<Req,Rsp>: Sink<SinkItem=(u64,Req)> + Stream<Item=(u64,Rsp)> {

    type Error: From<<Self as Sink>::SinkError> + From<<Self as Stream>::Error> + fmt::Display;
}


impl<T,Req,Rsp> Service<Req,Rsp> for T where 
        T: Sink<SinkItem=(u64,Req)> + Stream<Item=(u64,Rsp),Error = <T as Sink>::SinkError>,
        <T as Stream>::Error: fmt::Display, <T as Sink>::SinkError: fmt::Display {

    type Error = <Self as Stream>::Error;
}


pub fn spawn<Srv,Req,Rsp>(service: Srv) -> impl Future<Item=Handle<Req,Rsp>,Error=Never> where
        Srv: Service<Req,Rsp> + Send + 'static,
        Req: Send + 'static, Rsp: Send + 'static {
    future::lazy(|| Ok(spawn_sync(service)))
}


pub fn spawn_sync<Srv,Req,Rsp>(service: Srv) -> Handle<Req,Rsp> where
        Srv: Service<Req,Rsp> + Send + 'static,
        Req: Send + 'static, Rsp: Send + 'static {
    let (tx,rx) = mpsc::unbounded();
    let handle = Handle::new(tx);
    let plex = Plex::new(rx.map_err(drop)).sink_map_err(drop);
    let (p_tx,p_rx) = plex.split();
    let (s_tx,s_rx) = service.map_err(From::from)
        .sink_map_err(From::from)
        .map_err(|e: <Srv as Service<_,_>>::Error| error!("In service stream: {}",e))
        .sink_map_err(|e: <Srv as Service<_,_>>::Error| error!("In service sink: {}",e)).split();
    let fwd_req = p_rx.forward(s_tx).map(drop);
    let fwd_rsp = s_rx.forward(p_tx).map(drop);
    let work = fwd_req.select(fwd_rsp).then(|_| Ok(()));
    tokio::spawn(work);
    handle
}


#[derive(Debug)]
pub struct Call<Req,Rsp> {
    req: Req,
    tx: oneshot::Sender<Rsp>,
}


#[derive(Debug,Clone)]
pub struct TimeoutHandle<Req,Rsp> {
    inner: Handle<Req,Rsp>,
    timeout: Duration,
}


impl<Req,Rsp> TimeoutHandle<Req,Rsp> {

    pub fn new(inner: mpsc::UnboundedSender<Call<Req,Rsp>>) -> Self {
        let inner = Handle::new(inner);
        let timeout = Duration::from_secs(37);
        Self { inner, timeout }
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    pub fn call(&self, req: Req) -> impl Future<Item=Rsp,Error=&'static str> {
        self.inner.call(req).timeout(self.timeout).map_err(|e| {
            match e.into_inner() {
                Some(e) => e,
                None => "request failed; timeout exceeded"
            }
        })
    }
}


impl<Req,Rsp> From<Handle<Req,Rsp>> for TimeoutHandle<Req,Rsp> {

    fn from(handle: Handle<Req,Rsp>) -> Self {
        let Handle { inner } = handle;
        Self::new(inner)
    }
}



#[derive(Debug)]
pub struct Handle<Req,Rsp> {
    inner: mpsc::UnboundedSender<Call<Req,Rsp>>,
}


impl<Req,Rsp> Clone for Handle<Req,Rsp> {

    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        Self { inner }
    }
}


impl<Req,Rsp> Handle<Req,Rsp> {

    pub fn new(inner: mpsc::UnboundedSender<Call<Req,Rsp>>) -> Self {
        Self { inner }
    }
}


impl<Req,Rsp> Handle<Req,Rsp> {

    pub fn call(&self, req: Req) -> impl Future<Item=Rsp,Error=&'static str> {
        let (tx,rx) = oneshot::channel();
        let call = Call { req, tx };
        self.inner.unbounded_send(call).map_err(|_| "unable to enqueue request; recv handle dropped")
            .map(|_| rx.map_err(|_| "request failed; respose channel dropped")).into_future()
            .flatten()
    }
}


pub struct Plex<Inc,Req,Rsp> {
    incoming: Option<Inc>,
    plex: MultiPlex<Rsp>,
    _r: PhantomData<Req>,
}


impl<Inc,Req,Rsp> Plex<Inc,Req,Rsp> {

    pub fn new(incoming: Inc) -> Self {
        let incoming = Some(incoming);
        let plex = Default::default();
        let _r = PhantomData;
        Self { incoming, plex, _r }
    }
}


impl<Inc,Req,Rsp> Plex<Inc,Req,Rsp> where Inc: Stream {

    fn poll_incoming(&mut self) -> Poll<Option<Inc::Item>,Inc::Error> {
        if self.incoming.is_some() {
            match try_ready!(self.incoming.as_mut().unwrap().poll()) {
                Some(item) => Ok(Async::Ready(Some(item))),
                None => {
                    let _ = self.incoming.take();
                    Ok(Async::Ready(None))
                }
            }
        } else {
            Ok(Async::Ready(None))
        }
    }
}


impl<Inc,Req,Rsp> Stream for Plex<Inc,Req,Rsp> where Inc: Stream<Item=Call<Req,Rsp>> {

    type Item = (u64,Req);

    type Error = <Inc as Stream>::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>,Self::Error> {
        if let Some(Call { req, tx }) = try_ready!(self.poll_incoming()) {
            let id = self.plex.reserve_pending(tx);
            Ok(Async::Ready(Some((id,req))))
        } else {
            // Inner stream has terminated; ensure all pending `tx` handles are
            // resolved before actually terminating this stream (allows this stream
            // to function as proxy for graceful shutdown trigger).
            try_ready!(self.plex.poll_pending().map_err(|e|e.into()));
            Ok(Async::Ready(None))
        }
    }
}


impl<Inc,Req,Rsp> Sink for Plex<Inc,Req,Rsp>  {

    type SinkItem = (u64,Rsp);

    type SinkError = Never;

    fn start_send(&mut self, item: Self::SinkItem) -> Result<AsyncSink<Self::SinkItem>,Self::SinkError> {
        self.plex.start_send(item)
    }

    fn poll_complete(&mut self) -> Poll<(),Self::SinkError> {
        self.plex.poll_complete()
    }
}


#[derive(Debug)]
pub struct MultiPlex<T> {
    inner: HashMap<u64,oneshot::Sender<T>>,
    next: u64,
}


impl<T> Default for MultiPlex<T> {

    fn default() -> Self {
        let (inner,next) = Default::default();
        Self { inner, next }
    }
}

impl<T> MultiPlex<T> {


    /// Poll for pending `tx` handles.
    ///
    /// Removes cancelled `tx` handles & registers task for wake on future cancellations.
    /// If `Async::Ready(())` is returned, no pending handles exist.
    ///   
    pub fn poll_pending(&mut self) -> Poll<(),Never> {
        // Set up collector for cancellation ids
        let mut cancelled: SmallVec<[u64;32]> = smallvec![];
        // Poll inner `tx` handles, recording cancellations 
        for (id,tx) in self.inner.iter_mut() {
            if let Async::Ready(()) = tx.poll_cancel().expect("Sender::poll_cancel is infallible") {
                cancelled.push(*id);
            }
        }
        // Remove any cancelled `tx` handles
        for id in cancelled.into_iter() {
            self.inner.remove(&id);
        }
        // If no `tx` handles remain, then nothing is pending; if any do remain,
        // they must be pending since all cancellations have now been removed.
        if self.inner.is_empty() {
            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }

    /// Get next available id
    fn next_id(&mut self) -> u64 {
        let id: u64 = self.next;
        self.next = id.wrapping_add(1);
        debug_assert!(!self.inner.contains_key(&id));
        id
    }

    /// Reserve a new pending id/channel pair
    pub fn reserve_pending(&mut self, tx: oneshot::Sender<T>) -> u64 {
        let id = self.next_id();
        self.inner.insert(id,tx);
        id
    }

    /// Resolve an existing id/channel pair
    pub fn resolve_pending(&mut self, id: u64, value: T) {
        if let Some(sender) = self.inner.remove(&id) {
            if let Err(_) = sender.send(value) {
                warn!("Receiver dropped for id {}",id);
            }
        } else {
            warn!("No receiver found for id {}",id);
        }
    }
}


impl<T> Sink for MultiPlex<T> {

    type SinkItem = (u64,T);

    type SinkError = Never;

    fn start_send(&mut self, item: Self::SinkItem) -> Result<AsyncSink<Self::SinkItem>,Self::SinkError> {
        let (id,value) = item;
        self.resolve_pending(id,value);
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(),Self::SinkError> { Ok(Async::Ready(())) }
}

