#!/usr/bin/env bash
# Measures Ed25519 verification gas, Stylus against pure Solidity, on a local
# Arbitrum Nitro devnode. No funded wallet and no testnet needed: the devnode
# ships prefunded accounts.
#
# Correctness is asserted before any number is printed. A verifier that reverted
# early would look cheap and mean nothing, so both contracts must accept the
# RFC 8032 vector and reject a tampered message before the gas is reported.
#
# Prerequisites: docker, foundry (cast, forge), cargo stylus.
set -euo pipefail

RPC=${RPC:-http://localhost:8547}
# The standard prefunded account of the Nitro dev node. Local only, never funded
# on a real network.
KEY=${DEV_KEY:-0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659}
NITRO_IMAGE=${NITRO_IMAGE:-offchainlabs/nitro-node:v3.11.2-3599aca}

# RFC 8032 section 7.1 TEST 2.
MSG=0x72
TAMPERED=0x73
PK=0x3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c
SIG=0x92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00
SELECTOR="verifyEd25519(bytes,bytes,bytes)(bool)"

root=$(cd "$(dirname "$0")/.." && pwd)

if ! curl -s -X POST -H 'Content-Type: application/json' \
  --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' "$RPC" >/dev/null 2>&1; then
  echo "starting a Nitro devnode on $RPC"
  docker run --rm -d --name nitro-dev -p 8547:8547 "$NITRO_IMAGE" \
    --dev --http.addr 0.0.0.0 --http.port 8547 --http.api=net,web3,eth,debug \
    --http.corsdomain='*' --http.vhosts='*' >/dev/null
  until curl -s -X POST -H 'Content-Type: application/json' \
    --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' "$RPC" >/dev/null 2>&1; do
    sleep 2
  done
fi

echo "deploying the Stylus contract"
stylus_addr=$(cd "$root/contracts/demo" && cargo stylus deploy \
  --private-key "$KEY" --endpoint "$RPC" --no-verify 2>&1 |
  grep -oE 'deployed code at address: .*0x[0-9a-fA-F]{40}' | grep -oE '0x[0-9a-fA-F]{40}')

echo "deploying the Solidity contract"
sol_addr=$(cd "$root/benchmarks/solidity" && forge create src/Ed25519Verifier.sol:Ed25519Verifier \
  --rpc-url "$RPC" --private-key "$KEY" --broadcast 2>&1 |
  grep -oE 'Deployed to: 0x[0-9a-fA-F]{40}' | grep -oE '0x[0-9a-fA-F]{40}')

check() { # address label
  local valid tampered
  valid=$(cast call "$1" "$SELECTOR" "$MSG" "$PK" "$SIG" --rpc-url "$RPC")
  tampered=$(cast call "$1" "$SELECTOR" "$TAMPERED" "$PK" "$SIG" --rpc-url "$RPC")
  if [ "$valid" != "true" ] || [ "$tampered" != "false" ]; then
    echo "$2 failed the correctness check (valid=$valid tampered=$tampered)"
    exit 1
  fi
}

check "$stylus_addr" stylus
check "$sol_addr" solidity
echo "both implementations accept the RFC 8032 vector and reject a tampered message"

stylus_gas=$(cast estimate "$stylus_addr" "$SELECTOR" "$MSG" "$PK" "$SIG" --rpc-url "$RPC")
sol_gas=$(cast estimate "$sol_addr" "$SELECTOR" "$MSG" "$PK" "$SIG" --rpc-url "$RPC")

printf '\nverify one Ed25519 signature, identical calldata:\n'
printf '  stylus:   %s gas\n' "$stylus_gas"
printf '  solidity: %s gas\n' "$sol_gas"
printf '  ratio:    %sx cheaper on stylus\n' "$(awk "BEGIN { printf \"%.1f\", $sol_gas/$stylus_gas }")"
