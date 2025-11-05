"use client";
import { useState } from "react";
import { useAbstraxionAccount, useAbstraxionSigningClient } from "@burnt-labs/abstraxion";
import { Button, Input } from "@burnt-labs/ui";
import "@burnt-labs/ui/dist/index.css";
import { TradeOSQueries } from "@/components/TradeOSQueries";
import { SendTokenSection } from "@/components/SendTokenSection";

const tradeosContract = process.env.NEXT_PUBLIC_TRADEOS_CONTRACT || "";

export default function Page(): JSX.Element {
  const { data: account, login, logout, isConnected, isLoading } = useAbstraxionAccount();

  return (
    <main className="m-auto flex min-h-screen max-w-4xl flex-col gap-6 p-4">
      <div className="text-center">
        <h1 className="text-3xl font-bold tracking-tighter text-white">
          TradeOS Contract Demo
        </h1>
        <p className="mt-2 text-sm text-gray-400">
          Interact with TradeOS Smart Contract on XION Testnet
        </p>
      </div>

      {/* Connection Section */}
      <div className="rounded-lg border border-white/10 bg-gray-900/50 p-6">
        <h2 className="mb-4 text-xl font-semibold text-white">Wallet Connection</h2>
        {!isConnected ? (
          <div className="space-y-4">
            <p className="text-sm text-gray-400">
              Connect your wallet to interact with the TradeOS contract
            </p>
            <Button
              onClick={login}
              disabled={isLoading}
              fullWidth
              structure="base"
            >
              {isLoading ? "Connecting..." : "Connect Wallet"}
            </Button>
          </div>
        ) : (
          <div className="space-y-4">
            <div className="rounded border border-green-500/30 bg-green-500/10 p-4">
              <div className="flex items-center gap-2 mb-2">
                <div className="h-2 w-2 animate-pulse rounded-full bg-green-400"></div>
                <span className="font-semibold text-green-400">Connected</span>
              </div>
              <p className="text-xs text-gray-300 break-all">
                {account.bech32Address}
              </p>
            </div>
            <Button
              onClick={logout}
              fullWidth
              structure="outlined"
            >
              Disconnect
            </Button>
          </div>
        )}
      </div>

      {isConnected && (
        <>
          {/* Query Section */}
          <TradeOSQueries contractAddress={tradeosContract} />

          {/* Send Token Section */}
          <SendTokenSection contractAddress={tradeosContract} />
        </>
      )}

      {/* Info Section */}
      <div className="rounded-lg border border-blue-500/30 bg-blue-500/10 p-6">
        <h3 className="mb-2 text-lg font-semibold text-blue-400">Important Notes</h3>
        <ul className="list-disc list-inside space-y-2 text-sm text-gray-300">
          <li>
            Before using this demo, make sure you have configured the Treasury contract
            with <strong>Send Funds</strong> permission. See{" "}
            <a
              href="https://docs.burnt.com/xion/developers/getting-started-advanced/gasless-ux-and-permission-grants/treasury-contracts"
              target="_blank"
              rel="noopener noreferrer"
              className="text-blue-400 underline"
            >
              Treasury Contracts Documentation
            </a>
          </li>
          <li>
            Set <code className="bg-gray-800 px-1 rounded">NEXT_PUBLIC_TREASURY_CONTRACT</code> and{" "}
            <code className="bg-gray-800 px-1 rounded">NEXT_PUBLIC_TRADEOS_CONTRACT</code> in your .env file
          </li>
          <li>
            The TradeOS contract address can be generated using the instantiate command
            in the main README.md (lines 17-43)
          </li>
        </ul>
      </div>
    </main>
  );
}

