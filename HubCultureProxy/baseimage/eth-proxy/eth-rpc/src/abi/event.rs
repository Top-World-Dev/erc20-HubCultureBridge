use smallvec::SmallVec;
use types::{Topics,Topic,H256};
use std::{iter,fmt,error};

use abi::{self,Token,Value};


/// A solidity event specification.
/// 
/// See module-level docs for example usage.
///
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct Event {
    pub name: String,
    pub inputs: Params,
}


impl Event {

    /// Get the signature hash for this event.
    ///
    pub fn signature(&self) -> H256 {
        let types = self.inputs.iter().map(|p| p.kind);
        let sighash = abi::signature(&self.name,types);
        debug!("calculated signature {} for event {}",sighash,self.name);
        sighash
    }


    /// Attempt to encode filter topics w/ specified values.
    ///
    pub fn encode_topics<T>(&self, topic_values: &[T]) -> Result<Topics,FilterError> where T: AsRef<[Value]> {
        let indexed_count = self.iter_indexed().count();
        if topic_values.len() == indexed_count {
            let mut topics = Topics::with_capacity(indexed_count + 1);
            topics.push(Some(self.signature().into()));
            for (tindex,((_,kind),values)) in self.iter_indexed().zip(topic_values).enumerate() {
                let values = values.as_ref();
                let mut topic_words = Vec::with_capacity(values.len());
                for (vindex,value) in values.iter().enumerate() {
                    match value.try_cast(kind) {
                        Ok(expected) => {
                            let word = expected.into_word();
                            topic_words.push(H256::from(word));
                        },
                        Err(other) => {
                            return Err(FilterError::TopicType {
                                expecting: kind,
                                got: other,
                                topic: tindex,
                                value: vindex,
                            });
                        }
                    }
                }
                let topic = Topic::from(topic_words);
                topics.push(topic.compress());
            }
            Ok(topics)
        } else {
            Err(FilterError::TopicCount {
                expecting: indexed_count,
                got: topic_values.len(),
            })
        }
    }


    /// Apply decoders to indexed topics (truncating).
    ///
    /// This function applies decoders to topics in order, but may yield
    /// less decoded fields than expected if fewer topics were provided.
    ///
    pub fn decode<'a>(&'a self, topics: &'a [H256]) -> impl Iterator<Item=(&'a str,Value)> + 'a {
        topics.iter().zip(self.iter_indexed()).map(|(topic,(name,token))| {
            let parsed = token.cast_word(*topic);
            (name,parsed)
        })
    }

    /// Apply decoders to indexed topics.
    ///
    /// This function applies decoders to topics in order, yielding
    /// `None` for trailing fields.
    ///
    pub fn decode_all<'a>(&'a self, topics: &'a [H256]) -> impl Iterator<Item=(&'a str,Option<Value>)> + 'a {
        let topics = topics.iter().map(|topic| Some(*topic))
            .chain(iter::repeat(None));
        topics.zip(self.iter_indexed()).map(|(topic,(name,token))| {
            let parsed = topic.map(|t| token.cast_word(t));
            (name,parsed)
        })
    }

    /// Iterate across the set of *indexed* parameters
    ///
    pub fn iter_indexed(&self) -> impl Iterator<Item=(&str,Token)> {
        self.inputs.iter().filter(|p|p.indexed).map(|param| {
            (param.name.as_ref(),param.kind)
        })
    }
}


impl From<(String,Params)> for Event {

    fn from(parts: (String,Params)) -> Self {
        let (name,inputs) = parts;
        Self { name, inputs }
    }
}


///  An ordered collection of event parameters.
///
pub type Params = SmallVec<[Param;4]>;


/// A named/typed event parameter.
///
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Param {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: Token,
    pub indexed: bool,
}


/// Indicates failure to encode filter topics.
///
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum FilterError {
    /// Topic count didn't match
    TopicCount {
        expecting: usize,
        got: usize,
    },
    /// Topic type didn't match
    TopicType {
        expecting: Token,
        got: Token,
        topic: usize,
        value: usize,
    },
}


impl fmt::Display for FilterError {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FilterError::TopicCount { expecting, got } => {
                write!(f,"Invalid topic count; expecting {} got {}",expecting,got)
            },
            FilterError::TopicType { expecting, got, topic, value } => {
                write!(f,"Invalid topic type; expecting {} got {} (topic {} value {})",expecting,got,topic,value)
            },
        }
    }
}


impl error::Error for FilterError {

    fn description(&self) -> &str {
        match self {
            FilterError::TopicCount { .. } => "invalid topic count",
            FilterError::TopicType { .. } => "invalid topic type",
        }
    }
}

