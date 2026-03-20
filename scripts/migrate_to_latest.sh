#!/usr/bin/env bash
set -euo pipefail

# This script uploads the latest optimized TradeOS contract wasm to XION testnet
# and migrates an existing contract instance to the new code ID.
#
# Prerequisites:
# - Docker installed (for cosmwasm/optimizer, if you haven't built artifacts yet)
# - xiond installed and configured (client.toml should point to a working RPC)
# - Local key "tbh-test" available and funded on xion-testnet-2

########################
# Configuration
########################

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Wallet key name used for signing transactions
WALLET="${WALLET:-tbh-test}"

# Existing contract address to migrate.
# Override via environment variable when running the script:
#   CONTRACT_OLD=xion1... ./scripts/migrate_to_latest.sh
CONTRACT_OLD="${CONTRACT_OLD:-xion1klcchukh5x62kr2v9cwkfd6edspu2x9jw7054pyf64gddn76havsajxpj5}"

CHAIN_ID="${CHAIN_ID:-xion-testnet-2}"

# RPC node to use for all xiond calls.
# Matches official docs: https://docs.burnt.com/xion/developers/section-overview/public-endpoints-and-resources
RPC_NODE="${RPC_NODE:-https://rpc.xion-testnet-2.burnt.com:443}"

# If your migrate message requires fields, adjust this JSON accordingly.
MIGRATE_MSG="${MIGRATE_MSG:-{}}"

########################
# Build optimized wasm
########################

cd "$PROJECT_DIR"

if [ ! -f ./artifacts/tradeos_cw_sc.wasm ]; then
  echo "==> Building optimized wasm with cosmwasm/optimizer..."
  docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/optimizer:0.16.1
else
  echo "==> Using existing ./artifacts/tradeos_cw_sc.wasm"
fi

########################
# Store code on-chain
########################

echo "==> Storing latest wasm on-chain (wallet: $WALLET, chain-id: $CHAIN_ID)..."

RES=$(xiond tx wasm store ./artifacts/tradeos_cw_sc.wasm \
  --chain-id "$CHAIN_ID" \
  --gas-adjustment 1.3 \
  --gas-prices 0.001uxion \
  --gas auto \
  -y --output json \
  --node "$RPC_NODE" \
  --from "$WALLET")

echo "$RES"

TXHASH=$(echo "$RES" | jq -r '.txhash')
if [ -z "$TXHASH" ] || [ "$TXHASH" = "null" ]; then
  echo "ERROR: Failed to extract txhash from store-code transaction response."
  exit 1
fi

echo "==> Store-code txhash: $TXHASH"

########################
# Extract new CODE_ID
########################

echo "==> Querying chain for new CODE_ID..."

QUERY_RES=""
MAX_RETRIES=15
SLEEP_SECONDS=4

for i in $(seq 1 "$MAX_RETRIES"); do
  echo "   -> Attempt $i/$MAX_RETRIES to fetch tx..."
  # swallow errors in case tx is not yet indexed
  if QUERY_RES=$(xiond query tx "$TXHASH" \
    --node "$RPC_NODE" \
    --output json 2>/dev/null); then
    # basic sanity check that events exist
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
  exit 1
fi

echo "==> New CODE_ID: $CODE_ID_NEW"

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
echo "  xiond query wasm contract-history $CONTRACT_OLD --output json"

