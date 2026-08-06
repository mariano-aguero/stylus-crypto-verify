//! Security edge cases: the properties `verify_strict` is chosen to guarantee.
//! Fixtures are generated deterministically from fixed seeds so the tests need
//! no network and no transcription of long vectors.

use ed25519_dalek::{Signer, SigningKey};
use stylus_crypto_verify::verify_ed25519;

/// Ed25519 group order L, little-endian, as used to build the S + L
/// malleability case. L = 2^252 + 27742317777372353535851937790883648493.
const L_LE: [u8; 32] = [
    0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
];

fn signer(seed_byte: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed_byte; 32])
}

#[test]
fn verifies_a_multi_block_message() {
    // 1023 bytes forces SHA-512 over several blocks, the concern behind
    // RFC 8032 TEST 4. A deterministic keypair signs it and the wrapper agrees.
    let key = signer(0x42);
    let message = [0xABu8; 1023];
    let sig = key.sign(&message);
    let pubkey = key.verifying_key().to_bytes();
    assert!(verify_ed25519(&message, &pubkey, &sig.to_bytes()));
}

#[test]
fn rejects_the_s_plus_l_malleability() {
    // Take a valid signature and add L to its S component (the high 32 bytes).
    // The result encodes the same signature mathematically but with a
    // non-canonical S. The permissive `verify` would accept it; `verify_strict`
    // must reject it, which is the whole reason we use strict verification.
    let key = signer(0x07);
    let message = b"malleability check";
    let sig = key.sign(message);
    let pubkey = key.verifying_key().to_bytes();
    let mut bytes = sig.to_bytes();
    assert!(
        verify_ed25519(message, &pubkey, &bytes),
        "the honest signature verifies"
    );

    // S occupies bytes 32..64, little-endian. Add L with carry.
    let mut carry = 0u16;
    for i in 0..32 {
        let sum = u16::from(bytes[32 + i]) + u16::from(L_LE[i]) + carry;
        bytes[32 + i] = sum as u8;
        carry = sum >> 8;
    }
    assert!(
        !verify_ed25519(message, &pubkey, &bytes),
        "S + L must be rejected"
    );
}

#[test]
fn rejects_a_small_order_public_key() {
    // The identity point, 0x01 followed by 31 zero bytes, is small order.
    // verify_strict rejects small-order keys regardless of the signature.
    let mut identity = [0u8; 32];
    identity[0] = 0x01;
    let key = signer(0x09);
    let message = b"anything";
    let sig = key.sign(message);
    assert!(!verify_ed25519(message, &identity, &sig.to_bytes()));
}

#[test]
fn rejects_an_all_zero_key_and_signature() {
    // The degenerate all-zero input must not verify. This is the doctest case,
    // called out explicitly because it is an easy accidental "empty auth".
    assert!(!verify_ed25519(b"hello", &[0u8; 32], &[0u8; 64]));
}

#[test]
fn rejects_a_signature_reused_for_a_different_key() {
    // A signature valid under one key must not verify under another, even when
    // both keys are well formed. Guards against key-substitution mistakes.
    let signer_a = signer(0x11);
    let signer_b = signer(0x22);
    let message = b"bound to one key";
    let sig = signer_a.sign(message);
    assert!(verify_ed25519(
        message,
        &signer_a.verifying_key().to_bytes(),
        &sig.to_bytes()
    ));
    assert!(!verify_ed25519(
        message,
        &signer_b.verifying_key().to_bytes(),
        &sig.to_bytes()
    ));
}
