use signer::functions::Call;
use ethrpc::types::{U256,Bytes};
use ethrpc::crypto::Address;


/// A contract-calling transaction (minus nonce and gas-price).
///
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct TxCall {
    /// Gas limit (default: 90000)
    #[serde(rename = "gas",default = "default_gas_limit")]
    pub gas_limit: U256,

    /// Destination address
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub to: Option<Address>,

    /// Transaction value in wei (default: 0)
    #[serde(default)]
    pub value: U256,

    /// Description of a contract call
    pub call: Call,
}


/// A transaction body (minus nonce and gas-price).
///
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct Transaction {
    /// Gas limit (default: 90000)
    #[serde(rename = "gas",default = "default_gas_limit")]
    pub gas_limit: U256,

    /// Destination address (required unless contract-creation)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub to: Option<Address>,

    /// Transaction value in wei (default: 0)
    #[serde(default)]
    pub value: U256, 

    /// Transaction data (default: empty)
    #[serde(default)]
    pub data: Bytes,
}


/// Official default value for gas limit (per ethereum rpc spec).
///
fn default_gas_limit() -> U256 { U256::from(90000u32) }

