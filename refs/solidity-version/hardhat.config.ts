import "@nomicfoundation/hardhat-toolbox";
import "hardhat-deploy";
import type { HardhatUserConfig } from "hardhat/config";
import * as dotenv from "dotenv";

dotenv.config();

const config: HardhatUserConfig = {
    defaultNetwork: "hardhat",
    networks: {
        hardhat: {},
        sepolia: {
            chainId: 11155111,
            url: "https://rpc.ankr.com/eth_sepolia",
            accounts: {
                mnemonic: process.env.MNEMONICS,
            },
        },
        bscTestnet: {
            chainId: 97,
            url: "https://rpc.ankr.com/bsc_testnet_chapel",
            accounts: {
                mnemonic: process.env.MNEMONICS,
            },
        },
        baseSepolia: {
            chainId: 84532,
            url: "https://rpc.ankr.com/base_sepolia",
            accounts: {
                mnemonic: process.env.MNEMONICS,
            },
        },
        hashkeyTestnet: {
            chainId: 133,
            url: "https://hashkeychain-testnet.alt.technology",
            accounts: {
                mnemonic: process.env.MNEMONICS,
            },
        },
        ethereum: {
            chainId: 1,
            url: "https://rpc.ankr.com/eth",
            accounts: {
                mnemonic: process.env.MNEMONICS,
            },
        },
        bsc: {
            chainId: 56,
            url: "https://1rpc.io/bnb",
            accounts: {
                mnemonic: process.env.MNEMONICS,
            },
        },
        base: {
            chainId: 8453,
            url: "https://rpc.ankr.com/base",
            accounts: {
                mnemonic: process.env.MNEMONICS,
            },
        },
        hashkey: {
            chainId: 177,
            url: "https://mainnet.hsk.xyz",
            accounts: {
                mnemonic: process.env.MNEMONICS,
            },
        },
    },
    etherscan: {
        apiKey: {
            sepolia: process.env.ETHERSCAN_API_KEY,
            bscTestnet: process.env.BSCSCAN_API_KEY,
            baseSepolia: process.env.BASESCAN_API_KEY,
            ethereum: process.env.ETHERSCAN_API_KEY,
            bsc: process.env.BSCSCAN_API_KEY,
            base: process.env.BASESCAN_API_KEY,
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
    namedAccounts: {
        deployer: {
            default: 0,
        },
        mockUser: {
            default: 1,
        },
    },
    gasReporter: {
        enabled: true,
        src: "./contracts",
        excludeContracts: ["Forwarder"],
        currency: "USD",
    },
    paths: {
        sources: "./contracts",
        tests: "./tests",
        cache: "./cache",
        artifacts: "./artifacts",
    },
    remappings: ["@openzeppelin/=node_modules/@openzeppelin/"],
    typechain: {
        outDir: "types",
        target: "ethers-v6",
    },
};

export default config;
