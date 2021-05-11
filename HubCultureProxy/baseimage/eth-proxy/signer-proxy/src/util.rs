use std::time::{Instant,Duration};
use tokio::prelude::*;
use tokio::timer;


/// Represents a piece of work which may be
/// reattempted.
///
pub trait Work {

    type Item;

    type Error;

    type Future: Future<Item=Self::Item,Error=Self::Error>;

    fn get_job(&mut self) -> Self::Future;
}


impl<T,F> Work for T where T: FnMut() -> F, F: IntoFuture {

    type Item = <F as IntoFuture>::Item;

    type Error = <F as IntoFuture>::Error;

    type Future = <F as IntoFuture>::Future;

    fn get_job(&mut self) -> Self::Future { (self)().into_future() }
}


pub fn retry<W: Work>(retries: u32, work: W) -> impl Future<Item=W::Item,Error=W::Error>
        where W::Error: From<timer::Error> {
    let backoff = Duration::from_millis(127);
    retry_inner(backoff,retries,work)
}


fn retry_inner<W: Work>(backoff: Duration, retries: u32, mut work: W) -> impl Future<Item=W::Item,Error=W::Error>
        where W::Error: From<timer::Error> {
    let mut fail_count = 0u32; 
    let mut job = None;
    future::poll_fn(move || -> Poll<_,_> {
        let poll = job.get_or_insert_with(|| {
            let start_time = Instant::now() + (backoff * fail_count);
            let inner_job = work.get_job();
            timer::Delay::new(start_time).from_err()
                .and_then(move |()| inner_job)
        }).poll();
        match poll {
            Ok(Async::Ready(item)) => {
                let _ = job.take();
                Ok(Async::Ready(item))
            },
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(err) => {
                let _ = job.take();
                fail_count += 1;
                if fail_count > retries {
                    Err(err)
                } else {
                    debug!("Job failed; scheduling retry ({} of {})",fail_count,retries);
                    task::current().notify();
                    Ok(Async::NotReady)
                }
            }
        }
    })
}
