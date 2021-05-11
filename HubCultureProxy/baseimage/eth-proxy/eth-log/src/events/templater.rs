use ethrpc::types::{Log,H256};
use config::Event;
use tera::{self,Tera};
use tokio_util::{Never,service};
use tokio::prelude::*;
use std::collections::HashMap;
use std::borrow::Borrow;
use std::{fmt,error};


pub type Handle = service::Handle<Vec<Log>,Result<Vec<String>,Error>>;


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


/// Applies templates to raw logs.
///
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Templater<T> {
    events: HashMap<H256,Event>,
    tera: T,
}


impl<T> Templater<T> {

    pub fn new(events: HashMap<H256,Event>, tera: T) -> Self { Self { events, tera } }
}


impl<T> Templater<T> where T: Borrow<Tera> + Send + 'static {


    pub fn spawn(self) -> impl Future<Item=Handle,Error=Never> {
        future::lazy(move || { Ok(self.spawn_now()) })
    }

    pub fn spawn_now(self) -> Handle {
        service::spawn_now(move |logs: Vec<Log>| {
            let rslt: Result<Vec<_>,_> = logs.iter().map(|log| {
                self.template_log(log)
            }).collect();
            Ok(rslt)
        })
    }
}


impl<T> Templater<T> where T: Borrow<Tera> {

    pub fn template_log(&self, log: &Log) -> Result<String,Error> {
        if let Some((event_topic,params)) = log.topics.split_first() {
            if let Some(event) = self.events.get(event_topic) {
                let decoded = event.decode(params)
                    .collect::<HashMap<_,_>>();
                let meta = Meta { event_name: event.name() };
                let data = Data { event: decoded, log, meta };
                let templated = self.tera().render(event.template(),&data)?;
                Ok(templated)
            } else {
                Err(Error::UnknownTopic {
                    topic: *event_topic
                })
            }
        } else {
            Err(Error::EmptyLog)
        }
    }

    fn tera(&self) -> &Tera { self.tera.borrow() }
}


#[derive(Debug)]
pub enum Error {
    Tera(tera::Error),
    UnknownTopic {
        topic: H256,
    },
    EmptyLog,
}


impl From<tera::Error> for Error {

    fn from(err: tera::Error) -> Self { Error::Tera(err) }
}


impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Tera(err) => err.fmt(f),
            Error::UnknownTopic { topic } => {
                write!(f,"No event found for topic {}",topic)
            },
            Error::EmptyLog => f.write_str("Got log with zero topics"),
        }
    }
}


impl error::Error for Error {

    fn description(&self) -> &str {
        match self {
            Error::Tera(err) => err.description(),
            Error::UnknownTopic { .. } => "log contained unknown topic",
            Error::EmptyLog => "got log with zero topics",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            Error::Tera(err) => Some(err),
            _other => None,
        }
    }
}

