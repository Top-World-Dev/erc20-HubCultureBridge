use tokio::net::TcpStream;
use tokio_tls::TlsStream;
use tokio::prelude::*;
use std::io;


#[derive(Debug)]
pub enum MaybeTls {
    Tls(TlsStream<TcpStream>),
    Tcp(TcpStream),
}

impl MaybeTls {

    pub fn get_ref(&self) -> &TcpStream {
        match self {
            MaybeTls::Tls(stream) => stream.get_ref().get_ref(),
            MaybeTls::Tcp(stream) => stream
        }
    }
}


impl Read for MaybeTls {

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            MaybeTls::Tls(stream) => stream.read(buf),
            MaybeTls::Tcp(stream) => stream.read(buf),
        }
    }
}


impl Write for MaybeTls {

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            MaybeTls::Tls(stream) => stream.write(buf),
            MaybeTls::Tcp(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            MaybeTls::Tls(stream) => stream.flush(),
            MaybeTls::Tcp(stream) => stream.flush(),
        }
    }
}


impl AsyncRead for MaybeTls { }

impl AsyncWrite for MaybeTls {

    fn shutdown(&mut self) -> Poll<(),io::Error> {
        match self {
            MaybeTls::Tls(stream) => stream.shutdown(),
            MaybeTls::Tcp(stream) => stream.shutdown(),
        }
    }
}



/// Serialize/Deserialize a type using its `Display` and `FromStr` implementations respectively.
pub mod serde_str {
    use serde::de::{self,Deserializer};
    use serde::ser::Serializer;
    use std::fmt::Display;
    use std::str::FromStr;

    /// Serialize any type which implemented `Display`
    ///
    pub fn serialize<T,S>(item: &T, serializer: S) -> Result<S::Ok,S::Error> where S: Serializer, T: Display {
        serializer.collect_str(item)
    }

    /// Deserialize any type which implements `FromStr`
    ///
    pub fn deserialize<'de,T,D>(deserializer: D) -> Result<T,D::Error> where T: FromStr, T::Err: Display, D: Deserializer<'de> {
        let target: Target = de::Deserialize::deserialize(deserializer)?;
        let parsed = target.as_str().parse().map_err(de::Error::custom)?;
        Ok(parsed)
    }


    /// Intermediate deserialization target; prefers `&str` but will successfully
    /// accept `String` (e.g. when deserializing from `serde_json::Value`).
    #[derive(Debug,Serialize,Deserialize)]
    #[serde(untagged)]
    enum Target<'a> {
        Ref(&'a str),
        Own(String),
    }

    impl<'a> Target<'a> {

        fn as_str(&self) -> &str {
            match self {
                Target::Ref(s) => s,
                Target::Own(s) => s,
            }
        }
    }
}
