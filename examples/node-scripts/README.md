# TradeOS Node Scripts

This directory contains TypeScript scripts for interacting with the TradeOS smart contract on XION Testnet from a backend environment.

## Prerequisites

1. Node.js and npm/pnpm installed
2. A deployed TradeOS contract address
3. Owner wallet mnemonic (for admin operations)

## Setup

1. Install dependencies:

```bash
npm install
# or
pnpm install
```

2. Create a `.env` file (copy from `.env.example`):

```bash
# Network configuration
XION_RPC_URL=https://rpc.xion-testnet-2.burnt.com:443
XION_CHAIN_ID=xion-testnet-2

# Wallet configuration (owner mnemonic for admin operations)
OWNER_MNEMONIC=your owner wallet mnemonic phrase here

# Contract address
CONTRACT_ADDRESS=xion1tg3x833n3qmccpzde32mw093vsr6eltgrxzvy5rjk2ehmf8u6hdqzyh58l
```

## Available Scripts

### Query Scripts

#### Query Config

Query the contract configuration (owner and verifier public key):

```bash
npm run query-config
```

#### Query Claim Digest

Compute the digest for a claim:

```bash
npm run query-claim-digest
```

#### Query Is Claimed

Check if a claim digest has been claimed:

```bash
npm run query-is-claimed
```

### Transaction Scripts

#### Claim

Execute a claim transaction (requires valid signature from verifier):

```bash
npm run claim
```

**Note**: This requires a valid signature from the verifier. The signature should be generated off-chain using the verifier's private key and the claim digest. You can get the claim digest by running `npm run query-claim-digest`.

#### Set Verifier (Owner Only)

Update the verifier public key (owner only):

```bash
npm run set-verifier
```

**Note**: This operation can only be performed by the contract owner.

#### Transfer Ownership (Owner Only)

Transfer contract ownership to a new address (owner only):

```bash
npm run transfer-ownership
```

**Note**: This operation can only be performed by the contract owner. Make sure to update the `newOwnerAddress` variable in the script before running.

## Transaction Examples

The scripts in this directory demonstrate three main transaction types:

1. **Claim**: Claim tokens using a claim info and signature from the verifier
2. **Set Verifier**: Update the verifier public key (owner only)
3. **Transfer Ownership**: Transfer contract ownership to a new address (owner only)

## Important Notes

- **Owner Operations**: `set-verifier` and `transfer-ownership` can only be executed by the contract owner.
- **Claim Signature**: The `claim` transaction requires a valid signature from the verifier. The signature must be generated off-chain using the verifier's private key and the claim digest.
- **Gas Fees**: Make sure your wallet has enough XION tokens to cover gas fees.
- **Environment Variables**: Always set the required environment variables in your `.env` file before running the scripts.

## Example Workflow

1. Query the contract config to verify ownership:
   ```bash
   npm run query-config
   ```

2. Query the claim digest for a claim:
   ```bash
   npm run query-claim-digest
   ```

3. Get a signature from the verifier for the digest

4. Execute the claim transaction:
   ```bash
   npm run claim
   ```

5. Verify the claim was processed:
   ```bash
   npm run query-is-claimed
   ```

## Error Handling

The scripts include comprehensive error handling for common issues:

- Insufficient funds
- Unauthorized operations (non-owner)
- Invalid contract addresses
- Invalid signatures
- Already claimed digests

## References

- [XION TypeScript Guide](https://docs.burnt.com/xion/developers/getting-started-advanced/your-first-dapp/xion-typescript)
- [CosmJS Documentation](https://cosmos.github.io/cosmjs/)

