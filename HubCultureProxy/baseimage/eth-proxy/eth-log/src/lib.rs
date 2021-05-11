#[macro_use]
extern crate proxy;
#[macro_use]
extern crate structopt;
extern crate ethrpc;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;
extern crate tokio_channel;
extern crate tokio_util;
extern crate tokio;
extern crate tera;
extern crate ignore;
#[macro_use]
extern crate log;

pub mod callback;
pub mod options;
pub mod config;
pub mod util;
pub mod events;
mod error;


pub use error::Error;

use callback::LogCallback;
use options::RunOptions;
use proxy::options::ServerOptions;
use events::EventServer;
use proxy::http;
use tera::Tera;
use tokio::prelude::*;
use std::sync::Arc;



pub fn run_with_servers(opt: RunOptions, srv: ServerOptions) -> Result<(),Error> {
    let config = opt.load_config()?;
    let templates = opt.load_templates()?;
    for (_,event) in config.iter_events() {
        if !templates.contains_key(event.template()) {
            let msg = format!("Missing template `{}` (required by `{}`)",event.template(),event.name());
            return Err(Error::message(msg));
        }
    }
    let mut tera = Tera::default();
    tera.add_raw_templates(
        templates.iter().map(|(name,data)| {
            (name.as_str(),data.as_str())
        }).collect()
        )?;
    let tera = Arc::new(tera);
    let client = http::client()?;

    let server_conns: Vec<_> = config.iter_servers().enumerate().map(|(index,server)| {
        info!("Binding event-sever {} to {}",index,server.address);
        proxy::bind_with_options(&server.address,&srv)
    }).collect::<Result<_,_>>()?;
    
    let work = future::lazy(move || {
        let stdout = util::spawn_stdout();
        let mut jobs = Vec::new();
        // construct all callback jobs
        for (index,callback) in config.iter_callbacks().enumerate() {
            let (tera,stdout,client) = (tera.clone(),stdout.clone(),client.clone());
            let log_callback = LogCallback::new(callback.to_owned(),tera,stdout,client);
            let filter = callback.filter();
            info!("Configuring callback {} ({})",index,callback.endpoint());
            debug!("{:?}",filter);
            let logs = util::stream_logs_with_retry(opt.node_addr.clone(),opt.start_block,filter,opt.lag_by);
            let work = logs.from_err::<Error>().for_each(move |(_blk,logs)| {
                for log in logs.iter() {
                    let work = log_callback.handle_log(&log).map_err(move |e| {
                        error!("In callback {}: {}",index,e);
                    });
                    tokio::spawn(work);
                }
                Ok(())
            }).map_err(move |e| error!("In log-stream {}: {}",index,e));
            jobs.push(future::Either::A(work));
        }

        // construct all server jobs
        for (incoming,server_config) in server_conns.into_iter().zip(config.iter_servers()) {
            let event_server = EventServer::spawn_now(
                opt.node_addr.clone(),
                opt.start_block,
                server_config.origin.to_owned(),
                tera.clone(),
                server_config.events.to_owned()
                );
            let work = http::serve_json(incoming, move |request| {
                event_server.call(request).map_err(|err| {
                    error!("Serving event-request: {}",err);
                    err.to_string()
                }).then(|rslt| Ok(rslt))
            }).map_err(|e| error!("In event-server: {}",e));
            jobs.push(future::Either::B(work));
        }
        // TODO: Move "job" futures all behind oneshots.
        future::select_all(jobs)
            .map_err(drop)
            .map(drop)
    });
    tokio::run(work);
    Ok(())
}




pub fn run(opt: RunOptions) -> Result<(),Error> {
    let config = opt.load_config()?;
    let templates = opt.load_templates()?;
    for (_,event) in config.iter_events() {
        if !templates.contains_key(event.template()) {
            let msg = format!("Missing template `{}` (required by `{}`)",event.template(),event.name());
            return Err(Error::message(msg));
        }
    }
    let mut tera = Tera::default();
    tera.add_raw_templates(
        templates.iter().map(|(name,data)| {
            (name.as_str(),data.as_str())
        }).collect()
        )?;
    let tera = Arc::new(tera);
    let client = http::client()?;
    let work = future::lazy(move || {
        let stdout = util::spawn_stdout();
        let mut jobs = Vec::new();
        for (index,callback) in config.iter_callbacks().enumerate() {
            let (tera,stdout,client) = (tera.clone(),stdout.clone(),client.clone());
            let log_callback = LogCallback::new(callback.to_owned(),tera,stdout,client);
            let filter = callback.filter();
            info!("Configuring callback {} ({})",index,callback.endpoint());
            debug!("{:?}",filter);
            let logs = util::stream_logs_with_retry(opt.node_addr.clone(),opt.start_block,filter,opt.lag_by);
            let work = logs.from_err::<Error>().for_each(move |(_blk,logs)| {
                for log in logs.iter() {
                    let work = log_callback.handle_log(&log).map_err(move |e| {
                        error!("In callback {}: {}",index,e);
                    });
                    tokio::spawn(work);
                }
                Ok(())
            }).map_err(move |e| error!("In log-stream {}: {}",index,e));
            jobs.push(work);
        }
        future::select_all(jobs)
            .map_err(drop)
            .map(drop)
    });
    tokio::run(work);
    Ok(())
}

