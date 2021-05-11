extern crate tokio_tungstenite;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate serde;
#[macro_use]
extern crate structopt;
extern crate native_tls;
extern crate tokio_util;
extern crate tokio_tls;
extern crate tokio;
extern crate hyper_tls;
extern crate hyper;
extern crate http as _http;
#[macro_use]
extern crate log;
extern crate url;


#[macro_use]
mod macros;
pub mod options;
pub mod policy;
pub mod error;
pub mod http;
pub mod util;
pub mod ws;

pub use error::Error;

use util::MaybeTls;
use tokio::prelude::*;
use tokio::net::{self,TcpStream};
use tokio_tls::TlsStream;
use native_tls::Identity;
use std::net::SocketAddr;

use options::ServerOptions;
use policy::{Policy,Filter};




// TODO: Apply `Filter` prior to TLS handshake instead of after!


pub fn bind_with_options(addr: &SocketAddr, opt: &ServerOptions) -> Result<impl Stream<Item=MaybeTls,Error=Error>,Error> {
    let identity = if opt.use_tls {
        // Load PKCS-12 identity file
        let identity = opt.load_identity()?;
        info!("Configured for TLS; incoming connections will be encrypted");
        Some(identity)
    } else {
        warn!("Not configured for TLS; connections will be unencrypted");
        None
    };

    // Configure incoming connection filter
    let filter = if let Some(whitelist) = opt.load_whitelist()? {
        match whitelist.len() {
            0 => warn!("Configuring server with empty whitelist"),
            other => info!("Configuring server with {} whitelist addrs",other),
        }
        Filter::whitelist(whitelist)
    } else {
        warn!("No ip whitelist specified; allowing connections from all sources");
        Filter::blacklist(None)
    };

    debug!("{:?}",filter);

    Ok(bind_filtered(&addr,identity,filter))
}


/// Wrapper around `bind_maybe_tls` which applies a policy filter
///
fn bind_filtered(address: &SocketAddr, identity: Option<Identity>, filter: Filter) -> impl Stream<Item=MaybeTls,Error=Error> {
    bind_maybe_tls(address,identity).filter(move |stream| {
        match stream.get_ref().peer_addr() {
            Ok(addr) => {
                match filter.policy(addr.ip()) {
                    Policy::Allow => {
                        info!("Allowing connection from {}",addr);
                        true
                    },
                    Policy::Deny => {
                        warn!("Denying connection from {}",addr);
                        false
                    },
                }
            },
            Err(err) => {
                warn!("Error resolving peer address: `{}`",err);
                false
            }
        }
    })
}


fn bind_maybe_tls(address: &SocketAddr, identity: Option<Identity>) -> impl Stream<Item=MaybeTls,Error=Error> {
    enum Either<A,B> { Tls(A), Tcp(B) }
    let mut stream = match identity {
        Some(ident) => {
            let tls_stream = bind_tls(address,ident)
                .map(|conn| MaybeTls::Tls(conn));
            Either::Tls(tls_stream)
        },
        None => {
            let tcp_stream = bind(address)
                .map(|conn| MaybeTls::Tcp(conn));
            Either::Tcp(tcp_stream)
        },
    };
    stream::poll_fn(move || -> Poll<Option<MaybeTls>,Error> {
        match &mut stream {
            Either::Tls(stream) => stream.poll(),
            Either::Tcp(stream) => stream.poll(),
        }
    })
}


fn bind_tls(address: &SocketAddr, identity: Identity) -> impl Stream<Item=TlsStream<TcpStream>,Error=Error> {
    native_tls::TlsAcceptor::new(identity).map(tokio_tls::TlsAcceptor::from)
        .map_err(From::from).map(move |acceptor| {
            bind(address).and_then(move |tcp_stream| {
                acceptor.accept(tcp_stream).from_err()
            })
        }).into_future().flatten_stream()
}



fn bind(address: &SocketAddr) -> impl Stream<Item=TcpStream,Error=Error> {
    net::TcpListener::bind(address).into_future()
        .map(|listener| listener.incoming())
        .flatten_stream().from_err()
}

