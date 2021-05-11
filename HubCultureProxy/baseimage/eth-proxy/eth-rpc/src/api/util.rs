use types::{BlockId,Block,U256,H256,Filter,Log};
use api::{Request,Response,Error,Api};
use util::bufmath;
use rpc;
use std::time::{Duration,Instant};
use tokio::prelude::*;
use tokio::timer;


/// Extra utility namespace.
///
pub struct Util<'a,T: 'a> {
    transport: &'a T,
}


impl<'a,T> Util<'a,T> {

    pub fn new(transport: &'a T) -> Self { Self { transport } }
}


impl<'a,T> Util<'a,T> where T: rpc::Transport<Request,Response> {


    /// Wait for a specified block to exist.
    ///
    pub fn await_block(&self, block: BlockId, poll: Duration) -> impl Future<Item=Block<H256>,Error=Error> {
        future::loop_fn((Instant::now(),self.api()), move |(time,api)| {
            timer::Delay::new(time).from_err().and_then(move |()| {
                api.eth().get_block_by_number(block).map(move |block| {
                    match block {
                        Some(block) => future::Loop::Break(block),
                        None => future::Loop::Continue((time + poll,api)),
                    }
                })
            })
        })
    }

    /// Future which waits until a block number is reached.
    ///
    pub fn await_block_number(&self, number: U256, poll: Duration) -> impl Future<Item=(),Error=Error> {
        future::loop_fn((Instant::now(),self.api()), move |(time,api)| {
            timer::Delay::new(time).from_err().and_then(move |()| {
                api.eth().block_number().map(move |current| {
                    if current >= number {
                        future::Loop::Break(())
                    } else {
                        future::Loop::Continue((time + poll,api))
                    }
                })
            })
        })
    }


    /// Stream blocks in order.
    /// 
    /// Stream may be configured to lag by up to 255 blocks; if so, block `n` will not
    /// be loaded until the current block number is at least `n + lag`.  This is useful
    /// for reducing the probability of experiencing a chain reorg.
    ///
    pub fn block_stream(&self, start: U256, poll: Duration, lag: Option<u8>) -> impl Stream<Item=Block<H256>,Error=Error> {
        let mut block = start;
        let api = self.api();
        let mut work = None;
        stream::poll_fn(move || {
            let rslt = work.get_or_insert_with(|| {
                let api = api.clone();
                let target_block = block.clone();
                let mut not_before = block.clone();
                add_assign(&mut not_before,lag.unwrap_or(0)); 
                api.util().await_block_number(not_before,poll).and_then(move |()| {
                    api.util().await_block(target_block.into(),poll)
                })
            }).poll();
            match rslt {
                Ok(Async::Ready(item)) => {
                    let _ = work.take();
                    add_assign(&mut block,1);
                    Ok(Async::Ready(Some(item)))
                },
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Err(err) => {
                    let _ = work.take();
                    Err(err)
                }
            }
        })
    }


    /// Stream logs in order (batched by block).
    ///
    /// See `block_stream` for detailes on how the `lag` argument is handled.
    ///
    pub fn log_stream(&self, start: U256, poll: Duration, filter: Filter, lag: Option<u8>) -> impl Stream<Item=(U256,Vec<Log>),Error=Error> {
        let filter = {
            let mut empty = Filter::default();
            empty.topics = filter.topics;
            empty.address = filter.address;
            empty
        };
        let mut block = start;
        let api = self.api();
        let mut work = None;
        stream::poll_fn(move || {
            let rslt = work.get_or_insert_with(|| {
                let api = api.clone();
                let mut log_filter = filter.clone();
                let target_block = block.clone();
                log_filter.from_block = Some(target_block.into());
                log_filter.to_block = Some(target_block.into());
                let mut not_before = target_block.clone();
                add_assign(&mut not_before,lag.unwrap_or(0));
                api.util().await_block_number(not_before,poll).and_then(move |()| {
                    api.eth().get_logs(log_filter).map(move |logs| {
                        debug_assert!(logs.iter().all(|l| l.block_number == Some(target_block)));
                        (target_block,logs)
                    })
                })
            }).poll();
            match rslt {
                Ok(Async::Ready(item)) => {
                    let _ = work.take();
                    add_assign(&mut block,1);
                    Ok(Async::Ready(Some(item)))
                },
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Err(err) => {
                    let _ = work.take();
                    Err(err)
                }
            }
        })
    }


