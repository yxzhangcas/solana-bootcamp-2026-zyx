import { Address } from "@solana/kit";
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  getMarketDecoder,
  getUserPositionDecoder,
  Market,
  UserPosition,
} from "../generated/vault";
import { useCluster } from "../providers/cluster-provider";
import { rpcFetchMarketAccounts, rpcFetchPositions } from "../utils/fetch";
import { AnimateSpin, EmptyIcon } from "../utils/icon";
import { ActivityStats } from "./activity-stats";
import { PositionCard } from "./position-card";

interface PositionWithMarket {
  positionAddress: Address;
  position: UserPosition;
  marketAddress: Address;
  market: Market | null;
}

export interface ActivityStatsData {
  totalInvested: bigint;
  totalWon: bigint;
  totalClaimed: bigint;
  totalLost: bigint;
  roiPercent: number;
  activePositions: number;
  claimablePositions: number;
}

type FilterTab = "all" | "active" | "resolved" | "claimable";

interface PositionsListProps {
  walletAddress: Address;
}

export function PositionsList({ walletAddress }: PositionsListProps) {
  const { cluster } = useCluster();

  const [positions, setPositions] = useState<PositionWithMarket[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<FilterTab>("all");

  const fetchPositions = useCallback(async () => {
    if (!walletAddress) return;
    try {
      const result = await rpcFetchPositions(cluster, walletAddress);

      const positionDecoder = getUserPositionDecoder();
      const decodedPositions: Array<{
        address: Address;
        position: UserPosition;
      }> = [];
      for (const account of result.result || []) {
        try {
          const data = Uint8Array.from(atob(account.account.data[0]), (c) =>
            c.charCodeAt(0)
          );
          const position = positionDecoder.decode(data);
          decodedPositions.push({
            address: account.pubkey as Address,
            position,
          });
        } catch (decodeError) {
          console.warn(
            "Failed to decode position:",
            account.pubkey,
            decodeError
          );
        }
      }
      const marketAddresses = [
        ...new Set(decodedPositions.map((p) => p.position.market)),
      ];
      const marketMap = new Map<string, Market>();
      if (marketAddresses.length > 0) {
        const marketsResult = await rpcFetchMarketAccounts(
          cluster,
          marketAddresses
        );
        const marketDecoder = getMarketDecoder();

        if (marketsResult.result?.value) {
          marketsResult.result.value.forEach(
            (account: { data: string[] } | null, index: number) => {
              if (account && account.data) {
                try {
                  const data = Uint8Array.from(atob(account.data[0]), (c) =>
                    c.charCodeAt(0)
                  );
                  const market = marketDecoder.decode(data);
                  marketMap.set(marketAddresses[index], market);
                } catch (e) {
                  console.warn(
                    "Failed to decode market:",
                    marketAddresses[index]
                  );
                }
              }
            }
          );
        }
      }
      const enrichedPositions: PositionWithMarket[] = decodedPositions.map(
        (p) => ({
          positionAddress: p.address,
          position: p.position,
          marketAddress: p.position.market,
          market: marketMap.get(p.position.market) || null,
        })
      );

      enrichedPositions.sort((a, b) => {
        const aTime = a.market?.resolutionTime ?? 0n;
        const bTime = b.market?.resolutionTime ?? 0n;
        return Number(bTime - aTime);
      });

      setPositions(enrichedPositions);
      setError(null);
    } catch (e) {
      console.error("Failed to fetch positions:", e);
      setError(e instanceof Error ? e.message : "Failed to fetch positions");
    } finally {
      setLoading(false);
    }
  }, [walletAddress]);

  const stats = useMemo((): ActivityStatsData => {
    let totalInvested = 0n;
    let totalWon = 0n;
    let totalClaimed = 0n;
    let totalLost = 0n;
    let activeCount = 0;
    let claimableCount = 0;

    for (const { position, market } of positions) {
      const invested = position.yesAmount + position.noAmount;
      totalInvested += invested;

      if (!market || !market.resolved) {
        activeCount++;
        continue;
      }

      const outcome = market.outcome;
      if (outcome === null || outcome === undefined) continue;

      const userWinningBet = outcome ? position.yesAmount : position.noAmount;
      const userLosingBet = outcome ? position.noAmount : position.yesAmount;

      if (userWinningBet > 0n) {
        const winningPool = outcome
          ? market.yesPoolLamports
          : market.noPoolLamports;
        const losingPool = outcome
          ? market.noPoolLamports
          : market.yesPoolLamports;
        const winnings =
          winningPool > 0n ? (userWinningBet * losingPool) / winningPool : 0n;
        const payout = userWinningBet + winnings;

        totalWon += winnings;
        totalLost += userLosingBet;

        if (position.claimed) {
          totalClaimed += payout;
        } else {
          claimableCount++;
        }
      } else {
        totalLost += invested;
      }
    }

    const netPnL = totalWon - totalLost;
    const roiPercent =
      totalInvested > 0n ? Number((netPnL * 10000n) / totalInvested) / 100 : 0;

    return {
      totalInvested,
      totalWon,
      totalClaimed,
      totalLost,
      roiPercent,
      activePositions: activeCount,
      claimablePositions: claimableCount,
    };
  }, [positions]);

  useEffect(() => {
    fetchPositions();
    const interval = setInterval(fetchPositions, 5000);
    return () => clearInterval(interval);
  }, [fetchPositions]);

  const filteredPositions = useMemo(() => {
    return positions.filter(({ position, market }) => {
      if (activeTab === "all") return true;
      if (activeTab === "active") return !market?.resolved;
      if (activeTab === "resolved") return market?.resolved;

      if (!market?.resolved || position.claimed) return false;
      const outcome = market.outcome;
      if (outcome === null || outcome === undefined) return false;
      const userWinningBet = outcome ? position.yesAmount : position.noAmount;
      return userWinningBet > 0n;
    });
  }, [positions, activeTab]);

  const tabCounts = useMemo(() => {
    let active = 0;
    let resolved = 0;
    let claimable = 0;

    for (const { position, market } of positions) {
      if (!market?.resolved) {
        active++;
      } else {
        resolved++;
        const outcome = market.outcome;
        if (outcome !== null && outcome !== undefined) {
          const userWinningBet = outcome
            ? position.yesAmount
            : position.noAmount;
          if (userWinningBet > 0n && !position.claimed) {
            claimable++;
          }
        }
      }
    }
    return { all: positions.length, active, resolved, claimable };
  }, [positions]);

  if (loading && positions.length === 0) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="flex items-center gap-2 text-sm text-muted">
          <AnimateSpin size={4} />
          Loading your positions...
        </div>
      </div>
    );
  }
  if (error) {
    return (
      <div className="rounded-xl border border-red-200 bg-red-50 p-4 text-center">
        <p className="text-sm text-red-700 mb-2">{error}</p>
        <button
          onClick={fetchPositions}
          className="text-sm font-medium text-red-600 hover:underline"
        >
          Try again
        </button>
      </div>
    );
  }
  if (positions.length === 0) {
    return (
      <div className="rounded-xl border border-dashed border-border-low p-8 text-center">
        <div className="mx-auto w-12 h-12 rounded-full bg-cream flex items-center justify-center mb-3">
          <EmptyIcon size={6} />
        </div>
        <p className="text-sm text-muted mb-1">No positions yet</p>
        <p className="text-xs text-muted/70 mb-4">
          Place your first bet to start tracking your activity
        </p>
        <a
          href="/"
          className="inline-block text-sm font-medium text-foreground hover:underline"
        >
          Browse markets
        </a>
      </div>
    );
  }
  return (
    <div>
      <ActivityStats stats={stats} isLoading={loading} />
      {/* Positions Section */}
      <div className="space-y-4 py-5">
        {/* Tabs */}
        <div className="flex items-center justify-between">
          <div className="flex gap-1 rounded-lg bg-cream p-1">
            {(["all", "active", "resolved", "claimable"] as const).map(
              (tab) => (
                <button
                  key={tab}
                  onClick={() => setActiveTab(tab)}
                  className={`rounded-md px-3 py-1.5 text-sm font-medium transition ${
                    activeTab === tab
                      ? "bg-card text-foreground shadow-sm"
                      : "text-muted hover:text-foreground"
                  }`}
                >
                  {tab.charAt(0).toUpperCase() + tab.slice(1)}
                  {tabCounts[tab] > 0 && (
                    <span className="ml-1.5 text-xs text-muted">
                      ({tabCounts[tab]})
                    </span>
                  )}
                </button>
              )
            )}
          </div>
          <button
            onClick={fetchPositions}
            disabled={loading}
            className="text-xs text-muted hover:text-foreground transition disabled:opacity-50"
          >
            {loading ? "Refreshing..." : "Refresh"}
          </button>
        </div>

        {/* Positions Grid */}
        {filteredPositions.length === 0 ? (
          <div className="rounded-xl border border-dashed border-border-low p-8 text-center">
            <p className="text-sm text-muted">
              No {activeTab === "all" ? "" : activeTab} positions
            </p>
          </div>
        ) : (
          <div className="space-y-3">
            {filteredPositions.map((item, index) => (
              <PositionCard
                key={item.positionAddress}
                position={item.position}
                positionAddress={item.positionAddress}
                market={item.market}
                marketAddress={item.marketAddress}
                onUpdate={fetchPositions}
                animationDelay={index * 50}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
