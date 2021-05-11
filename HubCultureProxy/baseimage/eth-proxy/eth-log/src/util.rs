use ethrpc::types::{U256,Filter,Never,Log};
use ethrpc::util::bufmath;
use ethrpc::{self,Url};
use error::Error;
use tokio::timer::Delay;
use tokio::prelude::*;
use std::time::{Instant,Duration};

use tokio_channel::mpsc;
use tokio::{self,io};


pub type StdoutError = mpsc::SendError<String>;

/// Handle which allows pushing lines to stdout from multiple tasks.
///
#[derive(Debug,Clone)]
pub struct Stdout {
    inner: mpsc::UnboundedSender<String>
}


impl Stdout {

    fn new(inner: mpsc::UnboundedSender<String>) -> Self { Self { inner } }

    pub fn push_line(&self, line: impl Into<String>) -> Result<(),StdoutError> {
        self.inner.unbounded_send(line.into())
    }
}


/// Asynchronously spawn a background task which manages writing lines to stdout.
///
pub fn stdout() -> impl Future<Item=Stdout,Error=Never> {
    future::lazy(|| { Ok(spawn_stdout()) })
}


/// Spawn a background task which manages writing lines to stdout.
/// 
/// ## Panics
///
/// This function will panic if called outside of an event loop.  The `stdout`
/// function is a non-panicking alternative.
///
pub fn spawn_stdout() -> Stdout {
    let (tx,rx) = mpsc::unbounded();
    let handle = Stdout::new(tx);
    let work = rx.for_each(|mut line| {
        line.push('\n');
        io::write_all(io::stdout(),line).then(|rslt| {
            if let Err(e) = rslt { error!("In stdout: {}",e); }
            Ok(())
        })
    });
    tokio::spawn(work);
    handle
}



/// Stream logs with basic retry behavior.
///
/// *note*: This helper only implements retry behavior in the strictest sense; it will attempt
/// to reestablish a log-stream up to three times within a two minute period.  *However*, it
/// enforces only minimal backoff with no jitter.  This function is intended to address
/// connection failures due to red/blue transitions & load-balancer connection timeouts,
/// *not* failures/restarts of a monolithic node.  This function is generally intended for
/// use against load-balanced node pools where near-instant reconnect attempts are
/// practical and appropriate.
///
pub fn stream_logs_with_retry(url: Url, start: U256, filter: Filter, lag: u8) -> impl Stream<Item=(U256,Vec<Log>),Error=Error> + Send + 'static {
    let builder = log_stream_builder(url,filter,lag);
    stream_logs_with_builder(builder,start)
}


fn stream_logs_with_builder(builder: impl LogStreamBuilder, start: U256) -> impl Stream<Item=(U256,Vec<Log>),Error=Error> {
    let backoff = Duration::from_millis(128);
    let cooldown = Duration::from_secs(127); 
    let max_fail = 3;
    let mut stream = None;
    let mut last_seen: Option<U256> = None;
    let mut last_err = None;
    let mut err_count = 0;
    stream::poll_fn(move || -> Result<Async<Option<(U256,Vec<Log>)>>,Error> {
        let poll = stream.get_or_insert_with(|| {
            let target = match last_seen.as_ref() {
                Some(block) => {
                    let mut block = block.to_owned();
                    add_assign(&mut block,1);
                    block
                },
                None => start,
            };
            // `ethapi` crate does not attempt connection until polled, so we
            // can pre-assemble the stream to avoid moving `builder`.
            let stream = builder.build(target);
            let until = Instant::now() + (backoff * err_count);
            Delay::new(until).from_err().map(move |()| {
                stream
            }).flatten_stream()
        }).poll();
        match poll {
            Ok(Async::Ready(Some((block,logs)))) => {
                last_seen = Some(block);
                Ok(Async::Ready(Some((block,logs))))
            },
            Ok(Async::Ready(None)) => {
                let _ = stream.take();
                Ok(Async::Ready(None))
            },
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(err) => {
                let _ = stream.take();
                let now = Instant::now();
                let mut last = last_err.get_or_insert(now);
                if (*last + cooldown) < now {
                    *last = now;
                    err_count = 0;
                }
                if err_count < max_fail {
                    err_count += 1;
                    warn!("Log stream failed, attempting retry ({} of {})",err_count,max_fail);
                    task::current().notify();
                    Ok(Async::NotReady)
                } else {
                    Err(err)
                }
            },
        }
    })
}


trait LogStreamBuilder {

    type Stream: Stream<Item=(U256,Vec<Log>),Error=Error> + Send + 'static;

    fn build(&self, start: U256) -> Self::Stream;
}


impl<T,S> LogStreamBuilder for T where T: Fn(U256) -> S, S: Stream<Item=(U256,Vec<Log>),Error=Error> + Send + 'static {

    type Stream = S;

    fn build(&self, start: U256) -> Self::Stream { (self)(start) }
}


fn log_stream_builder(url: Url, filter: Filter, lag: u8) -> impl LogStreamBuilder + Send + 'static {
    move |start| { stream_logs(url.clone(),start,filter.clone(),lag) }
}


fn stream_logs(url: Url, start: U256, filter: Filter, lag: u8) -> impl Stream<Item=(U256,Vec<Log>),Error=Error> + Send + 'static {
    ethrpc::connect(url).from_err::<Error>().map(move |api| {
        api.util().log_stream(start,Duration::from_millis(1024),filter,Some(lag)).from_err::<Error>()
    }).flatten_stream()
}


fn add_assign(num: &mut U256, add: u8) {
    let lhs = U256::from(add as u64);
    let overflow = bufmath::add(num,&lhs);
    assert!(overflow == false,"256-bit integer overflow during add-assign");
}
