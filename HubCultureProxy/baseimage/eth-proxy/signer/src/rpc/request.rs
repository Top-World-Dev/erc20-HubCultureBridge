use rpc::transaction::{Transaction,TxCall};
use ethtokens::Request as TokenRequest;
use functions::Call;


#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Request {
    SignToken(TokenRequest),
    EncodeToken(TokenRequest),
    SignRawTx(Transaction),
    SignTxCall(TxCall),
    EncodeCall(Call),
    GetAddress { },
    GetContracts { },
}
