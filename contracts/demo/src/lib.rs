//! Deployable Stylus contract that exposes `stylus-crypto-verify` over the
//! Solidity ABI, so the verification path can be called onchain and measured.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;
use stylus_crypto_verify::verify_ed25519_slices;
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    pub struct CryptoVerify {}
}

#[public]
impl CryptoVerify {
    /// Verifies an Ed25519 signature. Returns true only for a valid signature
    /// by `pubkey` over `message`; malformed input returns false.
    ///
    /// `pubkey` is 32 bytes, `signature` is 64 bytes; both arrive as dynamic
    /// bytes so the ABI stays simple for callers in either Solidity or Rust.
    pub fn verify_ed25519(&self, message: Vec<u8>, pubkey: Vec<u8>, signature: Vec<u8>) -> bool {
        verify_ed25519_slices(&message, &pubkey, &signature)
    }
}
