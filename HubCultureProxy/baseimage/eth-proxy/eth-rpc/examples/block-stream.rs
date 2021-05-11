extern crate tokio;
extern crate ethrpc;


use std::time::Duration;
use tokio::prelude::*;
use std::env;

fn main() {

    let uri = env::args().skip(1).next().map(|s| s.parse().unwrap())
        .unwrap_or_else(|| "ws://127.0.0.1:8546".parse().unwrap());

    println!("Connecting to {}",uri);

    let work = ethrpc::connect(uri).and_then(|api| {
        api.eth().block_number().and_then(move |num| {
            println!("Starting from block {}",num);
            let poll = Duration::from_millis(256);
            api.util().block_stream(num,poll,None).for_each(|block| {
                println!("Got {:?}",block);
                Ok(())
            })
        })
    }).map_err(|e| eprintln!("ERROR: {}",e));

    tokio::run(work);
}
