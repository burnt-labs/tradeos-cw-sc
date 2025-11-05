"use client";
import { useState } from "react";
import { useAbstraxionAccount, useAbstraxionSigningClient } from "@burnt-labs/abstraxion";
import { Button, Input } from "@burnt-labs/ui";

interface SendTokenSectionProps {
  contractAddress: string;
}

export function SendTokenSection({ contractAddress }: SendTokenSectionProps) {
  const { data: account } = useAbstraxionAccount();
  const { client } = useAbstraxionSigningClient();
  const [amount, setAmount] = useState("");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<{
    success: boolean;
    txHash?: string;
    error?: string;
  } | null>(null);

  const handleSendToken = async () => {
    if (!client || !account.bech32Address || !amount || !contractAddress) {
      return;
    }

    setLoading(true);
    setResult(null);

    try {
      // Convert amount to micro units (uxion)
      const amountNum = parseFloat(amount);
      if (isNaN(amountNum) || amountNum <= 0) {
        throw new Error("Invalid amount");
      }
      const microAmount = Math.floor(amountNum * 1_000_000).toString();

      // Send tokens to contract address
      const txResult = await client.sendTokens(
        account.bech32Address,
        contractAddress,
        [{ denom: "uxion", amount: microAmount }],
        1.5
      );

      if (txResult.code === 0) {
        setResult({
          success: true,
          txHash: txResult.transactionHash,
        });
        setAmount(""); // Clear amount on success
      } else {
        setResult({
          success: false,
          error: `Transaction failed with code: ${txResult.code}`,
        });
      }
    } catch (error) {
      setResult({
        success: false,
        error: error instanceof Error ? error.message : "Unknown error",
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="rounded-lg border border-white/10 bg-gray-900/50 p-6">
      <h2 className="mb-4 text-xl font-semibold text-white">Send Tokens to Contract</h2>
      <p className="mb-4 text-sm text-gray-400">
        Send XION tokens to the TradeOS contract address. This requires the Treasury
        to have <strong>Send Funds</strong> permission configured.
      </p>

      <div className="space-y-4">
        <div>
          <label className="mb-2 block text-sm font-medium text-gray-300">
            Amount (XION)
          </label>
          <Input
            type="number"
            step="0.000001"
            min="0"
            placeholder="0.0"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            disabled={loading}
          />
          <p className="mt-1 text-xs text-gray-500">
            Enter the amount in XION (will be converted to uxion)
          </p>
        </div>

        <div className="rounded border border-gray-700 bg-gray-800/50 p-3">
          <p className="text-xs text-gray-400">
            <strong>Contract Address:</strong>{" "}
            <span className="font-mono text-gray-300 break-all">
              {contractAddress || "Not configured"}
            </span>
          </p>
        </div>

        <Button
          onClick={handleSendToken}
          disabled={
            loading ||
            !amount ||
            !client ||
            !account.bech32Address ||
            !contractAddress
          }
          fullWidth
          structure="base"
        >
          {loading ? "Sending..." : "Send Tokens"}
        </Button>

        {result && (
          <div
            className={`rounded p-4 ${
              result.success
                ? "border border-green-500/30 bg-green-500/10"
                : "border border-red-500/30 bg-red-500/10"
            }`}
          >
            {result.success ? (
              <div>
                <p className="mb-2 text-sm font-semibold text-green-400">
                  Transaction Successful!
                </p>
                <p className="text-xs text-gray-300 break-all">
                  <strong>Tx Hash:</strong> {result.txHash}
                </p>
                <a
                  href={`https://explorer.burnt.com/xion-testnet-2/tx/${result.txHash}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="mt-2 inline-block text-xs text-blue-400 underline"
                >
                  View on Explorer
                </a>
              </div>
            ) : (
              <div>
                <p className="mb-2 text-sm font-semibold text-red-400">
                  Transaction Failed
                </p>
                <p className="text-xs text-gray-300">{result.error}</p>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

