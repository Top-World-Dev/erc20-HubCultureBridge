/// Signer RPC
///

mod transaction;
mod error;

pub use self::transaction::{
    TxCall,
    Transaction,
};
pub use self::error::Error;

pub use signer::rpc::{
    Request as BaseRequest,
    Response as BaseResponse,
};

use ethrpc::types::{H256,U256,Status};
use signer::functions::Call as FunctionCall;
use signer::ethtokens::Request as TokenRequest;
use signer::rpc;


/// Transaction status report.
///
#[derive(Hash,Debug,Copy,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TxStatus {
    Pending { },
    #[serde(rename_all = "kebab-case")]
    Mined {
        block_number: U256,
        block_hash: H256,
        #[serde(default)]
        execution: Option<CallStatus>,
    },
}

/// Evm call status.
///
#[derive(Hash,Debug,Copy,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CallStatus {
    Success,
    Failure,
}

impl From<Status> for CallStatus {

    fn from(status: Status) -> Self {
        match status {
            Status::Success => CallStatus::Success,
            Status::Failure => CallStatus::Failure,
        }
    }
}


/// An incoming request.
///
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(untagged)]
pub enum Request {
    Transact(TxRequest),
    Call(CallRequest),
    Ext(ExtRequest),
}


/// An outbound response.
///
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(untagged)]
pub enum Response {
    TxStatus(Option<TxStatus>),
    Base(BaseResponse),
}


impl From<Option<TxStatus>> for Response {

    fn from(status: Option<TxStatus>) -> Self { Response::TxStatus(status) }
}

impl From<H256> for Response {

    fn from(hash: H256) -> Self { Response::Base(hash.into()) }
}

impl From<BaseResponse> for Response {

    fn from(rsp: BaseResponse) -> Self { Response::Base(rsp) }
}

/// Request subset which generates transactions.
///
/// These requests must be handled in a semi-synchronous fashion in
/// order to ensure proper nonce/ordering for all submissions.
///
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TxRequest {
    SignRawTx(Transaction),
    SignTxCall(TxCall),
}


impl TxRequest {

    /// Convert to base request, inserting required values.
    ///
    pub fn seed(self, nonce: U256, gas_price: U256) -> BaseRequest {
        match self {
            TxRequest::SignRawTx(tx) => {
                let Transaction { gas_limit, to, value, data } = tx;
                let tx = rpc::Transaction { nonce, gas_price, gas_limit, to, value, data };
                BaseRequest::SignRawTx(tx)
            },
            TxRequest::SignTxCall(tx) => {
                let TxCall { gas_limit, to, value, call } = tx;
                let tx = rpc::TxCall { nonce, gas_price, gas_limit, to, value, call };
                BaseRequest::SignTxCall(tx)
            },
        }
    }
}


/// Request subset which does not generate transactions.
///
/// These requests may be processed in a fully asynchronous fashion
/// since they do not modify blockchain state.
///
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CallRequest {
    SignToken(TokenRequest),
    EncodeToken(TokenRequest),
    EncodeCall(FunctionCall),
    GetContracts { },
    GetAddress { },
}


impl CallRequest {

    /// Convert to base request.
    ///
    pub fn into_base(self) -> BaseRequest {
        match self {
            CallRequest::SignToken(req) => BaseRequest::SignToken(req),
            CallRequest::EncodeToken(req) => BaseRequest::EncodeToken(req),
            CallRequest::EncodeCall(req) => BaseRequest::EncodeCall(req),
            CallRequest::GetContracts { } => BaseRequest::GetContracts { },
            CallRequest::GetAddress { } => BaseRequest::GetAddress { },
        }
    }
}


/// Request subset not directly served by signer.
///
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExtRequest {
    GetTxStatus {
        hash: H256,
    }
}

