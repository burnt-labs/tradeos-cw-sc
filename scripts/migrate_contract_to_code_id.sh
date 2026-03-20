#!/usr/bin/env bash
set -euo pipefail

# Migrate an existing contract instance to a new CODE_ID.
# Requires that the contract was instantiated with an admin,
# and that WALLET controls the admin address.
#
# Usage examples:
#   ./scripts/migrate_contract_to_code_id.sh xion1... 2026
#   CONTRACT_OLD=xion1... NEW_CODE_ID=2026 ./scripts/migrate_contract_to_code_id.sh
#
# You can override defaults via environment variables:
#   WALLET, CHAIN_ID, RPC_NODE, MIGRATE_MSG

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

CONTRACT_OLD="${1:-${CONTRACT_OLD:-}}"
NEW_CODE_ID="${2:-${NEW_CODE_ID:-}}"

if [ -z "${CONTRACT_OLD:-}" ] || [ -z "${NEW_CODE_ID:-}" ]; then
  echo "ERROR: CONTRACT_OLD and NEW_CODE_ID are required."
  echo "Usage: $0 <contract_address> <new_code_id>"
  exit 1
fi

WALLET="${WALLET:-tbh-test}"
CHAIN_ID="${CHAIN_ID:-xion-testnet-2}"
RPC_NODE="${RPC_NODE:-https://rpc.xion-testnet-2.burnt.com:443}"

# Adjust MIGRATE_MSG according to your contract's migrate message schema.
MIGRATE_MSG="${MIGRATE_MSG:-{}}"

echo "==> Migrating contract"
echo "    CONTRACT_OLD: $CONTRACT_OLD"
echo "    NEW_CODE_ID:  $NEW_CODE_ID"
echo "    WALLET:       $WALLET"
echo "    CHAIN_ID:     $CHAIN_ID"
echo "    RPC_NODE:     $RPC_NODE"

xiond tx wasm migrate "$CONTRACT_OLD" "$NEW_CODE_ID" "$MIGRATE_MSG" \
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

