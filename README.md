# TradeOS Cosmwasm Smart Contract

This is the Cosmwasm Smart Contract for the TradeOS project.

## Deployment Information

### XION Testnet

- **Network**: `xion-testnet-2`
- **RPC Endpoint**: `https://rpc.xion-testnet-2.burnt.com:443`
- **Chain ID**: `xion-testnet-2`

### Contract Details

#### Testnet

- **Deployed Code ID**: `2026`
- **Contract Address**: `xion1lf0j559uuj9uh6dx8apwnf9mvcxpjjv66dxm5k7gny8430e96d2qdz5ueq`
- **Owner / Admin**: `xion1epzznazp28up4asses7jdcyqnw3n8lu7f5g9xs` (Deployed by Bohao's wallet)
- **Verifier Pubkey**: `0x03a6a96da6e704f74f53b3a98e0ae37123abf5a96803d8d971795637b0034a60cf`

### Instantiate Contract

1. **Instantiate Contract**

   ```bash
   CODE_ID="2026"
   WALLET="your-wallet-id"
   ADMIN_ADDR=$(xiond keys show "$WALLET" -a)
   MSG='{"owner":null,"verifier_pubkey":"0x03a6a96da6e704f74f53b3a98e0ae37123abf5a96803d8d971795637b0034a60cf"}'

   xiond tx wasm instantiate "$CODE_ID" "$MSG" \
     --from "$WALLET" \
     --admin "$ADMIN_ADDR" \
     --label "tradeos-cw-sc" \
     --gas-prices 0.025uxion \
     --gas auto \
     --gas-adjustment 1.3 \
     -y \
     --chain-id xion-testnet-2 \
     --node https://rpc.xion-testnet-2.burnt.com:443
   ```

2. **Get Contract Address**

   ```bash
   TXHASH="<instantiate-transaction-hash>"
   CONTRACT=$(xiond query tx $TXHASH \
     --node https://rpc.xion-testnet-2.burnt.com:443 \
     --output json | jq -r '.events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
   ```

### Query Contract

```bash
CONTRACT="<your-contract-address>"

# Query config
QUERY='{"config":{}}'
xiond query wasm contract-state smart $CONTRACT "$QUERY" \
  --output json \
  --node https://rpc.xion-testnet-2.burnt.com:443

# Query claim digest
QUERY='{"get_claim_digest":{"claim":{...}}}'
xiond query wasm contract-state smart $CONTRACT "$QUERY" \
  --output json \
  --node https://rpc.xion-testnet-2.burnt.com:443

# Check if claimed
QUERY='{"is_claimed":{"digest_hex":"0x..."}}'
xiond query wasm contract-state smart $CONTRACT "$QUERY" \
  --output json \
  --node https://rpc.xion-testnet-2.burnt.com:443
```

### Execute Transactions

```bash
CONTRACT="<your-contract-address>"
WALLET="your-wallet-id"

# Claim
EXECUTE='{"claim":{"claim":{...},"signature":"0x..."}}'
xiond tx wasm execute $CONTRACT "$EXECUTE" \
  --from $WALLET \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y \
  --node https://rpc.xion-testnet-2.burnt.com:443 \
  --chain-id xion-testnet-2

# Set Verifier (owner only)
EXECUTE='{"set_verifier":{"verifier_pubkey":"0x..."}}'
xiond tx wasm execute $CONTRACT "$EXECUTE" \
  --from $WALLET \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y \
  --node https://rpc.xion-testnet-2.burnt.com:443 \
  --chain-id xion-testnet-2

# Transfer Ownership (owner only)
EXECUTE='{"transfer_ownership":{"new_owner":"xion1..."}}'
xiond tx wasm execute $CONTRACT "$EXECUTE" \
  --from $WALLET \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y \
  --node https://rpc.xion-testnet-2.burnt.com:443 \
  --chain-id xion-testnet-2

# Emergency Withdraw (owner only)
EXECUTE='{"emergency_withdraw":{"asset":{...},"to":"xion1...","value":"1000"}}'
xiond tx wasm execute $CONTRACT "$EXECUTE" \
  --from $WALLET \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y \
  --node https://rpc.xion-testnet-2.burnt.com:443 \
  --chain-id xion-testnet-2
```

### Migrate Contract

The contract supports migration to upgrade the contract code while preserving the contract state.

```bash
CONTRACT="<your-contract-address>"
WALLET="your-wallet-id"
NEW_CODE_ID="<new-code-id>"
MSG='{}'

xiond tx wasm migrate $CONTRACT $NEW_CODE_ID "$MSG" \
  --from $WALLET \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y \
  --chain-id xion-testnet-2 \
  --node https://rpc.xion-testnet-2.burnt.com:443
```

**Note**: Migration requires the contract admin/owner permission. The new code must have a `migrate` entry point implemented.

### Notes

- The current `verifier_pubkey` is a test key. You can update it using the `SetVerifier` execute message if needed.
- The contract owner/admin can update the verifier pubkey, transfer ownership, perform emergency withdrawals, and execute migrations.
- Code ID 2026 includes migration support, and contracts instantiated **with an admin** can be upgraded to future code IDs via `xiond tx wasm migrate`.

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

You can skip this section as the code id is already deployed and can be used for instantiation.
If you want to deploy the contract code again, you can follow the steps below.

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
