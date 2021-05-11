//! Crptography types/utils
pub use mimir_crypto::traits::{Hashable,Hasher};
pub use mimir_crypto::keccak256::Keccak256;
pub use mimir_crypto::secp256k1::{
    Address,
    Signature,
    Secret,
    Signer,
    Error,
};
use types::H256;


/// Get the `keccak-256` hash of some `Hashable` type
///
pub fn keccak<H: Hashable + ?Sized>(hashable: &H) -> H256 {
    hashable.hash::<Keccak256>().into()
}
