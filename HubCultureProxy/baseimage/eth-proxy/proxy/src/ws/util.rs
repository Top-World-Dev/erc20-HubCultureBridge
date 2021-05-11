//! misc internal websocket helpers
//!
use tokio_tungstenite::tungstenite::Message as WebSocketMessage;
use tokio::timer::Interval;
use tokio::prelude::*;
use std::time::{Instant,Duration};
use std::mem;
use ws::types::Message;
use ws::Error;


macro_rules! try_ready {
    ($e:expr) => (match $e {
        Ok(Async::Ready(t)) => t,
        Ok(Async::NotReady) => return Ok(Async::NotReady),
        Err(e) => return Err(From::from(e)),
    })
}


macro_rules! try_ready_stream {
    ($e:expr) => (match $e {
        Ok(Async::Ready(Some(item))) => item,
        Ok(Async::Ready(None)) => return Ok(Async::Ready(None)),
        Ok(Async::NotReady) => return Ok(Async::NotReady),
        Err(e) => return Err(From::from(e)),
    })
}


#[derive(Debug)]
pub struct Connection<T> {
    inner: T,
    ping_interval: Interval,
    ping_state: PingState, 
    ping_buff: Option<WebSocketMessage>,
    pong_buff: Option<WebSocketMessage>,
    msg_buff: Option<WebSocketMessage>,
}


impl<T> Connection<T> {

    pub fn new(inner: T, max_idle: Duration) -> Self {
        let expiry = max_idle * 2;
        let ping_interval = Interval::new(Instant::now() + max_idle, max_idle);
        let ping_state = PingState::new(expiry);
        let (ping_buff,pong_buff,msg_buff) = Default::default();
        Self { inner, ping_interval, ping_state, ping_buff, pong_buff, msg_buff }
    }
}


impl<T> Connection<T> where T: Sink<SinkItem=WebSocketMessage> {

    #[inline]
    fn try_start_ping(&mut self, ping: WebSocketMessage) -> Poll<(),T::SinkError> {
        debug_assert!(self.ping_buff.is_none());
        debug_assert!(ping.is_ping());
        if let AsyncSink::NotReady(unsent_ping) = self.inner.start_send(ping)? {
            self.ping_buff = Some(unsent_ping);
            Ok(Async::NotReady)
        } else {
            Ok(Async::Ready(()))
        }
    }

    #[inline]
    fn try_start_pong(&mut self, pong: WebSocketMessage) -> Poll<(),T::SinkError> {
        debug_assert!(self.pong_buff.is_none());
        debug_assert!(pong.is_pong());
        if let AsyncSink::NotReady(unsent_pong) = self.inner.start_send(pong)? {
            self.pong_buff = Some(unsent_pong);
            Ok(Async::NotReady)
        } else {
            Ok(Async::Ready(()))
        }
    }

    #[inline]
    fn try_start_send_msg(&mut self, msg: WebSocketMessage) -> Poll<(),T::SinkError> {
        debug_assert!(self.msg_buff.is_none());
        debug_assert!(msg.is_text() || msg.is_binary());
        if let AsyncSink::NotReady(unsent_msg) = self.inner.start_send(msg)? {
            self.msg_buff = Some(unsent_msg);
            Ok(Async::NotReady)
        } else {
            Ok(Async::Ready(()))
        }
    }

    #[inline]
    fn poll_pending_buffs(&mut self) -> Poll<(),T::SinkError> {
        // poll buffered ping if exists
        if let Some(ping) = self.ping_buff.take() {
            try_ready!(self.try_start_ping(ping));
        }

        // ping buffer clear, poll buffered pong if exists
        if let Some(pong) = self.pong_buff.take() {
            try_ready!(self.try_start_pong(pong));
        }

        // ping & pong buffers clear, poll buffered message if exists
        if let Some(msg) = self.msg_buff.take() {
            try_ready!(self.try_start_send_msg(msg));
        }

        // all buffers clear...
        Ok(Async::Ready(()))
    }
}


impl<T> Connection<T> where T: Sink<SinkItem=WebSocketMessage>, Error: From<T::SinkError> {

    #[inline]
    fn process_incoming(&mut self, message: WebSocketMessage) -> Result<Option<Message>,Error> {
        self.ping_state.message_seen();
        match message {
            WebSocketMessage::Text(text) => Ok(Some(Message::from(text))),
            WebSocketMessage::Binary(binary) => Ok(Some(Message::from(binary))),
            WebSocketMessage::Ping(ping) => {
                if self.pong_buff.is_none() {
                    let pong = WebSocketMessage::Pong(ping);
                    trace!("xmit {:?}",pong);
                    let _: Async<()> = self.try_start_pong(pong)?;
                    Ok(None)
                } else {
                    Err(Error::message("pinged before previous pong could send"))
                }
            },
            WebSocketMessage::Pong(pong) => {
                self.ping_state.visit_pong(&pong)?;
                Ok(None)
            },
        }
    }
}


