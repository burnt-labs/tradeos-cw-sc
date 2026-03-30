# TradeOS Cosmwasm Smart Contract

This is the Cosmwasm Smart Contract for the TradeOS project.

## Deployment Information

### XION Mainnet

- **Network**: `xion-mainnet-1`
- **RPC Endpoint**: `https://rpc.xion-mainnet-1.burnt.com:443`
- **Chain ID**: `xion-mainnet-1`
- **Governance (store code)**: [Proposal #56](https://www.mintscan.io/xion/proposals/56) — forum: [Deploy TradeOS CosmWasm on XION](https://discourse.xion.burnt.com/t/deploy-tradeos-cosmwasm-claim-contract-on-xion/110)
- **On-chain Code ID**: `66` (WASM checksum / `data_hash`: `45055180AB8DCCBF88F27FEC06765F58F95D55199480374377A61481E965315C`)
- **Store execution**: Governance runs `MsgStoreCode` in **`EndBlock`** when the proposal passes (no separate user "store" transaction). For this deployment the `store_code` event is on block **`20030788`** (`2026-03-28T14:02:18Z`).

#### Mainnet instances

There is no single canonical mainnet contract address—each integrator instantiates from **code ID `66`** and sets their own `verifier_pubkey` (or updates it with `set_verifier`).

### XION Testnet

- **Network**: `xion-testnet-2`
- **RPC Endpoint**: `https://rpc.xion-testnet-2.burnt.com:443`
- **Chain ID**: `xion-testnet-2`

#### Testnet reference instance

- **Deployed Code ID**: `2026`
- **Contract Address**: `xion1lf0j559uuj9uh6dx8apwnf9mvcxpjjv66dxm5k7gny8430e96d2qdz5ueq`
- **Owner / Admin**: `xion1epzznazp28up4asses7jdcyqnw3n8lu7f5g9xs` (Deployed by Bohao's wallet)
- **Verifier Pubkey** (example / test): `0x03a6a96da6e704f74f53b3a98e0ae37123abf5a96803d8d971795637b0034a60cf`

### Instantiate Contract

Use a **33-byte compressed secp256k1** public key for the verifier (hex with optional `0x`, or base64). Set `VERIFIER_PUBKEY` explicitly on mainnet; the default below matches the **testnet example** only.

1. **Instantiate on mainnet** (code ID **66**)

   ```bash
   CODE_ID="66"
   CHAIN_ID="xion-mainnet-1"
   RPC_NODE="https://rpc.xion-mainnet-1.burnt.com:443"
   VERIFIER_PUBKEY="0x..."   # required: your verifier pubkey (do not reuse unrelated examples)

   WALLET="your-wallet-id"
   ADMIN_ADDR=$(xiond keys show "$WALLET" -a)
   MSG="{\"owner\":null,\"verifier_pubkey\":\"${VERIFIER_PUBKEY}\"}"

   xiond tx wasm instantiate "$CODE_ID" "$MSG" \
     --from "$WALLET" \
     --admin "$ADMIN_ADDR" \
     --label "tradeos-cw-sc" \
     --gas-prices 0.025uxion \
     --gas auto \
     --gas-adjustment 1.3 \
     -y \
     --chain-id "$CHAIN_ID" \
     --node "$RPC_NODE"
   ```

2. **Instantiate on testnet** (code ID **2026**)

   ```bash
   CODE_ID="2026"
   CHAIN_ID="xion-testnet-2"
   RPC_NODE="https://rpc.xion-testnet-2.burnt.com:443"
   VERIFIER_PUBKEY="${VERIFIER_PUBKEY:-0x03a6a96da6e704f74f53b3a98e0ae37123abf5a96803d8d971795637b0034a60cf}"

   WALLET="your-wallet-id"
   ADMIN_ADDR=$(xiond keys show "$WALLET" -a)
   MSG="{\"owner\":null,\"verifier_pubkey\":\"${VERIFIER_PUBKEY}\"}"

   xiond tx wasm instantiate "$CODE_ID" "$MSG" \
     --from "$WALLET" \
     --admin "$ADMIN_ADDR" \
     --label "tradeos-cw-sc" \
     --gas-prices 0.025uxion \
     --gas auto \
     --gas-adjustment 1.3 \
     -y \
     --chain-id "$CHAIN_ID" \
     --node "$RPC_NODE"
   ```

Shell helper (same `VERIFIER_PUBKEY` and optional overrides): `scripts/init_migratable_from_code_id.sh` — pass `CODE_ID` and set `VERIFIER_PUBKEY` / `CHAIN_ID` / `RPC_NODE` as needed.

3. **Get Contract Address**

   ```bash
   TXHASH="<instantiate-transaction-hash>"
   RPC_NODE="${RPC_NODE:-https://rpc.xion-testnet-2.burnt.com:443}"

   CONTRACT=$(xiond query tx "$TXHASH" \
     --node "$RPC_NODE" \
     --output json | jq -r '.events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
   ```

### Query Contract

Set `RPC_NODE` to the testnet or mainnet RPC from the tables above (defaults below assume testnet).

```bash
CONTRACT="<your-contract-address>"
RPC_NODE="${RPC_NODE:-https://rpc.xion-testnet-2.burnt.com:443}"

# Query config
QUERY='{"config":{}}'
xiond query wasm contract-state smart "$CONTRACT" "$QUERY" \
  --output json \
  --node "$RPC_NODE"

# Query claim digest
QUERY='{"get_claim_digest":{"claim":{...}}}'
xiond query wasm contract-state smart "$CONTRACT" "$QUERY" \
  --output json \
  --node "$RPC_NODE"

# Check if claimed
QUERY='{"is_claimed":{"digest_hex":"0x..."}}'
xiond query wasm contract-state smart "$CONTRACT" "$QUERY" \
  --output json \
  --node "$RPC_NODE"
```

### Execute Transactions

```bash
CONTRACT="<your-contract-address>"
WALLET="your-wallet-id"
CHAIN_ID="${CHAIN_ID:-xion-testnet-2}"
RPC_NODE="${RPC_NODE:-https://rpc.xion-testnet-2.burnt.com:443}"

# Claim
EXECUTE='{"claim":{"claim":{...},"signature":"0x..."}}'
xiond tx wasm execute "$CONTRACT" "$EXECUTE" \
  --from "$WALLET" \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y \
  --node "$RPC_NODE" \
  --chain-id "$CHAIN_ID"

# Set Verifier (owner only)
VERIFIER_PUBKEY="0x..."
EXECUTE="{\"set_verifier\":{\"verifier_pubkey\":\"${VERIFIER_PUBKEY}\"}}"
xiond tx wasm execute "$CONTRACT" "$EXECUTE" \
  --from "$WALLET" \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y \
  --node "$RPC_NODE" \
  --chain-id "$CHAIN_ID"

# Transfer Ownership (owner only)
EXECUTE='{"transfer_ownership":{"new_owner":"xion1..."}}'
xiond tx wasm execute "$CONTRACT" "$EXECUTE" \
  --from "$WALLET" \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y \
  --node "$RPC_NODE" \
  --chain-id "$CHAIN_ID"

# Emergency Withdraw (owner only)
EXECUTE='{"emergency_withdraw":{"asset":{...},"to":"xion1...","value":"1000"}}'
xiond tx wasm execute "$CONTRACT" "$EXECUTE" \
  --from "$WALLET" \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y \
  --node "$RPC_NODE" \
  --chain-id "$CHAIN_ID"
```

### Migrate Contract

The contract supports migration to upgrade the contract code while preserving the contract state.

```bash
CONTRACT="<your-contract-address>"
WALLET="your-wallet-id"
NEW_CODE_ID="<new-code-id>"
MSG='{}'
CHAIN_ID="${CHAIN_ID:-xion-testnet-2}"
RPC_NODE="${RPC_NODE:-https://rpc.xion-testnet-2.burnt.com:443}"

xiond tx wasm migrate "$CONTRACT" "$NEW_CODE_ID" "$MSG" \
  --from "$WALLET" \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y \
  --chain-id "$CHAIN_ID" \
  --node "$RPC_NODE"
```

**Note**: Migration requires the contract admin/owner permission. The new code must have a `migrate` entry point implemented.

### Notes

- **Mainnet** code ID **66** and **testnet** code ID **2026** include migration support; instances must be created **with an admin** to use `xiond tx wasm migrate`.
- Set `VERIFIER_PUBKEY` (or `INIT_MSG`) to your production verifier on mainnet; the testnet default in this README is for examples only.
- The contract owner/admin can update the verifier pubkey (`set_verifier`), transfer ownership, perform emergency withdrawals, and execute migrations.

## Demo Applications

Demo applications are available in the `examples/` directory to help you get started with interacting with the TradeOS contract.

### Frontend Demo (demo-app)

A React/Next.js frontend application that demonstrates how to interact with the TradeOS contract from a web application.

**Location**: `examples/demo-app/`

**Features**:

- Wallet connection using Abstraxion authentication
- All contract query methods (Config, GetClaimDigest, IsClaimed)
- Send tokens to contract address

**Prerequisites**:

- Treasury contract configured with **Send Funds** permission (see [Treasury Contracts Documentation](https://docs.burnt.com/xion/developers/getting-started-advanced/gasless-ux-and-permission-grants/treasury-contracts))
- TradeOS contract address (generate using the instantiate command above)

**Setup**:

1. Navigate to `examples/demo-app/`
2. Install dependencies: `npm install` or `pnpm install`
3. Create `.env` file with `NEXT_PUBLIC_TREASURY_CONTRACT` and `NEXT_PUBLIC_TRADEOS_CONTRACT`
4. Run: `npm run dev` or `pnpm dev`

See `examples/demo-app/README.md` for detailed instructions.

### Backend Demo (node-scripts)

TypeScript scripts for interacting with the TradeOS contract from a backend environment using CosmJS.

**Location**: `examples/node-scripts/`

**Features**:

- Query scripts: Config, GetClaimDigest, IsClaimed
- Transaction scripts: Claim, SetVerifier, TransferOwnership

**Prerequisites**:

- Node.js and npm/pnpm installed
- Owner wallet mnemonic (for admin operations)

**Setup**:

1. Navigate to `examples/node-scripts/`
2. Install dependencies: `npm install` or `pnpm install`
3. Create `.env` file with `OWNER_MNEMONIC` and `CONTRACT_ADDRESS`
4. Run scripts: `npm run claim`, `npm run set-verifier`, etc.

See `examples/node-scripts/README.md` for detailed instructions.

## Contract Code Deployment Steps

**Mainnet:** WASM is already stored as **code ID `66`** ([governance proposal #56](https://www.mintscan.io/xion/proposals/56)). Instantiate from that code ID instead of uploading again.

**Testnet:** Code ID **2026** is already deployed for reference; use the steps below only if you need a new upload.

If you want to deploy the contract code again (e.g. to testnet or a devnet), you can follow the steps below.

1. **Compile and Optimize Contract**

   ```bash
   docker run --rm -v "$(pwd)":/code \
     --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
     --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
     cosmwasm/optimizer:0.16.1
   ```

2. **Upload Contract to XION Testnet**

   ```bash
   WALLET="your-wallet-name"
   xiond tx wasm store ./artifacts/tradeos_cw_sc.wasm \
     --chain-id xion-testnet-2 \
     --gas-adjustment 1.3 \
     --gas-prices 0.001uxion \
     --gas auto \
     -y --output json \
     --node https://rpc.xion-testnet-2.burnt.com:443 \
     --from $WALLET
   ```

3. **Get Code ID**

   ```bash
   TXHASH="<transaction-hash>"
   CODE_ID=$(xiond query tx $TXHASH \
     --node https://rpc.xion-testnet-2.burnt.com:443 \
     --output json | jq -r '.events[-1].attributes[1].value')
   ```
