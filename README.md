# stylus-crypto-verify

[![CI](https://github.com/mariano-aguero/stylus-crypto-verify/actions/workflows/ci.yml/badge.svg)](https://github.com/mariano-aguero/stylus-crypto-verify/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/stylus-crypto-verify)](https://crates.io/crates/stylus-crypto-verify)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Cheap onchain Ed25519 signature verification for Arbitrum Stylus contracts. Measured at **5.1x cheaper than the same verification in Solidity**, with the WASM size findings that decide what else fits.

Ed25519 verification is prohibitive in Solidity and modest on [Stylus](https://arbitrum.io/stylus), where an audited Rust crate compiles to WASM. This repo ships that as a publishable `no_std` library plus a deployable demo contract.

```
crates/stylus-crypto-verify   the publishable library: verify_ed25519 (no_std, no stylus-sdk)
contracts/demo                a deployable Stylus contract wrapping the library
```

## Measured gas

Verifying one Ed25519 signature, both contracts taking identical calldata, both accepting the RFC 8032 vector and rejecting a tampered message before any number was recorded:

| Implementation | Gas | Relative |
| --- | --- | --- |
| Stylus, this crate | **136,343** | 1.0x |
| Solidity, [chengwenxi/Ed25519](https://github.com/chengwenxi/Ed25519) | 701,492 | 5.1x more |

Reproduce it with `./scripts/benchmark.sh`, which starts a local Arbitrum Nitro devnode, deploys both contracts, asserts they agree on a valid and an invalid signature, and prints the table. It needs Docker, Foundry and `cargo stylus`, but no funded wallet and no testnet.

The Solidity implementation is vendored under `benchmarks/solidity/src` (Apache-2.0, unmodified apart from a thin wrapper that gives it the same ABI as the Stylus contract).

## What fits on Stylus

Stylus caps a program at **24 KB** (the brotli-compressed WASM). The reason this repo is Ed25519-first is a measured one:

| Verification | Implementation | Compressed size | Fits in 24 KB |
| --- | --- | --- | --- |
| Ed25519 | `ed25519-dalek` 3.0, default features off | **17.8 KB** | Yes |
| BLS12-381 pairing | `bls12_381` (zkcrypto) 0.8 | 66.9 KB | No |
| BLS12-381 pairing | `arkworks` 0.5 | 91.7 KB | No |

BLS pairing does not fit in a standard Stylus program today, with either mainstream Rust implementation, and that is before hash-to-curve. The multi-fragment deployment path is new and was not yet reliable when measured. BLS is therefore deferred; this is a documented negative result, not an omission.

Measured with `stylus-sdk` 0.10, `cargo-stylus` 0.10.7, Rust 1.91, release profile `opt-level = "z"`, LTO on.

## The library

See [`crates/stylus-crypto-verify`](crates/stylus-crypto-verify) for the API. In short: `verify_ed25519` and `verify_ed25519_slices`, both `#[must_use] -> bool`, strict verification, malformed input returns `false` rather than panicking.

## Build and test

```bash
cargo test -p stylus-crypto-verify   # RFC 8032 vectors, malleability and small-order edge cases
cargo clippy -p stylus-crypto-verify --all-targets
./scripts/check-size.sh              # builds the demo to WASM and asserts it fits 24 KB
./scripts/benchmark.sh               # gas comparison on a local devnode
```

The size script needs Python with the `brotli` module (`pip install brotli`), matching how Stylus measures onchain size.

## Status

Ed25519 verification, the demo contract, the size findings and the gas benchmark are complete, tested and reproducible. Publishing the library to crates.io is the remaining step.

## License

[MIT](LICENSE)
