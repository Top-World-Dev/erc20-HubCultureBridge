use ethrpc::abi::Function;
use ethtokens::EthToken;
use crypto::Address;
use std::collections::HashSet;


#[derive(Default,Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    #[serde(default,rename = "contract-whitelist")]
    pub contracts: HashSet<Address>,
    #[serde(default,rename = "function-config")]
    pub functions: Vec<Function>,
    #[serde(default,rename = "ethtoken-config")]
    pub ethtokens: Vec<EthToken>,
}

