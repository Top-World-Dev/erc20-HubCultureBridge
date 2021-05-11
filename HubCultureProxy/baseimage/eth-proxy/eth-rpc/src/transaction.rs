/// Helpers for off-chain transaction signing. 
///
use crypto::{self,Signature,Address,Signer};
use types::{U256,Bytes};
use rand::{Rand,Rng};
use util;
use _rlp;


/// A transaction body.
///
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct Body {
    /// Transaction nonce
    pub nonce: U256,
    /// Gas price (in wei)
    #[serde(rename = "gasPrice")]
    pub gas_price: U256,
    /// Gas limit
    #[serde(rename = "gas")]
    pub gas_limit: U256,
    /// To address (`None` if contract-creation)
    pub to: Option<Address>,
    /// Value (in wei)
    pub value: U256,
    /// Calldata (init code if contract-creation)
    pub data: Bytes,
}


impl Body {

    pub fn rlp(&self) -> Bytes {
        let params: [&[u8];6] = [
            util::trim(&self.nonce),
            util::trim(&self.gas_price),
            util::trim(&self.gas_limit),
            self.to.as_ref().map(|a| a.as_ref())
                .unwrap_or(&[]),
            util::trim(&self.value),
            &self.data
        ];
        let encoded = _rlp::encode_list::<&[u8],&[u8]>(&params);
        Bytes::from(encoded)
    }
}

impl Rand for Body {

    fn rand<R: Rng>(rng: &mut R) -> Self {
        Body {
            nonce: rng.gen(),
            gas_price: rng.gen(),
            gas_limit: rng.gen(),
            to: Some(rng.gen()),
            value: rng.gen(),
            data: rng.gen(),
        }
    }
}

/// A signed transaction.
///
#[derive(Hash,Debug,Clone,PartialEq,Eq)]
pub struct Transaction {
    body: Body,
    sig: Signature,
}


impl Transaction {

    pub fn builder() -> Builder { Default::default() }

    pub fn new(body: Body, signer: &Signer) -> Self {
        let encoded = body.rlp();
        let hash = crypto::keccak(encoded.as_slice());
        let sig = signer.sign(&hash.into_inner());
        Self { body, sig }
    }

    pub fn body(&self) -> &Body { &self.body }

    pub fn sig(&self) -> Signature { self.sig }

    pub fn rlp(&self) -> Bytes {
        let (v,r,s) = (
            self.sig.get_v(),
            self.sig.get_r(),
            self.sig.get_s(),
            );
        let v: [u8;1] = [v];
        let params: [&[u8];9] = [
            util::trim(&self.body.nonce),        // Scalar value
            util::trim(&self.body.gas_price),    // Scalar value
            util::trim(&self.body.gas_limit),    // Scalar value
            self.body.to.as_ref().map(|a| a.as_ref()) // Byte array (or empty if contract-creation)
                .unwrap_or(&[]),
            util::trim(&self.body.value),        // Scalar value
            &self.body.data,                     // Byte array
            util::trim(&v),                      // Scalar value
            util::trim(&r),                      // Scalar value
            util::trim(&s),                      // Scalar value
            ];
        let encoded = _rlp::encode_list::<&[u8],&[u8]>(&params);
        Bytes::from(encoded)
    }
}


/// Transaction builder.
///
#[derive(Hash,Default,Debug,Clone,PartialEq,Eq)]
pub struct Builder {
    nonce: Option<U256>,
    gas_price: Option<U256>,
    gas_limit: Option<U256>,
    to: Option<Option<Address>>,
    value: Option<U256>,
    data: Option<Bytes>,
}


impl Builder {

    pub fn nonce(&mut self, nonce: U256) -> &mut Self { self.nonce = Some(nonce); self }

    pub fn gas_price(&mut self, price: U256) -> &mut Self { self.gas_price = Some(price); self }

    pub fn gas_limit(&mut self, limit: U256) -> &mut Self { self.gas_limit = Some(limit); self }

    pub fn to(&mut self, to: Option<Address>) -> &mut Self { self.to = Some(to); self }

    pub fn value(&mut self, value: U256) -> &mut Self { self.value = Some(value); self }

    pub fn data(&mut self, data: Bytes) -> &mut Self { self.data = Some(data); self }

    pub fn sign(&mut self, signer: &Signer) -> Transaction {
        let body = self.to_body();
        Transaction::new(body,signer)
    }

    fn to_body(&mut self) -> Body {
        let nonce = self.nonce.take().unwrap_or_default();
        let gas_price = self.gas_price.take().unwrap_or_default();
        let gas_limit = self.gas_limit.take().unwrap_or_default();
        let to = self.to.take().unwrap_or_default();
        let value = self.value.take().unwrap_or_default();
        let data = self.data.take().unwrap_or_default();
        Body { nonce, gas_price, gas_limit, to, value, data }
    }
}

