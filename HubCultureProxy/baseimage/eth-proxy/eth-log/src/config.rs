use ethrpc::types::{Filter,Origin,H256};
use ethrpc::{abi,crypto};
use proxy::util::serde_str;
use proxy::http::Uri;
use error::Error;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::{env,fmt,error};

/// Fully sepcified configuration
///
#[derive(Debug,Clone)]
pub struct Config {
    servers: Vec<Server>,
    callbacks: Vec<Callback>,
    events: HashMap<H256,Event>,
}


impl Config {


    pub fn iter_callbacks(&self) -> impl Iterator<Item=&Callback> {
        self.callbacks.iter()
    }

    pub fn iter_servers(&self) -> impl Iterator<Item=&Server> {
        self.servers.iter()
    }

    pub fn iter_events(&self) -> impl Iterator<Item=(&H256,&Event)> {
        self.events.iter()
    }

    pub fn try_from(config: ConfigFile) -> Result<Self,Error> {
        // Store event specifications by signature
        let events = config.events.into_iter().map(|event| {
            let topic_sig = event.event.signature();
            (topic_sig,event)
        }).collect::<HashMap<_,_>>();
        
        // Attempt to assemble server configurations
        let servers = config.servers.into_iter().map(|server| -> Result<_,Error> {
            // Attempt to create server-specific event mapping.
            let events = server.events.into_iter().map(|name| {
                // We deal with conflicting event names by allowing the user to
                // specify an event by abi signature.  We must, therefore, attempt to
                // acquire the event by hash prior to a conventional name-based search.
                let sig = crypto::keccak(name.as_str());
                if let Some(event) = events.get(&sig) {
                    Ok((sig,event.to_owned()))
                } else {
                    // If unable to locate the event by hash, we just attempt to grab the 
                    // first event with the supplied name...
                    match events.iter().filter(|(_,e)| e.event.name == name).next() {
                        Some((topic,event)) => Ok((*topic,event.to_owned())),
                        None => {
                            let msg = format!("Unable to locate event `{}`",name);
                            Err(Error::message(msg))
                        },
                    }
                }
            }).collect::<Result<_,_>>()?;
            let (address,origin) = (server.address,server.origin);
            Ok(Server { address, origin, events })
        }).collect::<Result<_,_>>()?;

        // Attempt to assemble callback configurations
        let callbacks = config.callbacks.into_iter().map(|callback| -> Result<_,Error> {
            let endpoint = callback.callback.try_resolve()?;
            // Attempt to create callback-specific event mapping.
            let events = callback.events.into_iter().map(|name| {
                // We deal with conflicting event names by allowing the user to
                // specify an event by abi signature.  We must, therefore, attempt to
                // acquire the event by hash prior to a conventional name-based search.
                let sig = crypto::keccak(name.as_str());
                if let Some(event) = events.get(&sig) {
                    Ok((sig,event.to_owned()))
                } else {
                    // If unable to locate the event by hash, we just attempt to grab the 
                    // first event with the supplied name...
                    match events.iter().filter(|(_,e)| e.event.name == name).next() {
                        Some((topic,event)) => Ok((*topic,event.to_owned())),
                        None => {
                            let msg = format!("Unable to locate event `{}`",name);
                            Err(Error::message(msg))
                        },
                    }
                }
            }).collect::<Result<_,_>>()?;
            let (callback,origin) = (endpoint,callback.origin);
            Ok(Callback { callback, origin, events })
        }).collect::<Result<_,_>>()?;
        Ok(Self { servers, callbacks, events })
    }
}


/// Deserializaiton target for config file
///
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    #[serde(default,skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "server-config")]
    servers: Vec<ServerConfig>,
    #[serde(default,skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "callback-config")]
    callbacks: Vec<CallbackConfig>,
    #[serde(default,skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "event-config")]
    events: Vec<EventConfig>,
}



/// Basic server configuration
///
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct ServerConfig {
    /// Socket-address to serve on
    #[serde(rename = "socket-addr")]
    address: SocketAddr,
    /// Origin(s) of interest
    origin: Origin,
    /// Events which may be queried
    events: Vec<String>,
}


#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Server {
    pub address: SocketAddr,
    pub origin: Origin,
    pub events: HashMap<H256,Event>,
}


/// Basic callback configuration
///
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct CallbackConfig {
    /// Uri against against which the callback should be made
    #[serde(with = "serde_str")]
    callback: EndpointConfig,
    /// Origin(s) of interest
    origin: Origin,
    /// Events which may be emitted
    events: Vec<String>,
}


/// Fully specified callback
///
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Callback {
    callback: Endpoint,
    origin: Origin,
    events: HashMap<H256,Event>,
}


impl Callback {

