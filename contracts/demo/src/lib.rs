//! Deployable Stylus contract that exposes `stylus-crypto-verify` over the
//! Solidity ABI, so the verification path can be called onchain and measured.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use stylus_crypto_verify::verify_ed25519_slices;
use stylus_sdk::{abi::Bytes, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct CryptoVerify {}
}

#[public]
impl CryptoVerify {
    /// Verifies an Ed25519 signature. Returns true only for a valid signature
    /// by `pubkey` over `message`; malformed input returns false.
    ///
    /// `pubkey` is 32 bytes, `signature` is 64 bytes. Parameters are `bytes`,
    /// not `uint8[]`: the array form pads every byte to a 32 byte word, which
    /// would multiply the calldata cost of a signature by roughly sixteen.
    pub fn verify_ed25519(&self, message: Bytes, pubkey: Bytes, signature: Bytes) -> bool {
        verify_ed25519_slices(&message, &pubkey, &signature)
    }
}
