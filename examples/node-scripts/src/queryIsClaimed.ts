import { getQueryClient } from "./xionConnect";
import config from "./config";

/**
 * Example: Query Is Claimed
 * 
 * Checks if a claim digest has already been claimed.
 */
async function queryIsClaimed() {
  try {
    // Validate configuration
    if (!config.CONTRACT_ADDRESS) {
      throw new Error("CONTRACT_ADDRESS is required in .env file");
    }

    console.log("🔍 Querying TradeOS contract to check if claim is claimed...");
    console.log(`📜 Contract address: ${config.CONTRACT_ADDRESS}`);

    // Create query client
    const client = await getQueryClient();

    console.log(`🔗 Connected to ${config.XION_RPC_URL}`);

    // Example digest hex (from queryClaimDigest)
    // Replace with actual digest hex
    const digestHex = "0x" + "0".repeat(64); // Placeholder

    if (digestHex === "0x" + "0".repeat(64)) {
      console.error("❌ Please set a valid digest hex in the script.");
      console.error("   Example: const digestHex = '0xabc123...';");
      console.error("   You can get the digest by running: npm run query-claim-digest");
      return;
    }

    console.log(`\n📋 Digest Hex: ${digestHex}`);

    // Query is_claimed
    const result = await client.queryContractSmart(config.CONTRACT_ADDRESS, {
      is_claimed: {
        digest_hex: digestHex,
      },
    });

    console.log("\n✅ Claim Status:");
    console.log(`📋 Claimed: ${result.claimed ? "✅ YES" : "❌ NO"}`);

    // Pretty print JSON
    console.log("\n📋 Full Response:");
    console.log(JSON.stringify(result, null, 2));

    if (result.claimed) {
      console.log("\n⚠️  This claim has already been claimed and cannot be claimed again.");
    } else {
      console.log("\n✅ This claim is available and can be claimed.");
    }

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
queryIsClaimed();