impl<T> Stream for Connection<T> where T: Stream<Item=WebSocketMessage> + Sink<SinkItem=WebSocketMessage>, Error: From<T::Error> + From<T::SinkError> {

    type Error = Error;

    type Item = Message;

    fn poll(&mut self) -> Poll<Option<Self::Item>,Self::Error> {
        // make progress on pending buffs if needed/able...
        let _: Async<()> = self.poll_pending_buffs()?;

        // run keepalive checks if interval is ready, starting
        // the ping process if connection appears inactive...
        if let Async::Ready(Some(instant)) = self.ping_interval.poll()
                .map_err(|_| Error::message("critical timer failure"))? {
            // if `turn` returns `Some`, send new ping...
            if let Some(ping) = self.ping_state.turn(instant)? {
                trace!("xmit {:?}",ping);
                // `ping_state` should never trigger a new ping unless
                // it already got response from last ping.
                debug_assert!(self.ping_buff.is_none());
                let _: Async<()> = self.try_start_ping(ping)?;
            }
        }

        // got the housekeeping out of the way... poller inner stream...
        loop {
            // poll for next available websocket message
            let message = try_ready_stream!(self.inner.poll());
            trace!("recv {:?}",message);
            // if `process_incoming` returned `Some`, then message is `Text`
            // or `Binary` and variant and must be passed to caller...
            if let Some(message) = self.process_incoming(message)? {
                return Ok(Async::Ready(Some(message)));
            }
        }
    }
}


impl<T> Sink for Connection<T> where T: Stream<Item=WebSocketMessage> + Sink<SinkItem=WebSocketMessage>, Error: From<T::Error> + From<T::SinkError> {

    type SinkError = Error;

    type SinkItem = Message;

    fn start_send(&mut self, item: Self::SinkItem) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        // make progress of pending buffers if needed/able...
        let _: Async<()> = self.poll_pending_buffs()?;
        
        if self.msg_buff.is_none() {
            trace!("xmit {:?}",item);
            let _: Async<()> = self.try_start_send_msg(item.into())?;
            Ok(AsyncSink::Ready)
        } else {
            Ok(AsyncSink::NotReady(item))
        }
    }

    fn poll_complete(&mut self) -> Poll<(),Self::SinkError> {
        // drive completion of buffered sends...
        try_ready!(self.poll_pending_buffs());
        // buffers are clear... flush inner sink...
        try_ready!(self.inner.poll_complete());
        // fully flushed...
        Ok(Async::Ready(()))
    }
}



#[derive(Debug)]
struct PingState {
    pending: Option<(Instant,[u8;4])>,
    seen_item: bool,
    counter: u32,
    expiry: Duration,
}


impl PingState {

    fn new(expiry: Duration) -> Self {
        let (pending,seen_item,counter) = Default::default();
        Self { pending, seen_item, counter, expiry }
    }


    /// generate next ping token
    #[inline]
    fn get_next_token(&mut self) -> [u8;4] {
        let token = unsafe { mem::transmute(self.counter.to_be()) };
        self.counter = self.counter.wrapping_add(1);
        token
    }


    /// inform `PingState` that a message has been seen
    ///
    #[inline]
    fn message_seen(&mut self) { self.seen_item = true; }


    /// step forward...
    ///
    #[inline]
    fn turn(&mut self, now: Instant) -> Result<Option<WebSocketMessage>,Error> {
        if !self.seen_item || self.pending.is_some() {
            self.require_ping(now)
        } else {
            self.seen_item = false;
            Ok(None)
        }
    }


    /// check data of incoming `Pong` against current state
    ///
    #[inline]
    fn visit_pong(&mut self, pong_data: &[u8]) -> Result<(),Error> {
        if let Some((_instant,expected_data)) = self.pending.take() {
            if pong_data == expected_data.as_ref() {
                self.seen_item = true;
                Ok(())
            } else {
                warn!("expected pong data `{:?}`, got `{:?}`",expected_data,pong_data);
                Err(Error::message("pong data did not match"))
            }
        } else {
            warn!("unexpected pong `{:?}`",pong_data);
            Err(Error::message("got unexpected pong"))
        }
    }


    /// called when connection appears stale & should be pinged.
    ///
    /// checks ping state assuming the supplied instant is approximately
    /// equivalent to the present time.  if no pending ping state exists,
    /// a new token is generated and yielded.  if a ping is pending and not
    /// yet expired, `Ok(None)` is returned.  if a ping is pending, but appears
    /// expired (relative to supplied instant), an error is returned.
    ///
    #[inline]
    fn require_ping(&mut self, now: Instant) -> Result<Option<WebSocketMessage>,Error> {
        if self.pending.is_some() {
            let ping_start = self.pending.as_ref().map(|&(start,_)| start)
                .expect("always exists");
            if now > (ping_start + self.expiry) {
                Err(Error::message("client failed to respond to ping within timeout"))
            } else {
                Ok(None)
            }
        } else {
            let token = self.get_next_token();
            let message = WebSocketMessage::Ping(token.as_ref().into());
            self.pending = Some((now,token)); 
            Ok(Some(message))
        }
    }
}

