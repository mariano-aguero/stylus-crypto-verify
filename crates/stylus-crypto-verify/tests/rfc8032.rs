//! Ed25519 verification against the RFC 8032 section 7.1 test vectors, plus
//! negative cases that must fail.

use stylus_crypto_verify::{verify_ed25519, verify_ed25519_slices};

struct Vector {
    public_key: &'static str,
    message: &'static str,
    signature: &'static str,
}

// The canonical RFC 8032 section 7.1 vectors (public key, message, signature),
// hex-encoded. Test 1 has an empty message; the others exercise 1, 2 and 1023
// byte messages.
const VECTORS: &[Vector] = &[
    Vector {
        public_key: "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
        message: "",
        signature: "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b",
    },
    Vector {
        public_key: "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
        message: "72",
        signature: "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00",
    },
    Vector {
        public_key: "fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025",
        message: "af82",
        signature: "6291d657deec24024827e69c3abe01a30ce548a284743a445e3680d7db5ac3ac18ff9b538d16f290ae67f760984dc6594a7c15e9716ed28dc027beceea1ec40a",
    },
];

fn decode<const N: usize>(hex_str: &str) -> [u8; N] {
    let bytes = hex::decode(hex_str).expect("valid hex");
    bytes.try_into().expect("expected length")
}

#[test]
fn accepts_rfc8032_vectors() {
    for (i, v) in VECTORS.iter().enumerate() {
        let public_key = decode::<32>(v.public_key);
        let message = hex::decode(v.message).expect("valid hex");
        let signature = decode::<64>(v.signature);
        assert!(
            verify_ed25519(&message, &public_key, &signature),
            "vector {i} should verify",
        );
    }
}

#[test]
fn rejects_a_flipped_message_bit() {
    let v = &VECTORS[1];
    let public_key = decode::<32>(v.public_key);
    let signature = decode::<64>(v.signature);
    let tampered = [0x73u8]; // vector 1 signs 0x72
    assert!(!verify_ed25519(&tampered, &public_key, &signature));
}

#[test]
fn rejects_a_flipped_signature_bit() {
    let v = &VECTORS[0];
    let public_key = decode::<32>(v.public_key);
    let mut signature = decode::<64>(v.signature);
    signature[0] ^= 0x01;
    assert!(!verify_ed25519(&[], &public_key, &signature));
}

#[test]
fn rejects_the_wrong_public_key() {
    let signed = &VECTORS[1];
    let other_key = decode::<32>(VECTORS[2].public_key);
    let message = hex::decode(signed.message).expect("valid hex");
    let signature = decode::<64>(signed.signature);
    assert!(!verify_ed25519(&message, &other_key, &signature));
}

#[test]
fn rejects_wrong_length_slices() {
    let v = &VECTORS[0];
    let public_key = decode::<32>(v.public_key);
    let signature = decode::<64>(v.signature);
    // Correct arrays through the slice API still verify.
    assert!(verify_ed25519_slices(&[], &public_key, &signature));
    // A 31-byte key and a 63-byte signature must be rejected, not truncated.
    assert!(!verify_ed25519_slices(&[], &public_key[..31], &signature));
    assert!(!verify_ed25519_slices(&[], &public_key, &signature[..63]));
}
