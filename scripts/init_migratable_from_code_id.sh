#!/usr/bin/env bash
set -euo pipefail

# Initialize a new contract instance from an existing CODE_ID
# with an admin set so that future migrations are allowed.
#
# Usage examples:
#   ./scripts/init_migratable_from_code_id.sh 2026
#   CODE_ID=2026 ./scripts/init_migratable_from_code_id.sh
#   VERIFIER_PUBKEY=0x... CODE_ID=66 CHAIN_ID=xion-mainnet-1 \
#     RPC_NODE=https://rpc.xion-mainnet-1.burnt.com:443 ./scripts/init_migratable_from_code_id.sh 66
#
# You can override defaults via environment variables:
#   WALLET, ADMIN_ADDR, CHAIN_ID, RPC_NODE, INIT_MSG, LABEL, VERIFIER_PUBKEY

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

CODE_ID="${1:-${CODE_ID:-}}"
if [ -z "${CODE_ID:-}" ]; then
  echo "ERROR: CODE_ID is required. Pass as first argument or set CODE_ID env."
  exit 1
fi

WALLET="${WALLET:-tbh-test}"
CHAIN_ID="${CHAIN_ID:-xion-testnet-2}"
RPC_NODE="${RPC_NODE:-https://rpc.xion-testnet-2.burnt.com:443}"

# Admin address that will be allowed to migrate the contract in the future.
ADMIN_ADDR="${ADMIN_ADDR:-$(xiond keys show "$WALLET" -a)}"

# Default init msg matches README; override INIT_MSG to customize, or set VERIFIER_PUBKEY only.
VERIFIER_PUBKEY="${VERIFIER_PUBKEY:-0x03a6a96da6e704f74f53b3a98e0ae37123abf5a96803d8d971795637b0034a60cf}"
INIT_MSG="${INIT_MSG:-{\"owner\":null,\"verifier_pubkey\":\"$VERIFIER_PUBKEY\"}}"

LABEL="${LABEL:-tradeos-cw-sc-migratable}"

echo "==> Instantiating migratable contract"
echo "    CODE_ID:    $CODE_ID"
echo "    WALLET:     $WALLET"
echo "    ADMIN_ADDR: $ADMIN_ADDR"
echo "    CHAIN_ID:   $CHAIN_ID"
echo "    RPC_NODE:   $RPC_NODE"

RES=$(xiond tx wasm instantiate "$CODE_ID" "$INIT_MSG" \
  --from "$WALLET" \
  --admin "$ADMIN_ADDR" \
  --label "$LABEL" \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y \
  --chain-id "$CHAIN_ID" \
  --node "$RPC_NODE" \
  --output json)

echo "$RES"

TXHASH=$(echo "$RES" | jq -r '.txhash')
if [ -z "$TXHASH" ] || [ "$TXHASH" = "null" ]; then
  echo "ERROR: Failed to extract txhash from instantiate transaction response."
  exit 1
fi

echo "==> Instantiate txhash: $TXHASH"
echo "==> Querying chain for new contract address..."

QUERY_RES=""
MAX_RETRIES=15
SLEEP_SECONDS=4

for i in $(seq 1 "$MAX_RETRIES"); do
  echo "   -> Attempt $i/$MAX_RETRIES to fetch tx..."
  if QUERY_RES=$(xiond query tx "$TXHASH" \
    --node "$RPC_NODE" \
    --output json 2>/dev/null); then
    if echo "$QUERY_RES" | jq -e '.events' >/dev/null 2>&1; then
      break
    fi
  fi
  sleep "$SLEEP_SECONDS"
done

if [ -z "$QUERY_RES" ]; then
  echo "ERROR: Failed to query instantiate tx $TXHASH after ${MAX_RETRIES} attempts."
  exit 1
fi

CONTRACT_ADDR=$(echo "$QUERY_RES" | jq -r '
  .events[]
  | select(.type=="instantiate")
  | .attributes[]
  | select(.key=="_contract_address")
  | .value')

if [ -z "$CONTRACT_ADDR" ] || [ "$CONTRACT_ADDR" = "null" ]; then
  echo "ERROR: Failed to extract contract address from tx $TXHASH"
  echo "$QUERY_RES" | jq '.' || true
  exit 1
fi

echo "==> New contract address: $CONTRACT_ADDR"
echo "Done."

