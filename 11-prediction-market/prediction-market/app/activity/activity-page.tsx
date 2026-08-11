"use client";

import { Footer } from "../market/footer";
import { MarketHeader } from "../market/market-header";
import WalletButton from "../market/wallet-button";
import { useWallet } from "../providers/wallet-provider";
import { WalletIcon } from "../utils/icon";
import { ActivityContent } from "./activity-content";

function WalletNotConnected() {
  return (
    <div className="flex flex-col items-center justify-center py-20">
      <div className="mx-auto w-16 h-16 rounded-full bg-cream flex items-center justify-center mb-4">
        <WalletIcon size={8} />
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
