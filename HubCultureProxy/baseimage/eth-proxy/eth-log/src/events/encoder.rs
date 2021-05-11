use ethrpc::abi::{Value,FilterError};
use ethrpc::types::helpers::ValOrSeq;
use ethrpc::types::{BlockId,Topics,Topic};
use config::Event;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::{fmt,error};


pub type TopicValue = ValOrSeq<Value>;

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Request {
    GetEvents(EventRequest),
}

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(untagged)]
pub enum Response {
    Events(Vec<String>)
}


/// Description of a filter request
///
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct EventRequest {
    pub matching: EventMatchers,
    /// Optionally specify where to start from
    #[serde(default,skip_serializing_if = "Option::is_none")]
    #[serde(rename = "fromBlock")]
    pub from_block: Option<BlockId>,

    /// Optionally specify where to search to
    #[serde(default,skip_serializing_if = "Option::is_none")]
    #[serde(rename = "toBlock")]
    pub to_block: Option<BlockId>,
}


pub type EventMatchers = ValOrSeq<EventMatcher>;


#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct EventMatcher {
    /// Name of the event to be filtered for
    pub name: String,
    /// Topic inputs as a mapping from name to value(s)
    #[serde(default,skip_serializing_if = "HashMap::is_empty")]
    pub inputs: HashMap<String,TopicValue>,
}




/// Event topic encoder
///
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Encoder {
    inner: HashMap<String,Event>,
}


impl Encoder {


    /// Attempt to encode one or more event topics
    ///
    pub fn encode_matchers(&self, matchers: EventMatchers) -> Result<Topics,Error> {
        let mut raw_topics = Vec::new();
        for EventMatcher { name , inputs } in matchers.into_iter() {
            let encoded = self.encode_topics(name,inputs)?;
            while raw_topics.len() < encoded.len() { raw_topics.push(Vec::new()); }
            for (mut collector,topic) in raw_topics.iter_mut().zip(encoded.into_iter()) {
                if let Some(values) = topic {
                    collector.extend(values.into_iter());
                }
            }
        }
        let topics = raw_topics.into_iter().map(|mut raw| {
            raw.dedup();
            let topic = Topic::from(raw);
            topic.compress()
        }).collect();
        Ok(topics)
    }

    /// Attempt to encode event topics
    ///
    pub fn encode_topics(&self, name: String, mut inputs: HashMap<String,TopicValue>) -> Result<Topics,Error> {
        if let Some(event) = self.inner.get(&name) {
            let mut topics = Vec::new();
            for (name,_) in event.inner().iter_indexed() {
                if let Some(value) = inputs.remove(name) {
                    topics.push(value.to_vec());
                } else {
                    topics.push(Vec::new());
                }
            }
            if let Some((name,value)) = inputs.into_iter().next() {
                Err(Error::UnknownTopic { name, value })
            } else {
                let encoded = event.inner().encode_topics(&topics)?;
                Ok(encoded)
            }
        } else {
            Err(Error::NoSuchEvent { name })
        }
    }

    pub fn get(&self, name: &str) -> Option<&Event> { self.inner.get(name) }

    pub fn iter(&self) -> impl Iterator<Item=(&str,&Event)> {
        self.inner.iter().map(|(name,event)| {
            (name.as_ref(),event)
        })
    }
}



impl FromIterator<Event> for Encoder {

    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item=Event> {
        let inner = iter.into_iter().map(|event| {
            (event.name().to_owned(),event)
        }).collect();
        Self { inner }
    }
}

impl<'a> FromIterator<&'a Event> for Encoder {

    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item=&'a Event> {
        iter.into_iter().cloned().collect()
    }
}



#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Error {
    Encode(FilterError),
    UnknownTopic {
        name: String,
        value: TopicValue,
    },
    NoSuchEvent {
        name: String,
    },
}


impl From<FilterError> for Error {

    fn from(err: FilterError) -> Self { Error::Encode(err) }
}


impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Encode(err) => err.fmt(f),
            Error::UnknownTopic { name, value } => {
                write!(f,"Unexpected topic `{}` ({:?})",name,value)
            },
            Error::NoSuchEvent { name } => {
                write!(f,"Unable to locate event `{}`",name)
            }
        }
    }
}


impl error::Error for Error {

    fn description(&self) -> &str {
        match self {
            Error::Encode(err) => err.description(),
            Error::UnknownTopic { .. } => "got unexpected topic",
            Error::NoSuchEvent { .. } => "event does not exist",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            Error::Encode(err) => Some(err),
            _other => None,
        }
    }
}

