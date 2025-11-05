# TradeOS Demo App

This is a frontend demo application for interacting with the TradeOS CosmWasm Smart Contract on XION Testnet.

## Features

- **Wallet Connection**: Connect using Abstraxion authentication
- **Contract Queries**: Query all contract methods:
  - `Config`: Get contract owner and verifier public key
  - `GetClaimDigest`: Compute digest for a claim
  - `IsClaimed`: Check if a digest has been claimed
- **Send Tokens**: Send XION tokens to the TradeOS contract address

## Prerequisites

1. **Treasury Contract Setup**: Before using this demo, you must configure the Treasury contract with **Send Funds** permission. See the [Treasury Contracts Documentation](https://docs.burnt.com/xion/developers/getting-started-advanced/gasless-ux-and-permission-grants/treasury-contracts) for details.

2. **TradeOS Contract**: You need a deployed TradeOS contract address. Generate it using the instantiate command in the main README.md (lines 17-43).

## Setup

1. Install dependencies:

```bash
pnpm install
# or
npm install
```

2. Create a `.env` file (or copy from `.env.example`):

```bash
# Treasury Contract Address
NEXT_PUBLIC_TREASURY_CONTRACT=xion13uwmwzdes7urtjyv7mye8ty6uk0vsgdrh2a2k94tp0yxx9vv3e9qazapyu

# TradeOS Contract Address
# Generate using the instantiate command in README.md (lines 17-43)
NEXT_PUBLIC_TRADEOS_CONTRACT=xion1tg3x833n3qmccpzde32mw093vsr6eltgrxzvy5rjk2ehmf8u6hdqzyh58l
```

3. Run the development server:

```bash
pnpm dev
# or
npm run dev
```

4. Open [http://localhost:3002](http://localhost:3002) in your browser.

## Environment Variables

- `NEXT_PUBLIC_TREASURY_CONTRACT`: The Treasury contract address configured with Send Funds permission
- `NEXT_PUBLIC_TRADEOS_CONTRACT`: The deployed TradeOS contract address

## Important Notes

⚠️ **Treasury Permission Required**: The "Send Tokens" feature requires the Treasury contract to have the **Send Funds** permission configured. Without this permission, transactions will fail.

For more information on configuring Treasury permissions, see:
https://docs.burnt.com/xion/developers/getting-started-advanced/gasless-ux-and-permission-grants/treasury-contracts

