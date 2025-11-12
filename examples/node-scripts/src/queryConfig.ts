import { getQueryClient } from "./xionConnect";
import config from "./config";

/**
 * Example: Query Contract Config
 * 
 * Queries the contract configuration including owner and verifier public key.
 */
async function queryConfig() {
  try {
    // Validate configuration
    if (!config.CONTRACT_ADDRESS) {
      throw new Error("CONTRACT_ADDRESS is required in .env file");
    }

    console.log("🔍 Querying TradeOS contract config...");
    console.log(`📜 Contract address: ${config.CONTRACT_ADDRESS}`);

    // Create query client
    const client = await getQueryClient();

    console.log(`🔗 Connected to ${config.XION_RPC_URL}`);

    // Query config
    const result = await client.queryContractSmart(config.CONTRACT_ADDRESS, {
      config: {},
    });

    console.log("\n✅ Contract Config:");
    console.log(`👤 Owner: ${result.owner}`);
    console.log(`🔑 Verifier Pubkey: ${result.verifier_pubkey_hex}`);

    // Pretty print JSON
    console.log("\n📋 Full Response:");
    console.log(JSON.stringify(result, null, 2));

  } catch (error: any) {
    console.error("❌ Error:", error.message);
    if (error.stack) {
      console.error("📍 Stack trace:", error.stack);
    }

    if (error.message.includes("contract")) {
      console.error("📜 Contract error. Check if the contract address is correct.");
    }
  }
}

// Execute the function
queryConfig();

