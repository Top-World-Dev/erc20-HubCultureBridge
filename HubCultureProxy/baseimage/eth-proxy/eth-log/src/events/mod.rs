/// Event processing tools.
///
pub mod templater;
pub mod encoder;


pub use self::templater::Templater;
pub use self::encoder::{
    Encoder,
    Request,
    Response,
    EventRequest,
};

use config::Event;
use ethrpc::types::{U256,H256,Log};
use ethrpc::types::{BlockId,Filter,Origin};
use ethrpc::{self,Url};
use tera::Tera;
use tokio_util::{Never,service};
use tokio::prelude::*;
use std::collections::HashMap;
use std::borrow::Borrow;

wrap_errs! {
    Encode => encoder::Error,
    Template => templater::Error,
    EthRpc => ethrpc::api::Error,
    Service => service::Error<Vec<Log>>,
}


pub struct EventServer {
    encoder: Encoder,
    base_filter: Filter,
    handle: templater::Handle,
    node: Url,
}


impl EventServer {

    pub fn spawn<T>(node: Url, start: U256, origin: Origin, tera: T, events: HashMap<H256,Event>) -> impl Future<Item=Self,Error=Never> 
            where T: Borrow<Tera> + Send + 'static {
        future::lazy(move || { Ok(Self::spawn_now(node,start,origin,tera,events)) })
    }

    /// NOTE: must only be called within event-loop
    pub fn spawn_now<T>(node: Url, start: U256, origin: Origin, tera: T, events: HashMap<H256,Event>) -> Self
            where T: Borrow<Tera> + Send + 'static {
        let base_filter = Filter::builder()
            .from_block(start.into())
            .to_block(BlockId::Latest)
            .origin(origin)
            .finish();
        let encoder = events.values().collect();
        let templater = Templater::new(events,tera);
        let handle = templater.spawn_now();
        Self { encoder, base_filter, handle, node }
    }


    pub fn call(&self, req: Request) -> impl Future<Item=Response,Error=Error> {
        match req {
            Request::GetEvents(req) => {
                self.get_events(req).map(|events| {
                    Response::Events(events)
                })
            },
            // ...
        }
    }


    fn get_events(&self, req: EventRequest) -> impl Future<Item=Vec<String>,Error=Error> {
        let EventRequest { matching, from_block, to_block } = req;
        let work = self.encoder.encode_matchers(matching).map(|topics| {
            let mut filter = self.base_filter.clone();
            filter.topics = Some(topics);
            if let Some(block) = from_block { filter.from_block = Some(block); }
            if let Some(block) = to_block { filter.to_block = Some(block); }
            let template_handle = self.handle.clone();
            ethrpc::connect(self.node.clone()).and_then(move |api| {
                api.eth().get_logs(filter)
            }).from_err::<Error>().and_then(move |logs| {
                template_handle.call(logs).from_err().and_then(|rslt| {
                    rslt.map_err(|err| Error::from(err))
                })
            })
        }).into_future().flatten();
        work
    }
}


