extern crate mimir_common;
extern crate mimir_crypto;
extern crate ethrpc;
#[macro_use]
extern crate proxy;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate rand;
extern crate toml;
#[macro_use]
extern crate log;

pub mod contracts;
pub mod functions;
pub mod ethtokens;
pub mod options;
pub mod config;
pub mod crypto;
pub mod types;
pub mod util;
pub mod rpc;

mod error;

pub use error::Error;

use ethrpc::transaction::Transaction;
use contracts::Contracts;
use functions::Functions;
use ethtokens::EthTokens;
use options::SignerOptions;
use crypto::Address;
use types::Bytes;
use rpc::Request;


#[derive(Debug,Clone)]
pub struct Signer {
    address: Address,
    signer: crypto::Signer,
    contracts: Contracts,
    functions: Functions,
    ethtokens: EthTokens,
    allow_creation: bool,
    allow_raw: bool,
}


impl Signer {

    pub fn address(&self) -> Address {
        debug_assert!(self.address == self.signer.address());
        self.address
    }

    pub fn from_options(opt: &SignerOptions) -> Result<Self,Error> {
        let secret = opt.load_secret()?;
        let config = opt.load_config()?;
        let signer = crypto::Signer::new(secret)?;
        let address = signer.address();
        let mut contracts = Contracts::new(config.contracts);
        if let Some(addr) = opt.default_contract {
            contracts.set_default(addr)?;
        }
        let functions = config.functions.into_iter().collect();
        let ethtokens = config.ethtokens.into_iter().collect();
        let allow_creation = opt.allow_contract_creation;
        let allow_raw = opt.allow_raw_txns;
        info!("Initializing signer {}",address);
        if contracts.is_empty() {
            warn!("No contract whitelist specified; allowing all targets");
        }
        if let Some(addr) = contracts.get_default() {
            info!("Configured with default contract {}",addr);
        }
        if allow_raw {
            warn!("Allowing raw transactions for signer {}",address);
        }
        Ok(Self { address, signer, contracts, functions, ethtokens, allow_creation, allow_raw })
    }

    pub fn serve(&self, request: Request) -> rpc::Result {
        debug_assert!(self.address == self.signer.address());
        match request {
            Request::SignRawTx(body) => {
                let bytes = self.sign_raw_tx(body)?;
                Ok(bytes.into())
            },
            Request::SignTxCall(tx_call) => {
                let bytes = self.sign_tx_call(tx_call)?;
                Ok(bytes.into())
            },
            Request::EncodeCall(call) => {
                info!("{} encoding {:?}",self.address,call);
                let calldata = self.functions.try_encode(call)?;
                Ok(calldata.into())
            },
            Request::SignToken(req) => {
                info!("{} signing ethtoken {:?}",self.address,req);
                let encoded = self.ethtokens.try_encode(req)?;
                let hash = crypto::keccak(encoded.as_slice());
                let sig = self.signer.sign(&hash);
                Ok(sig.into())
            },
            Request::EncodeToken(req) => {
                info!("{} encoding ethtoken {:?}",self.address,req);
                let encoded = self.ethtokens.try_encode(req)?;
                Ok(encoded.into())
            },
            Request::GetContracts { } => {
                info!("{} serving {:?}",self.address,request);
                Ok(self.contracts.clone().into())
            },
            Request::GetAddress { } => {
                info!("{} serving {:?}",self.address,request);
                Ok(self.address.into())
            },
        }
    }


    fn assert_whitelisted(&self, addr: Address) -> Result<(),rpc::Error> {
        if self.contracts.is_allowed(addr) {
            Ok(())
        } else {
            let msg = format!("Address {} not in contract whitelist",addr);
            Err(rpc::Error::message(msg))
        }
    }


    fn sign_tx_call(&self, tx_call: rpc::TxCall) -> Result<Bytes,rpc::Error> {
        info!("{} serving {:?}",self.address,tx_call);
        let to_addr = if let Some(addr) = tx_call.to {
            self.assert_whitelisted(addr)?;
            addr
        } else if let Some(addr) = self.contracts.get_default() {
            debug_assert!(self.assert_whitelisted(addr).is_ok());
            addr
        } else {
            let msg = "Missing required field `to` (no default configured)";
            return Err(rpc::Error::message(msg));
        };
        let payable = self.functions.get(&tx_call.call.name)
            .map(|f| f.payable).unwrap_or(false);
        if payable || tx_call.value  == 0u32.into() {
            let calldata = self.functions.try_encode(tx_call.call)?;
            let tx = Transaction::builder()
                .nonce(tx_call.nonce)
                .gas_price(tx_call.gas_price)
                .gas_limit(tx_call.gas_limit)
                .to(Some(to_addr))
                .value(tx_call.value)
                .data(calldata)
                .sign(&self.signer);
            let encoded = tx.rlp();
            Ok(encoded)
        } else {
            let msg = format!("Nonzero value in call to non-payable function ({})",tx_call.value);
            Err(rpc::Error::message(msg))
        }
    } 

    fn sign_raw_tx(&self, body: rpc::Transaction) -> Result<Bytes,rpc::Error> {
        if self.allow_raw {
            if let Some(to_addr) = body.to {
                self.assert_whitelisted(to_addr)?;
            } else if !self.allow_creation {
                return Err(rpc::Error::message("contract-creation signing disabled"));
            }
            info!("{} signing raw {:?}",self.address,body);
            let tx = Transaction::builder()
                .nonce(body.nonce)
                .gas_price(body.gas_price)
                .gas_limit(body.gas_limit)
                .to(body.to)
                .value(body.value)
                .data(body.data)
                .sign(&self.signer);
            let encoded = tx.rlp();
            Ok(encoded)
        } else {
            warn!("{} denying raw {:?}",self.address,body);
            Err(rpc::Error::message("raw tx signing disabled"))
        }
    }
}

