import { useState } from "react";
import { PlusIcon } from "../utils/icon";
import { CreateMarket } from "./create-market";
import { Footer } from "./footer";
import { MarketHeader } from "./market-header";
import { MarketList } from "./market-list";
import { WorkDescription } from "./work-description";

export default function MarketPanel() {
  const [showCreateForm, setShowCreateForm] = useState(false);

  return (
    <div className="min-h-screen text-foreground bg-bg1">
      <MarketHeader path="/" />
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
            <PlusIcon size={4} />
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
