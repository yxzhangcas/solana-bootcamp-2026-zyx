import Link from "next/link";
import { useState } from "react";
import { CreateMarket } from "./create-market";
import { Footer } from "./footer";
import { MarketList } from "./market-list";
import WalletButton from "./wallet-button";
import { WorkDescription } from "./work-description";

export default function MarketPanel() {
  const [showCreateForm, setShowCreateForm] = useState(false);

  return (
    <div className="min-h-screen text-foreground bg-bg1">
      <header className="sticky top-0 border-b border-border-low bg-bg1/80 backdrop-blur-sm">
        <div className="mx-auto flex max-w-5xl items-center justify-between px-4 py-3">
          <div className="flex items-center gap-3">
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-foreground text-background font-bold text-sm">
              PM
            </div>
            <div>
              <h1 className="text-sm font-semibold">Prediction Markets</h1>
              <p className="text-xs text-muted">Solana Localnet</p>
            </div>
          </div>
          <div className="flex items-center gap-4">
            <span className="text-sm font-medium">Markets</span>
            <Link
              href="/activity"
              className="text-sm text-muted hover:text-foreground transition"
            >
              Activity
            </Link>
            <WalletButton />
          </div>
        </div>
      </header>
      <main className="mx-auto max-w-5xl px-4 py-8">
        <div className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 className="text-2xl font-semibold tracking-tight">Markets</h2>
            <p className="text-sm text-muted">
              Bet SOL on yes/no outcomes. Winners take the pool.
            </p>
          </div>
          <button
            onClick={() => setShowCreateForm(!showCreateForm)}
            className="flex items-center gap-2 rounded-lg bg-foreground text-background px-4 py-2.5 text-sm font-medium transition hover:opacity-90"
          >
            <svg
              className="h-4 w-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 4v16m8-8H4"
              />
            </svg>
            New Market
          </button>
        </div>
        {showCreateForm && (
          <div className="mb-8">
            <CreateMarket onCreated={() => setShowCreateForm(false)} />
          </div>
        )}
        <MarketList />
        <WorkDescription />
      </main>
      <Footer />
    </div>
  );
}
