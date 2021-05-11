use functions::Call;
use types::{U256,Bytes};
use crypto::Address;


/// A contract-calling transaction.
///
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct TxCall {
    /// Transaction nonce (required)
    pub nonce: U256,

    /// Gas prices (required)
    #[serde(rename = "gasPrice")]
    pub gas_price: U256,

    /// Gas limit (default: 90000)
    #[serde(rename = "gas",default = "default_gas_limit")]
    pub gas_limit: U256,

    #[serde(default,skip_serializing_if="Option::is_none")]
    /// Destination address (required if default contract is unset)
    pub to: Option<Address>,

    /// Transaction value in wei (default: 0)
    #[serde(default)]
    pub value: U256,

    /// Description of a contract call
    pub call: Call,
}


/// A transaction body.
///
/// This type is identical to `ethrpc::transaction::Body` except
/// that it handles default values differently.
///
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct Transaction {
    /// Transaction nonce (required)
    pub nonce: U256,

    /// Gas prices (required)
    #[serde(rename = "gasPrice")]
    pub gas_price: U256,
   
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