    /// Get the endpoint of this callback
    pub fn endpoint(&self) -> &Endpoint { &self.callback }

    /// Get the event matching the supplied hash (if exists)
    pub fn get_event(&self, hash: &H256) -> Option<&Event> {
        self.events.get(hash)
    }

    /// Compute an appropriate log filter
    ///
    pub fn filter(&self) -> Filter {
       let event_topics = self.events.keys().cloned()
           .collect::<Vec<_>>();
       Filter::builder()
           .origin(self.origin.to_owned())
           .topics(
               Some(event_topics.into()),
               None,
               None,
               None,
            ).finish()
    }
}


#[derive(Hash,Debug,Clone,PartialEq,Eq)]
pub enum EndpointConfig {
    Var(String),
    Endpoint(Endpoint),
}


impl EndpointConfig {

    pub fn try_resolve(self) -> Result<Endpoint,Error> {
        match self {
            EndpointConfig::Var(var) => {
                match env::var(&var) {
                    Ok(value) => {
                        let endpoint = value.parse().map_err(|err| {
                            let msg = format!("failed to parse var {}: {}",var,err);
                            Error::message(msg)
                        })?;
                        Ok(endpoint)
                    },
                    Err(err) => {
                        let msg = format!("failed load var {} ({})",var,err);
                        Err(Error::message(msg))
                    }
                }
            },
            EndpointConfig::Endpoint(e) => Ok(e),
        }
    }
}


impl fmt::Display for EndpointConfig {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EndpointConfig::Var(var) => write!(f,"${}",var),
            EndpointConfig::Endpoint(e) => e.fmt(f),
        }
    }
}


impl FromStr for EndpointConfig {

    type Err = Error;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        if s.starts_with("$") {
            let var = s.trim_left_matches("$");
            if var.len() > 0 {
                Ok(EndpointConfig::Var(var.to_owned()))
            } else {
                Err(Error::message("endpoint var must be non-empty"))
            }
        } else {
            match s.parse() {
                Ok(e) => Ok(EndpointConfig::Endpoint(e)),
                Err(_) => {
                    let msg = format!("invalid endpoint config: `{}`",s);
                    Err(Error::message(msg))
                }
            }
        }
    }
}


/// Indicates the endpoint for a callback.
///
/// ```
/// # extern crate eth_log;
/// # use eth_log::config::Endpoint;
/// # fn main() {
/// // Indicates that callback should write to stdout
/// let stdout: Endpoint = "stdout".parse().unwrap();
/// assert_eq!(stdout,Endpoint::Stdout);
/// 
/// // Indicates that callback should post to a remote server
/// let example: Endpoint = "http://example.org/hello/world".parse().unwrap();
/// assert_eq!(example.as_uri().unwrap().path(),"/hello/world");
/// # }
/// ```
#[derive(Hash,Debug,Clone,PartialEq,Eq)]
pub enum Endpoint {
    Uri(Uri),
    Stdout,
}


impl Endpoint {

    pub fn as_uri(&self) -> Option<&Uri> {
        match self {
            Endpoint::Uri(uri) => Some(uri),
            _other => None
        }
    }
}


impl fmt::Display for Endpoint {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Endpoint::Uri(uri) => uri.fmt(f),
            Endpoint::Stdout => f.write_str("stdout"),
        }
    }
}


impl FromStr for Endpoint {

    type Err = ParseEndpointError;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        match s {
            "stdout" => Ok(Endpoint::Stdout),
            other => {
                let uri = other.parse().map_err(|_| {
                    ParseEndpointError
                })?;
                Ok(Endpoint::Uri(uri))
            }
        }
    }
}

#[derive(Debug,Copy,Clone)]
pub struct ParseEndpointError;


impl ParseEndpointError {

    fn as_str(&self) -> &str { "expected `stdout` or a valid uri" }
}

impl fmt::Display for ParseEndpointError {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl error::Error for ParseEndpointError {

    fn description(&self) -> &str { self.as_str() }
}


/// Basic event configuration
///
pub type EventConfig = Event;


/// Fully specified event
///
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct Event {
    /// Name of the template which should be used to
    /// process this event.
    template: String,
    /// Inner event specification
    #[serde(flatten)]
    event: abi::Event,
}

impl Event {

    /// Get the name of the template for this event
    pub fn template(&self) -> &str { &self.template }

    /// Get the name of this event
    pub fn name(&self) -> &str { &self.event.name }

    /// Get ref to inner `ethrpc::abi::Event`.
    pub fn inner(&self) -> &abi::Event { &self.event }

    /// Decode topics for this event
    pub fn decode<'a>(&'a self, topics: &'a [H256]) -> impl Iterator<Item=(&'a str,Option<abi::Value>)> + 'a {
        self.event.decode_all(topics)
    }
}
