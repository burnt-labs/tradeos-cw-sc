import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice, StargateClient } from "@cosmjs/stargate";
import config from "./config";

/**
 * Creates a read-only client for querying the blockchain
 */
export async function getQueryClient(): Promise<StargateClient> {
  return await StargateClient.connect(config.XION_RPC_URL);
}

/**
 * Creates a signing client that can perform transactions
 */
export async function getSigningClient(
  mnemonic?: string
): Promise<SigningCosmWasmClient> {
  const mnemonicToUse = mnemonic || config.OWNER_MNEMONIC;
  if (!mnemonicToUse) {
    throw new Error("Mnemonic is required");
  }

  // Create wallet from mnemonic
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonicToUse, {
    prefix: "xion",
  });

  // Create and return a signing client
  return await SigningCosmWasmClient.connectWithSigner(
    config.XION_RPC_URL,
    wallet,
    { gasPrice: GasPrice.fromString("0.025uxion") }
  );
}

/**
 * Gets the address associated with a mnemonic
 */
export async function getAddressFromMnemonic(
  mnemonic: string
): Promise<string> {
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix: "xion",
  });
  const [firstAccount] = await wallet.getAccounts();
  return firstAccount.address;
}

