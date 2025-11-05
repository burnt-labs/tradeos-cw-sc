import { getQueryClient } from "./xionConnect";
import config from "./config";

interface ClaimInfo {
  asset: {
    native?: { denom: string };
    cw20?: { contract: string };
  };
  to: string;
  value: string;
  deadline: number;
  comment: string;
}

/**
 * Example: Query Claim Digest
 * 
 * Computes the digest for a claim that the verifier signs.
 * This digest is used for signature verification.
 */
async function queryClaimDigest() {
  try {
    // Validate configuration
    if (!config.CONTRACT_ADDRESS) {
      throw new Error("CONTRACT_ADDRESS is required in .env file");
    }

    console.log("🔍 Querying TradeOS contract for claim digest...");
    console.log(`📜 Contract address: ${config.CONTRACT_ADDRESS}`);

    // Create query client
    const client = await getQueryClient();

    console.log(`🔗 Connected to ${config.XION_RPC_URL}`);

    // Example claim info
    const claimInfo: ClaimInfo = {
      asset: {
        native: { denom: "uxion" },
      },
      to: "xion1...", // Replace with recipient address
      value: "1000000", // 1 XION (in uxion)
      deadline: 0, // 0 = no deadline
      comment: "Example claim",
    };

    console.log("\n📝 Claim Info:");
    console.log(`- Asset: ${JSON.stringify(claimInfo.asset)}`);
    console.log(`- To: ${claimInfo.to}`);
    console.log(`- Value: ${claimInfo.value} uxion`);
    console.log(`- Deadline: ${claimInfo.deadline === 0 ? "No deadline" : new Date(claimInfo.deadline * 1000).toISOString()}`);
    console.log(`- Comment: ${claimInfo.comment}`);

    // Query claim digest
    const result = await client.queryContractSmart(config.CONTRACT_ADDRESS, {
      get_claim_digest: {
        claim: claimInfo,
      },
    });

    console.log("\n✅ Claim Digest:");
    console.log(`📋 Digest (hex): ${result.digest_hex}`);

    // Pretty print JSON
    console.log("\n📋 Full Response:");
    console.log(JSON.stringify(result, null, 2));

    console.log("\n💡 Note: This digest should be signed by the verifier using their private key.");
    console.log("   The signature can then be used in the claim transaction.");

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
queryClaimDigest();

