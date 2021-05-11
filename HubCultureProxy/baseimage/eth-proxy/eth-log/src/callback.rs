use proxy::http::{Request,Body,ParseJsonBody,Client};
use ethrpc::types::Log;
use config::{Endpoint,Callback};
use error::Error;
use util::Stdout;
use std::collections::HashMap;
use std::borrow::Borrow;
use tera::Tera;
use tokio::prelude::*;

/// Serialization target for template data
///
#[derive(Debug,Clone,Serialize,Deserialize)]
struct Data<L,E,M> {
     /// Indexed event parameters (parsed & stored by name)
    event: E,
    /// Raw log datastructure, as provided by node
    log: L,
    /// Extra metadata
    meta: M,
}


/// Serialization target for metadata
///
#[derive(Debug,Clone,Serialize,Deserialize)]
struct Meta<'a> {
    /// Name of the event being handled
    event_name: &'a str,
}


pub struct LogCallback<T> {
    callback: Callback,
    tera: T,
    stdout: Stdout,
    client: Client,
}

impl<T> LogCallback<T> {

    pub fn new(callback: Callback, tera: T, stdout: Stdout, client: Client) -> Self { Self { callback, tera, stdout, client } }
}


impl<T> LogCallback<T> where T: Borrow<Tera> {

    pub fn handle_log(&self, log: &Log) -> impl Future<Item=(),Error=Error> {
        let work = self.template_log(log).map(|templated| {
            self.dispatch_log(templated)
        }).into_future().flatten();
        work
    }


    fn dispatch_log(&self, templated: String) -> impl Future<Item=(),Error=Error> {
        match self.callback.endpoint() {
            Endpoint::Uri(uri) => {
                let build_request = Request::post(uri.to_owned())
                    .header("Content-Type","application/json")
                    .body(templated.into());
                let send_request = build_request.map(|request| {
                    self.send_request(request)
                }).into_future().from_err::<Error>().flatten();
                future::Either::A(send_request)
            },
            Endpoint::Stdout => {
                let push_line = self.stdout.push_line(templated).map_err(|_| {
                    // FIXME: While highly unlikely, the failure of the stdout handle
                    // is permanent (unlike http callbacks, which may fail intermittently).
                    // This condition shuold be (but currently is not) treated differently
                    // than a simple http callback failure.
                    Error::message("stdout handle failed unexpectedly")
                });
                future::Either::B(push_line.into_future())
            },
        }
    }


    fn send_request(&self, request: Request<Body>) -> impl Future<Item=(),Error=Error> {
        self.client.request(request).from_err().and_then(move |rsp| {
            let status = rsp.status();
            ParseJsonBody::parse(rsp.into_body()).from_err().and_then(move |body| {
                if !status.is_success() {
                    warn!("Non-success status {} with body: {:?}",status,body);
                    let msg = format!("non-success status code `{}`",status);
                    Err(Error::message(msg))
                } else {
                    debug!("Status {} with body: {:?}",status,body);
                    Ok(())
                }
            })
        })
    }


    fn template_log(&self, log: &Log) -> Result<String,Error> {
        if let Some((event_topic,params)) = log.topics.split_first() {
            if let Some(event) = self.callback.get_event(event_topic) {
                let decoded = event.decode(params)
                    .collect::<HashMap<_,_>>();
                let meta = Meta { event_name: event.name() };
                let data = Data { event: decoded, log, meta };
                let templated = self.tera().render(event.template(),&data)?;
                Ok(templated)
            } else {
                let msg = format!("No event found for topic {}",event_topic);
                Err(Error::message(msg))
            }
        } else {
            Err(Error::message("got zero-topic log"))
        }
    }

    fn tera(&self) -> &Tera { self.tera.borrow() }
}

