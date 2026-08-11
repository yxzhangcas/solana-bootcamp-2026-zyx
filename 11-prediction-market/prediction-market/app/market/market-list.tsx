"use client";

import { Address } from "@solana/kit";
import { useCallback, useEffect, useMemo, useState } from "react";
import { getMarketDecoder, Market } from "../generated/vault";
import { useCluster } from "../providers/cluster-provider";
import { rpcFetchMarkets } from "../utils/fetch";
import { AnimateSpin, EmptyIcon } from "../utils/icon";
import { MarketCard } from "./market-card";

interface MarketWithAddress {
  address: Address;
  market: Market;
}
type FilterTab = "active" | "past";

export function MarketList() {
  const [markets, setMarkets] = useState<MarketWithAddress[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<FilterTab>("active");

  const { cluster } = useCluster();

  const fetchMarkets = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await rpcFetchMarkets(cluster);
      const decoder = getMarketDecoder();
      const fetchedMarkets: MarketWithAddress[] = [];
      for (const account of result.result || []) {
        try {
          const data = Uint8Array.from(atob(account.account.data[0]), (c) =>
            c.charCodeAt(0)
          );
          const market = decoder.decode(data);
          fetchedMarkets.push({
            address: account.pubkey as Address,
            market,
          });
        } catch (e) {
          console.warn("Failed to decode market account:", account.pubkey, e);
        }
      }
      fetchedMarkets.sort((a, b) =>
        Number(b.market.resolutionTime - a.market.resolutionTime)
      );
      setMarkets(fetchedMarkets);
    } catch (e) {
      console.error("Failed to fetch markets:", e);
      setError(e instanceof Error ? e.message : "Failed to fetch markets");
    } finally {
      setLoading(false);
    }
  }, [cluster]);

  useEffect(() => {
    fetchMarkets();
    const interval = setInterval(fetchMarkets, 5_000);
    return () => clearInterval(interval);
  }, [fetchMarkets]);

  const { activeMarkets, pastMarkets } = useMemo(() => {
    const active: MarketWithAddress[] = [];
    const past: MarketWithAddress[] = [];
    for (const item of markets) {
      if (item.market.resolved) {
        past.push(item);
      } else {
        active.push(item);
      }
    }
    active.sort((a, b) =>
      Number(a.market.resolutionTime - b.market.resolutionTime)
    );
    past.sort((a, b) =>
      Number(a.market.resolutionTime - b.market.resolutionTime)
    );
    return { activeMarkets: active, pastMarkets: past };
  }, [markets]);

  const displayMarkets = activeTab === "active" ? activeMarkets : pastMarkets;

  if (loading && markets.length === 0) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="flex items-center gap-2 text-sm text-muted">
          <AnimateSpin size={4} />
          Loading markets...
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-xl border border-red-200 bg-red-50 p-4 text-center">
        <p className="text-sm text-red-700 mb-2">{error}</p>
        <button
          onClick={fetchMarkets}
          className="text-sm font-medium text-red-600 hover:underline"
        >
          Try again
        </button>
      </div>
    );
  }

  if (markets.length === 0) {
    return (
      <div className="rounded-xl border border-dashed border-border-low p-8 text-center">
        <div className="mx-auto w-12 h-12 rounded-full bg-cream flex items-center justify-center mb-3">
          <EmptyIcon size={6} />
        </div>
        <p className="text-sm text-muted mb-1">No markets yet</p>
        <p className="text-xs text-muted/70">
          Create the first prediction market to get started
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex gap-1 rounded-lg bg-cream p-1">
          <button
            onClick={() => setActiveTab("active")}
            className={`rounded-md px-3 py-1.5 text-sm font-medium transition ${
              activeTab === "active"
                ? "bg-card text-foreground shadow-sm"
                : "text-muted hover:text-foreground"
            }`}
          >
            Active
            {activeMarkets.length > 0 && (
              <span className="ml-1.5 text-xs text-muted">
                ({activeMarkets.length})
              </span>
            )}
          </button>
          <button
            onClick={() => setActiveTab("past")}
            className={`rounded-md px-3 py-1.5 text-sm font-medium transition ${
              activeTab === "past"
                ? "bg-card text-foreground shadow-sm"
                : "text-muted hover:text-foreground"
            }`}
          >
            Past
            {pastMarkets.length > 0 && (
              <span className="ml-1.5 text-xs text-muted">
                ({pastMarkets.length})
              </span>
            )}
          </button>
        </div>
        <button
          onClick={fetchMarkets}
          disabled={loading}
          className="text-xs text-muted hover:text-foreground transition disabled:opacity-50"
        >
          {loading ? "Refreshing..." : "Refresh"}
        </button>
      </div>
      {displayMarkets.length === 0 ? (
        <div className="rounded-xl border border-dashed border-border-low p-8 text-center">
          <p className="text-sm text-muted">
            {activeTab === "active"
              ? "No active markets. Create one to get started!"
              : "No past markets yet."}
          </p>
        </div>
      ) : (
        <div className="grid gap-3 sm:grid-cols-2">
          {displayMarkets.map((item) => (
            <MarketCard
              key={item.address}
              market={item.market}
              marketAddress={item.address}
              onUpdate={fetchMarkets}
            />
          ))}
        </div>
      )}
    </div>
  );
}
