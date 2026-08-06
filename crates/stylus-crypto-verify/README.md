# stylus-crypto-verify

Cheap onchain Ed25519 signature verification for Arbitrum Stylus contracts.

Verifying an Ed25519 signature in Solidity means reimplementing the curve by hand, which is large and expensive. On [Arbitrum Stylus](https://arbitrum.io/stylus) the audited [`ed25519-dalek`](https://crates.io/crates/ed25519-dalek) crate compiles to WASM and runs at a fraction of that cost. This crate wraps it in one `no_std` function so any Stylus contract can verify a signature without pulling in an entrypoint or writing crypto of its own.

## Usage

```rust
use stylus_crypto_verify::verify_ed25519;

// pubkey: 32 bytes, signature: 64 bytes, both from RFC 8032 encodings.
let ok = verify_ed25519(message, &pubkey, &signature);
```

For dynamic ABI types where the lengths are not known at compile time:

```rust
use stylus_crypto_verify::verify_ed25519_slices;

// Returns false on a wrong-length slice instead of forcing a length check.
let ok = verify_ed25519_slices(&message, &pubkey_bytes, &signature_bytes);
```

## Strict by default

Verification uses [`verify_strict`](https://docs.rs/ed25519-dalek), not the looser default. It rejects signature malleability (non-canonical `S`), small-order public keys, and small-order `R`. That is the behavior consensus-adjacent and authorization code needs; the test suite proves each rejection, including the `S + L` malleability case.

Malformed input (a wrong-length key or signature, a non-decompressible point) returns `false` rather than panicking, so a contract can revert gracefully. The crate is `#![no_std]` and `#![forbid(unsafe_code)]`.

## License

[MIT](LICENSE)
