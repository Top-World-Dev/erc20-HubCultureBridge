use crypto::{Address,Signature};
use types::{Bytes,H256};
use contracts::Contracts;

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(untagged)]
pub enum Response {
    Contracts(Contracts),
    Addr(Address),
    Hash(H256),
    Sig(Signature),
    Bytes(Bytes),  
}


impl Response {

    pub fn to_contracts(self) -> Result<Contracts,Self> {
        match self {
            Response::Contracts(c) => Ok(c),
            other => Err(other),
        }
    }

    pub fn to_addr(self) -> Result<Address,Self> {
        match self {
            Response::Addr(addr) => Ok(addr),
            other => Err(other)
        }
    }

    pub fn to_hash(self) -> Result<H256,Self> {
        match self {
            Response::Hash(hash) => Ok(hash),
            other => Err(other),
        }
    }

    pub fn to_sig(self) -> Result<Signature,Self> {
        match self {
            Response::Sig(sig) => Ok(sig),
            other => Err(other),
        }
    }

    pub fn to_bytes(self) -> Result<Bytes,Self> {
        match self {
            Response::Addr(addr) => {
                let mut buff = Vec::with_capacity(addr.len());
                buff.extend_from_slice(&addr);
                Ok(buff.into())
            },
            Response::Hash(hash) => {
                let mut buff = Vec::with_capacity(hash.len());
                buff.extend_from_slice(&hash);
                Ok(buff.into())
            },
            Response::Sig(sig) => {
                let mut buff = Vec::with_capacity(sig.len());
                buff.extend_from_slice(&sig);
                Ok(buff.into())
            },
            Response::Bytes(bytes) => Ok(bytes),
            other => Err(other),
        }
    }
}


impl From<Contracts> for Response {

    fn from(c: Contracts) -> Self {
        Response::Contracts(c)
    }
}

impl From<Bytes> for Response {

    fn from(bytes: Bytes) -> Self {
        Response::Bytes(bytes)
    }
}

impl From<H256> for Response {

    fn from(hash: H256) -> Self {
        Response::Hash(hash)
    }
}

impl From<Address> for Response {

    fn from(addr: Address) -> Self {
        Response::Addr(addr)
    }
}

impl From<Signature> for Response {

    fn from(sig: Signature) -> Self {
        Response::Sig(sig)
    }
}

