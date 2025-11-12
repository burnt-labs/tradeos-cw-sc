import * as dotenv from 'dotenv';

dotenv.config();

interface Config {
  XION_RPC_URL: string;
  CHAIN_ID: string;
  OWNER_MNEMONIC: string | undefined;
  CONTRACT_ADDRESS: string;
  validateConfig: () => boolean;
}

const config: Config = {
  // Network configuration
  XION_RPC_URL: process.env.XION_RPC_URL || "https://rpc.xion-testnet-2.burnt.com:443",
  CHAIN_ID: process.env.XION_CHAIN_ID || "xion-testnet-2",

  // Wallet configuration
  OWNER_MNEMONIC: process.env.OWNER_MNEMONIC,

  // Contract address
  CONTRACT_ADDRESS: process.env.CONTRACT_ADDRESS || "",

  // Validate configuration
  validateConfig: function (): boolean {
    if (!this.OWNER_MNEMONIC) {
      throw new Error("OWNER_MNEMONIC is required in .env file");
    }
    if (!this.XION_RPC_URL) {
      throw new Error("XION_RPC_URL is required in .env file");
    }
    if (!this.CONTRACT_ADDRESS) {
      throw new Error("CONTRACT_ADDRESS is required in .env file");
    }
    return true;
  },
};

export default config;

