use tokio_tungstenite::tungstenite::Message as WebSocketMessage;
use serde::{Serialize,Deserialize};
use serde_json;
use std::str::{
    Utf8Error,
    FromStr,
};
use ws::Error;


/// a websocket message
///
/// ```
/// extern crate proxy;
/// 
/// use proxy::ws::Message;
/// use std::time::Duration;
///
/// # fn main() {
/// let msg = Message::encode_json(&Duration::from_secs(2)).unwrap();
///
/// assert_eq!(msg.as_str().unwrap(),r#"{"secs":2,"nanos":0}"#);
///
/// # }
/// ```
#[derive(Debug,Clone,Hash,PartialEq,Eq,Serialize,Deserialize)]
#[serde(untagged)]
pub enum Message {
    /// text variant
    Text(String),

    /// binary variant
    Binary(Vec<u8>),
}


impl Message {

    /// attempt to reference message as string slice
    ///
    pub fn as_str(&self) -> Result<&str,Utf8Error> {
        match self {
            Message::Text(text) => Ok(text),
            Message::Binary(binary) => ::std::str::from_utf8(binary),
        }
    }

    /// attempt to parse message using `FromStr`
    ///
    pub fn parse<T>(&self) -> Result<T,Error> where T: FromStr {
        let msg_str = self.as_str().map_err(|_|Error::ParseError)?;
        msg_str.parse().map_err(|_|Error::ParseError)
    }

    /// attempt to create json encoded `Message` from serializeable type
    ///
    pub fn encode_json<T>(value: &T) -> serde_json::Result<Self> where T: ?Sized + Serialize {
        let json = serde_json::to_string(value)?;
        Ok(Message::Text(json))
    }

    /// attempt to parse message contents as json
    ///
    pub fn parse_json<'a,T>(&'a self) -> serde_json::Result<T> where T: Deserialize<'a> {
        match self {
            Message::Text(text) => serde_json::from_str(text),
            Message::Binary(binary) => serde_json::from_slice(binary),
        }
    }
}


impl AsRef<[u8]> for Message {

    fn as_ref(&self) -> &[u8] {
        match self {
            Message::Text(msg) => msg.as_ref(),
            Message::Binary(msg) => msg.as_ref(),
        }
    }
}


impl From<String> for Message {

    fn from(text: String) -> Self { Message::Text(text) }
}


impl<'a> From<&'a str> for Message {

    fn from(text: &'a str) -> Self { String::from(text).into() }
}


impl From<Vec<u8>> for Message {

    fn from(binary: Vec<u8>) -> Self { Message::Binary(binary) }
}


impl<'a> From<&'a [u8]> for Message {

    fn from(binary: &'a [u8]) -> Self { Vec::from(binary).into() }
}


impl Into<WebSocketMessage> for Message {

    fn into(self) -> WebSocketMessage {
        match self {
            Message::Text(msg) => WebSocketMessage::Text(msg),
            Message::Binary(msg) => WebSocketMessage::Binary(msg),
        }
    }
}

