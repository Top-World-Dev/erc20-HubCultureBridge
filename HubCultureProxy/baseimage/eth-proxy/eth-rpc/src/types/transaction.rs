use types::{Log,Bytes,U256,H256};
use crypto::Address;

/// Basic description of a transaction
///
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct Transaction {
    /// Transaction nonce
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub nonce: Option<U256>,
    /// Gas price (in wei)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    #[serde(rename = "gasPrice")]
    pub gas_price: Option<U256>,
    /// Gas limit
    #[serde(default,skip_serializing_if = "Option::is_none")]
    #[serde(rename = "gas")]
    pub gas_limit: Option<U256>,
    /// Origin of transaction
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// To address (`None` if contract-creation)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub to: Option<Address>,
    /// Value (in wei)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,
    /// Calldata (init code if contract-creation)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub data: Option<Bytes>,
}


impl From<TxCall> for Transaction {

    fn from(call: TxCall) -> Self {
        Self {
            nonce: call.nonce,
            gas_price: call.gas_price,
            gas_limit: call.gas_limit,
            from: call.from,
            to: Some(call.to),
            value: call.value,
            data: call.data,
        }
    }
}


impl From<TxInfo> for Transaction {

    fn from(info: TxInfo) -> Self {
        Self {
            nonce: Some(info.nonce),
            gas_price: Some(info.gas_price),
            gas_limit: Some(info.gas_limit),
            from: Some(info.from),
            to: info.to,
            value: Some(info.value),
            data: Some(info.input),
        }
    }
}


/// Description of a transaction-call
///
/// Same as `Transaction` except that the `to` field is required.
///
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct TxCall {
    /// Transaction nonce
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub nonce: Option<U256>,
    /// Gas price (in wei)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    #[serde(rename = "gasPrice")]
    pub gas_price: Option<U256>,
    /// Gas limit
    #[serde(default,skip_serializing_if = "Option::is_none")]
    #[serde(rename = "gas")]
    pub gas_limit: Option<U256>,
    /// Origin of transaction
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// To address
    pub to: Address,
    /// Value (in wei)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,
    /// Calldata (init code if contract-creation)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub data: Option<Bytes>,
}


/// Description of a Transaction, pending or in the chain.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct TxInfo {
    /// Hash
    pub hash: H256,
    /// Nonce
    pub nonce: U256,
    /// Block hash. None when pending.
    #[serde(rename = "blockHash")]
    pub block_hash: Option<H256>,
    /// Block number. None when pending.
    #[serde(rename = "blockNumber")]
    pub block_number: Option<U256>,
    /// Transaction Index. None when pending.
    #[serde(rename = "transactionIndex")]
    pub transaction_index: Option<U256>,
    /// Sender
    pub from: Address,
    /// Recipient (None when contract creation)
    pub to: Option<Address>,
    /// Transfered value
    pub value: U256,
    /// Gas Price
    #[serde(rename = "gasPrice")]
    pub gas_price: U256, 
    /// Gas amount
    #[serde(rename = "gas")]
    pub gas_limit: U256,
    /// Input data
    pub input: Bytes,
}


/// "Receipt" of an executed transaction: details of its execution.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Receipt {
    /// Transaction hash.
    #[serde(rename = "transactionHash")]
    pub transaction_hash: H256,
    /// Index within the block.
    #[serde(rename = "transactionIndex")]
    pub transaction_index: U256,
    /// Hash of the block this transaction was included within.
    #[serde(rename = "blockHash")]
    pub block_hash: H256,
    /// Number of the block this transaction was included within.
    #[serde(rename = "blockNumber")]
    pub block_number: U256,
    /// Cumulative gas used within the block after this was executed.
    #[serde(rename = "cumulativeGasUsed")]
    pub cumulative_gas_used: U256,
    /// Gas used by this transaction alone.
    #[serde(rename = "gasUsed")]
    pub gas_used: U256,
    /// Contract address created, or `None` if not a deployment.
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<Address>,
    /// Logs generated within this transaction.
    pub logs: Vec<Log>,
    /// Execution status (post-byzantium)
    pub status: Option<Status>,
}


/// Contract execution exit-status
#[derive(Hash,Debug,Copy,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub enum Status {
    #[serde(rename = "0x1")]
    Success,
    #[serde(rename = "0x0")]
    Failure
}


impl Status {

    pub fn is_success(&self) -> bool {
        match self {
            Status::Success => true,
            Status::Failure => false,
        }
    }
}
