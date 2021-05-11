use signer::options::SignerOptions;
use proxy::http::Uri;
use ethrpc::Url;

#[derive(Debug,Clone,StructOpt)]
pub struct SignerProxyOptions {
    /// Address of ethereum node
    #[structopt(name = "node-url",long="node-addr",default_value="ws://127.0.0.1:8546")]
    pub node_addr: Url,
    /// Delegate signing to remote
    #[structopt(name = "signer-url",long="remote-signer")]
    pub remote_signer: Option<Uri>,
    #[structopt(flatten)]
    pub signer: SignerOptions,
}

