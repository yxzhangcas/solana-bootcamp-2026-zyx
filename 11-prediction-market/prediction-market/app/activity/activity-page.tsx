"use client";

import { Footer } from "../market/footer";
import { MarketHeader } from "../market/market-header";
import WalletButton from "../market/wallet-button";
import { useWallet } from "../providers/wallet-provider";
import { ActivityContent } from "./activity-content";

function WalletNotConnected() {
  return (
    <div className="flex flex-col items-center justify-center py-20">
      <div className="mx-auto w-16 h-16 rounded-full bg-cream flex items-center justify-center mb-4">
        <svg
          className="h-8 w-8 text-muted"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1.5}
            d="M21 12a2.25 2.25 0 00-2.25-2.25H15a3 3 0 11-6 0H5.25A2.25 2.25 0 003 12m18 0v6a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 18v-6m18 0V9M3 12V9m18 0a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 9m18 0V6a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 6v3"
          />
        </svg>
      </div>
      <h2 className="text-xl font-semibold mb-2">Connect your wallet</h2>
      <p className="text-sm text-muted mb-6 text-center max-w-sm">
        Connect a Solana wallet to view your betting activity and positions
      </p>
      <WalletButton />
    </div>
  );
}

export function ActivityPage() {
  const { wallet, status } = useWallet();
  const walletAddress = wallet?.account.address;
  return (
    <div className="min-h-screen bg-bg1 text-foreground">
      <MarketHeader path="/activity" />
      <main className="mx-auto max-w-5xl px-4 py-8">
        {status !== "connected" ? (
          <WalletNotConnected />
        ) : (
          <ActivityContent walletAddress={walletAddress!} />
        )}
      </main>
      <Footer />
    </div>
  );
}