    /// Stream of latest logs
    ///
    /// Due to the nature of blockchains, polling the `pending` queue for logs may return
    /// duplicates, logs which never end up being included, or miss logs entirely if they are
    /// produced by transactions which are included without fully propogating the network.
    /// This method attempts to increase the reliability of monitoring the pending queue
    /// by only polling the pending queue once per block, returning both
    /// the current state of the pending queue, as well as the logs included in `latest`.  While
    /// this doesn't preclude an unseen log being included on a reorg, it does significantly reduce
    /// the probability of missing a log in comparison to simply polling `pending`.
    ///
    pub fn stream_logs_latest(&self, poll: Duration, filter: Filter) -> impl Stream<Item=LatestLogs,Error=Error> {
        let api = self.api();
        api.eth().block_number().map(move |latest| {
            api.util().stream_logs_latest_inner(latest,poll,filter)
        }).flatten_stream()
    }


    fn stream_logs_latest_inner(&self, start: U256, poll: Duration, filter: Filter) -> impl Stream<Item=LatestLogs,Error=Error> {
        let filter = {
            let mut empty = Filter::default();
            empty.topics = filter.topics;
            empty.address = filter.address;
            empty
        };
        let pending_filter = {
            let mut base  = filter.clone();
            base.from_block = Some(BlockId::Pending);
            base.to_block = Some(BlockId::Pending);
            base
        };
        let mut block = start;
        let api = self.api();
        let mut work = None;
        stream::poll_fn(move || {
            let rslt = work.get_or_insert_with(|| {
                let api = api.clone();
                let target_block = block.clone();
                let pending_filter = pending_filter.clone();
                let mut latest_filter = filter.clone(); 
                latest_filter.from_block = Some(target_block.into());
                latest_filter.to_block = Some(target_block.into());
                api.util().await_block_number(target_block,poll).and_then(move |()| {
                    let get_pending = api.eth().get_logs(pending_filter).map(|logs| {
                        debug_assert!(logs.iter().all(|l| l.block_number.is_none()));
                        logs
                    });
                    let get_latest = api.eth().get_logs(latest_filter).map(move |logs| {
                        debug_assert!(logs.iter().all(|l| l.block_number == Some(target_block)));
                        (target_block,logs)
                    });
                    get_latest.join(get_pending).map(|((block_number,included),pending)| {
                        LatestLogs { block_number, included, pending }
                    })
                })
            }).poll();
            match rslt {
                Ok(Async::Ready(item)) => {
                    let _ = work.take();
                    add_assign(&mut block,1);
                    Ok(Async::Ready(Some(item)))
                },
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Err(err) => {
                    let _ = work.take();
                    Err(err)
                }
            }
        })
    }


    fn api(&self) -> Api<T> { Api::new(self.transport.to_owned()) }
}


/// Collection of logs from `latest` + `pending`
pub struct LatestLogs {
    /// Current block number
    pub block_number: U256,
    /// Logs included in this block
    pub included: Vec<Log>,
    /// Pending logs as of `block_number`
    pub pending: Vec<Log>,
}


impl LatestLogs {

    /// Iterate across all logs (included and pending)
    ///
    pub fn iter(&self) -> impl Iterator<Item=&Log> {
        self.included.iter().chain(self.pending.iter())
    }
}


fn add_assign(num: &mut U256, add: u8) {
    let lhs = U256::from(add as u64);
    let overflow = bufmath::add(num,&lhs);
    assert!(overflow == false,"256-bit integer overflow during add-assign");
}


