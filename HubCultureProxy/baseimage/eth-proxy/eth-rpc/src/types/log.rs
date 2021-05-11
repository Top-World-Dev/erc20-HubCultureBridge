use types::{U256,H256,Bytes};
use smallvec::SmallVec;
use crypto::Address;


/// A log produced by a transaction.
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct Log {
    /// Address of contract from which log originated
    pub address: Address,
    /// Log topic hashes
    pub topics: SmallVec<[H256;4]>,
    /// Non-indexed arguments of log
    pub data: Bytes,
    /// Block Hash
    #[serde(rename = "blockHash")]
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<H256>,
    /// Block Number
    #[serde(rename = "blockNumber")]
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub block_number: Option<U256>,
    /// Transaction Hash
    #[serde(rename = "transactionHash")]
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<H256>,
    /// Transaction Index
    #[serde(rename = "transactionIndex")]
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub transaction_index: Option<U256>,
    /// Log Index in Block
    #[serde(rename = "logIndex")]
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub log_index: Option<U256>,
    /// Log Index in Transaction
    #[serde(rename = "transactionLogIndex")]
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub transaction_log_index: Option<U256>,
    ///// Log Type
    //#[serde(rename = "logType")]
    //#[serde(default,skip_serializing_if = "Option::is_none")]
    //pub log_type: Option<String>,
    /// If true, log was removed due to reorg
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub removed: Option<bool>,
}

