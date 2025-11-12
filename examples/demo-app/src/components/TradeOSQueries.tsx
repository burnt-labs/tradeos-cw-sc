"use client";
import { useState } from "react";
import { useAbstraxionAccount, useAbstraxionClient } from "@burnt-labs/abstraxion";
import { Button, Input } from "@burnt-labs/ui";

interface TradeOSQueriesProps {
  contractAddress: string;
}

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

export function TradeOSQueries({ contractAddress }: TradeOSQueriesProps) {
  const { data: account } = useAbstraxionAccount();
  const { client: queryClient } = useAbstraxionClient();
  const [loading, setLoading] = useState<string | null>(null);
  const [queryResult, setQueryResult] = useState<any>(null);

  // Query Config
  const queryConfig = async () => {
    if (!contractAddress || !queryClient) return;
    setLoading("config");
    setQueryResult(null);
    try {
      const result = await queryClient.queryContractSmart(contractAddress, {
        config: {},
      });
      setQueryResult({ type: "Config", data: result });
    } catch (error) {
      setQueryResult({
        type: "Config",
        error: error instanceof Error ? error.message : "Unknown error",
      });
    } finally {
      setLoading(null);
    }
  };

  // Query GetClaimDigest
  const queryGetClaimDigest = async () => {
    if (!contractAddress || !account.bech32Address || !queryClient) return;
    setLoading("getClaimDigest");
    setQueryResult(null);

    const claimInfo: ClaimInfo = {
      asset: {
        native: { denom: "uxion" },
      },
      to: account.bech32Address,
      value: "1000000",
      deadline: 0,
      comment: "Test claim",
    };

    try {
      const result = await queryClient.queryContractSmart(contractAddress, {
        get_claim_digest: {
          claim: claimInfo,
        },
      });
      setQueryResult({ type: "GetClaimDigest", data: result });
    } catch (error) {
      setQueryResult({
        type: "GetClaimDigest",
        error: error instanceof Error ? error.message : "Unknown error",
      });
    } finally {
      setLoading(null);
    }
  };

  // Query IsClaimed
  const [digestHex, setDigestHex] = useState("");
  const queryIsClaimed = async () => {
    if (!contractAddress || !digestHex || !queryClient) return;
    setLoading("isClaimed");
    setQueryResult(null);
    try {
      const result = await queryClient.queryContractSmart(contractAddress, {
        is_claimed: {
          digest_hex: digestHex,
        },
      });
      setQueryResult({ type: "IsClaimed", data: result });
    } catch (error) {
      setQueryResult({
        type: "IsClaimed",
        error: error instanceof Error ? error.message : "Unknown error",
      });
    } finally {
      setLoading(null);
    }
  };

  return (
    <div className="rounded-lg border border-white/10 bg-gray-900/50 p-6">
      <h2 className="mb-4 text-xl font-semibold text-white">Contract Queries</h2>
      <p className="mb-4 text-sm text-gray-400">
        Query the TradeOS contract state. All queries are read-only and do not require
        gas.
      </p>

      <div className="space-y-4">
        {/* Query Config */}
        <div className="rounded border border-gray-700 bg-gray-800/50 p-4">
          <h3 className="mb-2 text-sm font-semibold text-white">Query Config</h3>
          <p className="mb-3 text-xs text-gray-400">
            Get the contract owner and verifier public key
          </p>
          <Button
            onClick={queryConfig}
            disabled={loading === "config" || !contractAddress || !queryClient}
            structure="outlined"
            fullWidth
          >
            {loading === "config" ? "Loading..." : "Query Config"}
          </Button>
          {queryResult?.type === "Config" && (
            <div className="mt-3 rounded bg-gray-900 p-3">
              {queryResult.error ? (
                <p className="text-xs text-red-400">{queryResult.error}</p>
              ) : (
                <pre className="text-xs text-gray-300 overflow-auto">
                  {JSON.stringify(queryResult.data, null, 2)}
                </pre>
              )}
            </div>
          )}
        </div>

        {/* Query GetClaimDigest */}
        <div className="rounded border border-gray-700 bg-gray-800/50 p-4">
          <h3 className="mb-2 text-sm font-semibold text-white">Query GetClaimDigest</h3>
          <p className="mb-3 text-xs text-gray-400">
            Compute the digest for a claim (using example values)
          </p>
          <Button
            onClick={queryGetClaimDigest}
            disabled={loading === "getClaimDigest" || !contractAddress || !account.bech32Address || !queryClient}
            structure="outlined"
            fullWidth
          >
            {loading === "getClaimDigest" ? "Loading..." : "Query GetClaimDigest"}
          </Button>
          {queryResult?.type === "GetClaimDigest" && (
            <div className="mt-3 rounded bg-gray-900 p-3">
              {queryResult.error ? (
                <p className="text-xs text-red-400">{queryResult.error}</p>
              ) : (
                <pre className="text-xs text-gray-300 overflow-auto">
                  {JSON.stringify(queryResult.data, null, 2)}
                </pre>
              )}
            </div>
          )}
        </div>

        {/* Query IsClaimed */}
        <div className="rounded border border-gray-700 bg-gray-800/50 p-4">
          <h3 className="mb-2 text-sm font-semibold text-white">Query IsClaimed</h3>
          <p className="mb-3 text-xs text-gray-400">
            Check if a digest has been claimed
          </p>
          <Input
            className="mb-3"
            placeholder="Enter digest hex (e.g., 0x...)"
            value={digestHex}
            onChange={(e) => setDigestHex(e.target.value)}
          />
          <Button
            onClick={queryIsClaimed}
            disabled={loading === "isClaimed" || !contractAddress || !digestHex || !queryClient}
            structure="outlined"
            fullWidth
          >
            {loading === "isClaimed" ? "Loading..." : "Query IsClaimed"}
          </Button>
          {queryResult?.type === "IsClaimed" && (
            <div className="mt-3 rounded bg-gray-900 p-3">
              {queryResult.error ? (
                <p className="text-xs text-red-400">{queryResult.error}</p>
              ) : (
                <pre className="text-xs text-gray-300 overflow-auto">
                  {JSON.stringify(queryResult.data, null, 2)}
                </pre>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

