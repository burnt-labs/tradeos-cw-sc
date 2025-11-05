# TradeOS Cosmwasm Smart Contract

This is the Cosmwasm Smart Contract for the TradeOS project.

## Deployment Information

### XION Testnet

- **Network**: `xion-testnet-2`
- **RPC Endpoint**: `https://rpc.xion-testnet-2.burnt.com:443`
- **Chain ID**: `xion-testnet-2`

### Contract Details

- **Deployed Code ID**: `1792`

### Deployment Steps(Please skip this as the code id is already deployed and can be used for instantiation)

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

### Instantiate Contract

1. **Instantiate Contract**

   ```bash
   CODE_ID="1792"
   MSG='{"owner":null,"verifier_pubkey":"0x93865a37af13405ba2f78197d1a0daabd827d4f8907ae754a9c2c9bb9eedd6703b"}'
   xiond tx wasm instantiate $CODE_ID "$MSG" \
     --from $WALLET \
     --label "tradeos-cw-sc" \
     --gas-prices 0.025uxion \
     --gas auto \
     --gas-adjustment 1.3 \
     -y --no-admin \
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
CONTRACT="xion1tg3x833n3qmccpzde32mw093vsr6eltgrxzvy5rjk2ehmf8u6hdqzyh58l"

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
CONTRACT="xion1tg3x833n3qmccpzde32mw093vsr6eltgrxzvy5rjk2ehmf8u6hdqzyh58l"
WALLET="your-wallet-name"

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

### Notes

- The current `verifier_pubkey` is a test key. You can update it using the `SetVerifier` execute message if needed.
- The contract owner can update the verifier pubkey, transfer ownership, and perform emergency withdrawals.
