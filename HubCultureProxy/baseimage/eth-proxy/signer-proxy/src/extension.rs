use rpc::{TxStatus,ExtRequest};
use ethrpc::types::H256;
use ethrpc::{self,api,Url};
use tokio::prelude::*;


/// Spawn a handler for extension-requests.
///
pub fn spawn(node: Url) -> impl ExtHandler<Error=Error> + Clone {
    let handler = Handler { node };
    move |req| { handler.handle_ext(req) }
}

pub type Error = api::Error;

/// Handler for extension-requests.
///
pub trait ExtHandler {

    type Error: Send + 'static;

    type Future: Future<Item=Option<TxStatus>,Error=Self::Error> + Send + 'static;

    fn handle_ext(&self, req: ExtRequest) -> Self::Future;
}


impl<T,F> ExtHandler for T
        where T: Fn(ExtRequest) -> F, F: IntoFuture<Item=Option<TxStatus>>,
              F::Future: Send + 'static, F::Error: Send + 'static {

    type Error = <F as IntoFuture>::Error;

    type Future = <F as IntoFuture>::Future;

    fn handle_ext(&self, req: ExtRequest) -> Self::Future {
        (self)(req).into_future()
    }
}

#[derive(Debug,Clone)]
struct Handler {
    node: Url,
}


impl Handler {

    pub fn handle_ext(&self, req: ExtRequest) -> impl Future<Item=Option<TxStatus>,Error=Error> {
        match req {
            ExtRequest::GetTxStatus { hash } => self.get_tx_status(hash),
        }
    }

    pub fn get_tx_status(&self, tx_hash: H256) -> impl Future<Item=Option<TxStatus>,Error=Error> {
        let work = ethrpc::connect(self.node.clone()).and_then(move |api| {
            api.eth().get_tx_by_hash(tx_hash).and_then(move |rslt| {
                match rslt.map(|tx| (tx.block_number,tx.block_hash)) {
                    Some((Some(number),Some(hash))) => {
                        let work = api.eth().block_number().and_then(move |latest| {
                            if number <= latest {
                                let work = api.eth().get_tx_receipt(tx_hash).map(move |receipt| {
                                    let status = receipt.and_then(|r| r.status.map(From::from));
                                    Some(TxStatus::Mined {
                                        block_number: number,
                                        block_hash: hash,
                                        execution: status,
                                    })
                                });
                                future::Either::A(work)
                            } else {
                                future::Either::B(future::ok(Some(TxStatus::Pending { })))
                            }
                        });
                        future::Either::A(work)
                    },
                    Some((_,_)) => future::Either::B(future::ok(Some(TxStatus::Pending { }))),
                    None => future::Either::B(future::ok(None)),
                }
            })
        });
        work
    }

    /*
    pub fn get_tx_status(&self, hash: H256) -> impl Future<Item=Option<TxStatus>,Error=Error> {
        let work = ethrpc::connect(self.node.clone()).and_then(move |api| {
            api.eth().get_tx_by_hash(hash).and_then(move |rslt| {
                match rslt.map(|tx| (tx.block_number,tx.block_hash)) {
                    Some((Some(number),Some(hash))) => {
                        let work = api.eth().block_number().map(move |latest| {
                            if number <= latest {
                                Some(TxStatus::Mined {
                                    block_number: number,
                                    block_hash: hash,
                                    execution: None, // TODO: Implement execution status lookup.
                                })
                            } else {
                                Some(TxStatus::Pending { })
                            }
                        });
                        future::Either::A(work)
                    },
                    Some((_,_)) => future::Either::B(future::ok(Some(TxStatus::Pending { }))),
                    None => future::Either::B(future::ok(None)),
                }
            })
        });
        work
    }
    */
}

