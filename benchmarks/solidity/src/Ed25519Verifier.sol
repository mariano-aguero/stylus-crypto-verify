// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.6.8;
pragma experimental ABIEncoderV2;

import "./Ed25519.sol";

/// Deployable wrapper around the Solidity Ed25519 library, exposing exactly the
/// same ABI as the Stylus contract (message, pubkey, signature as bytes) so the
/// two can be measured on identical calldata.
contract Ed25519Verifier {
    function verifyEd25519(
        bytes memory message,
        bytes memory pubkey,
        bytes memory signature
    ) public pure returns (bool) {
        if (pubkey.length != 32 || signature.length != 64) {
            return false;
        }
        bytes32 k;
        bytes32 r;
        bytes32 s;
        assembly {
            k := mload(add(pubkey, 32))
            r := mload(add(signature, 32))
            s := mload(add(signature, 64))
        }
        return Ed25519.verify(k, r, s, message);
    }
}
