import { getSigningClient, getAddressFromMnemonic } from "./xionConnect";
import config from "./config";

/**
 * Example: Set Verifier (Owner Only)
 * 
 * This transaction can only be executed by the contract owner.
 * It updates the verifier public key used for claim verification.
 */
async function setVerifier() {
  try {
    // Validate configuration
    config.validateConfig();

    console.log("🔐 Connecting to Xion testnet to set verifier...");

    // Get owner address
    const ownerAddress = await getAddressFromMnemonic(config.OWNER_MNEMONIC!);
    console.log(`📍 Owner wallet address: ${ownerAddress}`);

    // Create signing client
    const client = await getSigningClient();

    console.log(`🔗 Connected to ${config.XION_RPC_URL}`);
    console.log(`📜 Contract address: ${config.CONTRACT_ADDRESS}`);

    // Query current config first
    console.log("\n🔍 Querying current contract config...");
    const configResponse = await client.queryContractSmart(config.CONTRACT_ADDRESS, {
      config: {},
    });
    console.log(`📋 Current owner: ${configResponse.owner}`);
    console.log(`📋 Current verifier pubkey: ${configResponse.verifier_pubkey_hex}`);

    // Verify owner
    if (configResponse.owner !== ownerAddress) {
      console.error(`❌ Address ${ownerAddress} is not the contract owner.`);
      console.error(`   Current owner is: ${configResponse.owner}`);
      return;
    }
    console.log("✅ Address is the contract owner");

    // New verifier public key (33-byte compressed secp256k1 pubkey)
    // Example: Replace with your actual verifier public key
    const newVerifierPubkey = "0x93865a37af13405ba2f78197d1a0daabd827d4f8907ae754a9c2c9bb9eedd6703b";

    console.log("\n📝 Setting new verifier public key...");
    console.log(`🔑 New verifier pubkey: ${newVerifierPubkey}`);

    // Execute set_verifier
    const executeMsg = {
      set_verifier: {
        verifier_pubkey: newVerifierPubkey,
      },
    };

    const fee = "auto";
    const result = await client.execute(
      ownerAddress,
      config.CONTRACT_ADDRESS,
      executeMsg,
      fee
    );

    console.log("\n✅ Set verifier transaction successful!");
    console.log(`📋 Transaction hash: ${result.transactionHash}`);
    console.log(`⛽ Gas used: ${result.gasUsed}`);
    console.log(`📊 Height: ${result.height}`);

    // Verify verifier was updated
    console.log("\n🔍 Verifying verifier update...");
    const updatedConfigResponse = await client.queryContractSmart(config.CONTRACT_ADDRESS, {
      config: {},
    });
    console.log(`📋 Updated verifier pubkey: ${updatedConfigResponse.verifier_pubkey_hex}`);
    console.log(`✅ Verifier updated successfully`);

  } catch (error: any) {
    console.error("❌ Error:", error.message);
    if (error.stack) {
      console.error("📍 Stack trace:", error.stack);
    }

    // Handle specific error cases
    if (error.message.includes("unauthorized")) {
      console.error("🚫 Unauthorized. This operation can only be performed by the contract owner.");
    } else if (error.message.includes("insufficient funds")) {
      console.error("💰 Insufficient funds. Make sure your wallet has enough tokens for gas.");
    } else if (error.message.includes("contract")) {
      console.error("📜 Contract error. Check if the contract address is correct.");
    } else if (error.message.includes("OWNER_MNEMONIC")) {
      console.error("🔑 Please set OWNER_MNEMONIC in your .env file");
    } else if (error.message.includes("invalid verifier_pubkey")) {
      console.error("✍️  Invalid verifier public key. Must be a 33-byte compressed secp256k1 pubkey.");
    }
  }
}

// Execute the function
setVerifier();

