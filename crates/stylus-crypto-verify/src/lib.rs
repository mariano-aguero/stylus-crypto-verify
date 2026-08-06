//! Cheap onchain Ed25519 signature verification for Arbitrum Stylus contracts.
//!
//! Verifying an Ed25519 signature in Solidity means reimplementing the curve
//! by hand, which is large and costly. On Stylus the audited `ed25519-dalek`
//! crate compiles to WASM and runs at a fraction of that cost. This crate wraps
//! it in one `no_std` function so any Stylus contract can verify a signature
//! without pulling in an entrypoint or writing crypto of its own.
//!
//! ```
//! use stylus_crypto_verify::verify_ed25519;
//!
//! let message = b"hello";
//! let pubkey = [0u8; 32];
//! let signature = [0u8; 64];
//!
//! let ok = verify_ed25519(message, &pubkey, &signature);
//! assert!(!ok); // an all-zero key and signature do not verify
//! ```
#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

use ed25519_dalek::{Signature, VerifyingKey};

/// A 32-byte Ed25519 public key, the on-the-wire encoding from RFC 8032.
pub type PublicKeyBytes = [u8; 32];

/// A 64-byte Ed25519 signature (R concatenated with S).
pub type SignatureBytes = [u8; 64];

/// Verifies an Ed25519 signature over `message`.
///
/// Returns `true` only when `signature` is a valid signature by `public_key`
/// over `message`. A malformed key or signature returns `false` rather than
/// panicking, so a contract can revert gracefully on bad input.
///
/// Verification is strict: it rejects signature malleability and small-order
/// public keys, matching the guarantees consensus-adjacent code needs. This is
/// [`VerifyingKey::verify_strict`], not the looser default `verify`.
#[must_use]
pub fn verify_ed25519(
    message: &[u8],
    public_key: &PublicKeyBytes,
    signature: &SignatureBytes,
) -> bool {
    let Ok(key) = VerifyingKey::from_bytes(public_key) else {
        return false;
    };
    let sig = Signature::from_bytes(signature);
    key.verify_strict(message, &sig).is_ok()
}

/// Convenience wrapper accepting slices, for callers that decode signatures
/// from dynamic ABI types. Returns `false` when either slice has the wrong
/// length instead of forcing the caller to check first.
#[must_use]
pub fn verify_ed25519_slices(message: &[u8], public_key: &[u8], signature: &[u8]) -> bool {
    let (Ok(key), Ok(sig)) = (
        <&PublicKeyBytes>::try_from(public_key),
        <&SignatureBytes>::try_from(signature),
    ) else {
        return false;
    };
    verify_ed25519(message, key, sig)
}
