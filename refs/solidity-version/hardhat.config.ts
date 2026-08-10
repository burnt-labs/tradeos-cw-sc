import hardhatEthers from "@nomicfoundation/hardhat-ethers";
import hardhatEthersChaiMatchers from "@nomicfoundation/hardhat-ethers-chai-matchers";
import hardhatMocha from "@nomicfoundation/hardhat-mocha";
import hardhatNetworkHelpers from "@nomicfoundation/hardhat-network-helpers";
import hardhatTypechain from "@nomicfoundation/hardhat-typechain";
import { defineConfig } from "hardhat/config";
import * as dotenv from "dotenv";

dotenv.config();

const accounts = process.env.MNEMONICS ? { mnemonic: process.env.MNEMONICS } : [];

export default defineConfig({
    plugins: [hardhatEthers, hardhatEthersChaiMatchers, hardhatMocha, hardhatNetworkHelpers, hardhatTypechain],
    networks: {
        hardhat: {
            type: "edr-simulated",
            chainType: "l1",
        },
        sepolia: {
            type: "http",
            chainType: "l1",
            chainId: 11155111,
            url: "https://rpc.ankr.com/eth_sepolia",
            accounts,
        },
        bscTestnet: {
            type: "http",
            chainType: "l1",
            chainId: 97,
            url: "https://rpc.ankr.com/bsc_testnet_chapel",
            accounts,
        },
        baseSepolia: {
            type: "http",
            chainType: "l1",
            chainId: 84532,
            url: "https://rpc.ankr.com/base_sepolia",
            accounts,
        },
        hashkeyTestnet: {
            type: "http",
            chainType: "l1",
            chainId: 133,
            url: "https://hashkeychain-testnet.alt.technology",
            accounts,
        },
        ethereum: {
            type: "http",
            chainType: "l1",
            chainId: 1,
            url: "https://rpc.ankr.com/eth",
            accounts,
        },
        bsc: {
            type: "http",
            chainType: "l1",
            chainId: 56,
            url: "https://1rpc.io/bnb",
            accounts,
        },
        base: {
            type: "http",
            chainType: "l1",
            chainId: 8453,
            url: "https://rpc.ankr.com/base",
            accounts,
        },
        hashkey: {
            type: "http",
            chainType: "l1",
            chainId: 177,
            url: "https://mainnet.hsk.xyz",
            accounts,
        },
    },
    solidity: {
        version: "0.8.28",
        settings: {
            evmVersion: "cancun",
            optimizer: {
                enabled: true,
                runs: 200,
            },
            metadata: {
                bytecodeHash: "none",
            },
        },
    },
    paths: {
        sources: "./contracts",
        tests: {
            mocha: "./tests",
        },
        cache: "./cache",
        artifacts: "./artifacts",
    },
    typechain: {
        outDir: "types",
        tsNocheck: true,
    },
});
