#!/usr/bin/env bash
# Builds the demo contract to WASM and asserts it fits the Stylus program limit.
# The onchain size is the brotli-compressed WASM, which is what activation bills,
# so that is what we measure here rather than the raw artifact.
set -euo pipefail

LIMIT=24576 # 24 KB, the Stylus program size limit

cargo build -p stylus-crypto-verify-demo --release --target wasm32-unknown-unknown
WASM=$(find target/wasm32-unknown-unknown/release -maxdepth 1 -name 'stylus_crypto_verify_demo.wasm' | head -1)

SIZE=$(python3 - "$WASM" <<'PY'
import brotli, sys
data = open(sys.argv[1], "rb").read()
print(len(brotli.compress(data, quality=11)))
PY
)

printf 'compressed contract size: %s bytes (%.1f KB), limit %s bytes\n' \
  "$SIZE" "$(echo "scale=1; $SIZE/1024" | bc)" "$LIMIT"

if [ "$SIZE" -ge "$LIMIT" ]; then
  echo "over the Stylus program size limit"
  exit 1
fi
echo "fits"
