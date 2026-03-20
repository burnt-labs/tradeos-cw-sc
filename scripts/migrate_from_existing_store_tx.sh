#!/usr/bin/env bash
set -euo pipefail

# This script skips the store step and derives CODE_ID from an existing
# store-code tx, then migrates the target contract to that CODE_ID.

########################
# Configuration
########################

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

# Wallet key name used for signing migration tx
WALLET="${WALLET:-tbh-test}"

# Existing contract address to migrate.
# Override via environment variable when running the script:
#   CONTRACT_OLD=xion1... ./scripts/migrate_from_existing_store_tx.sh
CONTRACT_OLD="${CONTRACT_OLD:-xion1klcchukh5x62kr2v9cwkfd6edspu2x9jw7054pyf64gddn76havsajxpj5}"

CHAIN_ID="${CHAIN_ID:-xion-testnet-2}"

# RPC node to use for all xiond calls.
RPC_NODE="${RPC_NODE:-https://rpc.xion-testnet-2.burnt.com:443}"

# Store-code tx hash that already exists on-chain.
# You can override this with:
#   TXHASH=... ./scripts/migrate_from_existing_store_tx.sh
TXHASH="${TXHASH:-1198D054DB98B14B1E01D41B4C25B7471B7E7B6A2E061B51F867087A3AD62A0D}"

# If your migrate message requires fields, adjust this JSON accordingly.
MIGRATE_MSG="${MIGRATE_MSG:-{}}"

########################
# Derive CODE_ID from existing tx
########################

echo "==> Using existing store-code txhash: $TXHASH"
echo "==> Querying chain for CODE_ID from tx..."

QUERY_RES=""
MAX_RETRIES=20
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
  echo "ERROR: Failed to query tx $TXHASH after ${MAX_RETRIES} attempts."
  exit 1
fi

CODE_ID_NEW=$(echo "$QUERY_RES" | jq -r '
  .events[]
  | select(.type=="store_code")
  | .attributes[]
  | select(.key=="code_id")
  | .value')

if [ -z "$CODE_ID_NEW" ] || [ "$CODE_ID_NEW" = "null" ]; then
  echo "ERROR: Failed to extract CODE_ID from tx $TXHASH"
  echo "$QUERY_RES" | jq '.' || true
  exit 1
fi

echo "==> New CODE_ID derived from tx: $CODE_ID_NEW"

########################
# Migrate existing contract
########################

echo "==> Migrating contract $CONTRACT_OLD to CODE_ID $CODE_ID_NEW..."

xiond tx wasm migrate "$CONTRACT_OLD" "$CODE_ID_NEW" "$MIGRATE_MSG" \
  --from "$WALLET" \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y \
  --chain-id "$CHAIN_ID" \
  --node "$RPC_NODE"

echo "==> Migration transaction submitted."
echo "You can inspect contract history with:"
echo "  xiond query wasm contract-history $CONTRACT_OLD --output json --node $RPC_NODE"

