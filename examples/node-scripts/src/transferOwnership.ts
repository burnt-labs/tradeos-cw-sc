import { getSigningClient, getAddressFromMnemonic } from "./xionConnect";
import config from "./config";

/**
 * Example: Transfer Ownership (Owner Only)
 * 
 * This transaction can only be executed by the contract owner.
 * It transfers ownership of the contract to a new address.
 */
async function transferOwnership() {
  try {
    // Validate configuration
    config.validateConfig();

    console.log("🔐 Connecting to Xion testnet to transfer ownership...");

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

    // New owner address
    // Example: Replace with the actual new owner address
    // WARNING: This is a critical operation. Make sure you have the correct address.
    const newOwnerAddress = "xion1..."; // Replace with actual new owner address

    if (newOwnerAddress === "xion1...") {
      console.error("❌ Please set a valid new owner address in the script.");
      console.error("   Example: const newOwnerAddress = 'xion1abc123...';");
      return;
    }

    if (newOwnerAddress === ownerAddress) {
      console.error("❌ New owner address cannot be the same as current owner.");
      return;
    }

    console.log("\n📝 Transferring ownership...");
    console.log(`👤 Current owner: ${ownerAddress}`);
    console.log(`👤 New owner: ${newOwnerAddress}`);

    // Execute transfer_ownership
    const executeMsg = {
      transfer_ownership: {
        new_owner: newOwnerAddress,
      },
    };

    const fee = "auto";
    const result = await client.execute(
      ownerAddress,
      config.CONTRACT_ADDRESS,
      executeMsg,
      fee
    );

    console.log("\n✅ Transfer ownership transaction successful!");
    console.log(`📋 Transaction hash: ${result.transactionHash}`);
    console.log(`⛽ Gas used: ${result.gasUsed}`);
    console.log(`📊 Height: ${result.height}`);

    // Verify ownership was transferred
    console.log("\n🔍 Verifying ownership transfer...");
    const updatedConfigResponse = await client.queryContractSmart(config.CONTRACT_ADDRESS, {
      config: {},
    });
    console.log(`📋 Updated owner: ${updatedConfigResponse.owner}`);
    if (updatedConfigResponse.owner === newOwnerAddress) {
      console.log(`✅ Ownership successfully transferred to ${newOwnerAddress}`);
    } else {
      console.log(`⚠️  Warning: Ownership may not have been transferred correctly.`);
    }

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
    } else if (error.message.includes("invalid address")) {
      console.error("✍️  Invalid new owner address. Make sure it's a valid XION bech32 address.");
    }
  }
}

// Execute the function
transferOwnership();

