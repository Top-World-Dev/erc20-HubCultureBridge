#[macro_use]
extern crate structopt;
extern crate eth_log;
extern crate signer;
extern crate signer_proxy;
#[macro_use]
extern crate proxy;
extern crate tokio;
extern crate env_logger;
#[macro_use]
extern crate log;

use tokio::net::TcpStream;
use tokio::prelude::*;
use eth_log::options::RunOptions as LogOptions;
use proxy::http;
use proxy::options::ServerOptions;
use signer_proxy::options::SignerProxyOptions;
use signer::options::SignerOptions;
use std::net::SocketAddr;
use structopt::StructOpt;
use log::LevelFilter;


const MOD_NAME: &str = module_path!();


wrap_errs!(
    Proxy => proxy::Error,
    Signer => signer::Error,
    EthLog => eth_log::Error,
);


#[derive(Debug,Clone,StructOpt)]
pub struct Opt {
    /// Set internal log-level
    #[structopt(short="l",long="log",name="level",default_value="info")]
    log_level: LevelFilter,
    /// Command to be performed
    #[structopt(subcommand)]
    cmd: Cmd,
}


#[derive(Debug,Clone,StructOpt)]
pub enum Cmd {
    /// Forward tcp traffic (nothing ethereum specific)
    #[structopt(name = "forward-tcp")]
    ForwardTcp {
        /// Accept inbound TLS at addr (host:port)
        #[structopt(name = "src-addr")]
        src_addr: SocketAddr,
        /// Forward as raw TCP to addr (host:port)
        #[structopt(name = "dst-addr")]
        dst_addr: SocketAddr,
        #[structopt(flatten)]
        server_options: ServerOptions,
    },
    /// Run a restricted signer proxy instance
    #[structopt(name = "signer-proxy")]
    SignerProxy {
        /// Serve requests at addr (host:port)
        #[structopt(name = "srv-addr")]
        srv_addr: SocketAddr,
        #[structopt(flatten)]
        server_options: ServerOptions,
        #[structopt(flatten)]
        signer_options: SignerProxyOptions,
    },
    /// Run a restricted signer instance
    #[structopt(name = "run-signer")]
    RunSigner {
        /// Serve requests at addr (host:port)
        #[structopt(name = "srv-addr")]
        srv_addr: SocketAddr,
        #[structopt(flatten)]
        server_options: ServerOptions,
        #[structopt(flatten)]
        signer_options: SignerOptions,
    },
    /// Generate a random seck256k1 secret key (written to stdout)
    #[structopt(name = "keygen")]
    KeyGen {},
    /// Stream EVM event logs
    #[structopt(name = "stream-logs")]
    StreamLogs {
        #[structopt(flatten)]
        server_options: ServerOptions,
        #[structopt(flatten)]
        log_options: LogOptions,
    },
}


fn main() {
    let opt = Opt::from_args();

    env_logger::Builder::from_default_env()
        .filter_module(MOD_NAME,opt.log_level)
        .filter_module("signer",opt.log_level)
        .filter_module("signer_proxy",opt.log_level)
        .filter_module("proxy",opt.log_level)
        .filter_module("ethrpc",opt.log_level)
        .filter_module("eth_log",opt.log_level)
        .init();

    debug!("Configured with {:?}",opt);

    match run(opt.cmd) {
        Ok(()) => {},
        Err(err) => {
            debug!("{:?}",err);
            eprintln!("Error: {}",err);
            ::std::process::exit(1);
        }
    }
}


fn run(cmd: Cmd) -> Result<(),Error> {
    match cmd {
        Cmd::ForwardTcp { src_addr, dst_addr, server_options } => {
            let incoming = proxy::bind_with_options(&src_addr,&server_options)?;
            info!("Configured with forwarding {} => {}",src_addr,dst_addr);
            let server = incoming.for_each(move |tls_stream| {
                let forward = TcpStream::connect(&dst_addr).and_then(move |tcp_stream| {
                    let (tls_reader,tls_writer) = tls_stream.split();
                    let (tcp_reader,tcp_writer) = tcp_stream.split();
                    let forward_tls = tokio::io::copy(tls_reader,tcp_writer);
                    let forward_tcp = tokio::io::copy(tcp_reader,tls_writer);
                    forward_tls.join(forward_tcp)
                }).map_err(|e| error!("During TLS forwarding `{}`",e)).map(drop);
                tokio::spawn(forward);
                Ok(())
            }).map_err(|e| error!("Server task failed with `{}`",e));

            tokio::run(server);
        },
        Cmd::SignerProxy { srv_addr, server_options, signer_options } => {
            let incoming = proxy::bind_with_options(&srv_addr,&server_options)?;
            let signer_setup = signer_proxy::spawn_local(&signer_options)?;
            info!("Configured to server signer-proxy at {}",srv_addr);
            let work = signer_setup.map(move |request_handler| {
                let server = http::serve_json(incoming, move |request| {
                    request_handler.handle_request(request).then(|rslt| {
                        Ok(rslt.map_err(|e| e.to_string()))
                    })
                }).map_err(|e| error!("Server task failed with `{}`",e));
                tokio::spawn(server);
            }).map_err(|e| error!("Signer-proxy setup failed with `{}`",e));

            tokio::run(work);
        },
        Cmd::RunSigner { srv_addr, server_options, signer_options } => {
            let incoming = proxy::bind_with_options(&srv_addr,&server_options)?;
            let signer_instance = signer::Signer::from_options(&signer_options)?;
            info!("Configured to serve {} with signer {}",srv_addr,signer_instance.address());
            let server = http::serve_json(incoming, move |request| {
                let response = signer_instance.serve(request)
                    .map_err(|e| e.to_string());
                Ok(response)
            }).map_err(|e| error!("Server task failed with `{}`",e));
            
            tokio::run(server);
        },
        Cmd::KeyGen {} => { println!("{:?}",signer::crypto::keygen()); },
        Cmd::StreamLogs { log_options, server_options } => {
            eth_log::run_with_servers(log_options,server_options)?;
        }
    }

    Ok(())
}

