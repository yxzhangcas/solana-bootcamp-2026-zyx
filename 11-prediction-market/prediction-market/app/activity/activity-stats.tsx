import { formatSol } from "../market/utils";
import { DollarCircle, RightArrow } from "../utils/icon";
import { ActivityStatsData } from "./position-list";

function getRoiColorClass(hasActivity: boolean, isPositive: boolean): string {
  if (!hasActivity) return "text-muted";
  if (isPositive) return "text-green-600";
  return "text-red-600";
}

function getRoiPrefix(hasActivity: boolean, roiPercent: number): string {
  if (!hasActivity) return "";
  if (roiPercent >= 0) return "+";
  return "";
}

interface ActivityStatsProps {
  stats: ActivityStatsData;
  isLoading: boolean;
}

export function ActivityStats({ stats }: ActivityStatsProps) {
  const netPnL = stats.totalWon - stats.totalLost;
  const isPositive = netPnL >= 0n;
  const gradientClass = isPositive
    ? "bg-gradient-to-br from-green-500"
    : "bg-gradient-to-br from-red-500";
  const hasActivity = stats.totalInvested > 0n;
  const roiColorClass = getRoiColorClass(hasActivity, isPositive);
  const roiPrefix = getRoiPrefix(hasActivity, stats.roiPercent);
  const roiValue = hasActivity ? stats.roiPercent.toFixed(1) : "0.0";
  return (
    <div className="animate-fade-in">
      <div className="grid gap-4 md:grid-cols-12">
        <div className="md:col-span-5 rounded-xl border border-border-low bg-card p-6 relative overflow-hidden">
          <div className={`absolute inset-0 opacity-[0.03] ${gradientClass}`} />
          <div className="relative">
            <p className="text-xs font-medium text-muted uppercase tracking-wider mb-1">
              Return on Investment
            </p>
            <div className="flex items-baseline gap-2">
              <span
                className={`font-mono text-4xl font-bold tracking-tight ${roiColorClass}`}
              >
                {roiPrefix}
                {roiValue}%
              </span>
            </div>
            {hasActivity && (
              <p
                className={`mt-2 text-sm font-mono ${
                  isPositive ? "text-green-600/80" : "text-red-600/80"
                }`}
              >
                {isPositive ? "+" : "-"}
                {formatSol(isPositive ? netPnL : -netPnL)} SOL net
              </p>
            )}
          </div>
        </div>

        {/* Secondary Stats Grid */}
        <div className="md:col-span-7 grid grid-cols-2 gap-4">
          {/* Total Invested */}
          <div className="rounded-xl border border-border-low bg-card p-4 animate-fade-in stagger-1">
            <p className="text-xs font-medium text-muted uppercase tracking-wider mb-1">
              Total Invested
            </p>
            <p className="font-mono text-2xl font-semibold tracking-tight">
              {formatSol(stats.totalInvested)}
              <span className="text-sm font-normal text-muted ml-1">SOL</span>
            </p>
          </div>

          {/* Total Claimed */}
          <div className="rounded-xl border border-border-low bg-card p-4 animate-fade-in stagger-2">
            <p className="text-xs font-medium text-muted uppercase tracking-wider mb-1">
              Total Claimed
            </p>
            <p className="font-mono text-2xl font-semibold tracking-tight text-green-600">
              {formatSol(stats.totalClaimed)}
              <span className="text-sm font-normal text-green-600/70 ml-1">
                SOL
              </span>
            </p>
          </div>

          {/* Won (profit only) */}
          <div className="rounded-xl border border-border-low bg-card p-4 animate-fade-in stagger-3">
            <p className="text-xs font-medium text-muted uppercase tracking-wider mb-1">
              Profit Won
            </p>
            <p className="font-mono text-2xl font-semibold tracking-tight text-green-600">
              +{formatSol(stats.totalWon)}
              <span className="text-sm font-normal text-green-600/70 ml-1">
                SOL
              </span>
            </p>
          </div>

          {/* Lost */}
          <div className="rounded-xl border border-border-low bg-card p-4 animate-fade-in stagger-4">
            <p className="text-xs font-medium text-muted uppercase tracking-wider mb-1">
              Total Lost
            </p>
            <p className="font-mono text-2xl font-semibold tracking-tight text-red-600">
              -{formatSol(stats.totalLost)}
              <span className="text-sm font-normal text-red-600/70 ml-1">
                SOL
              </span>
            </p>
          </div>
        </div>
      </div>

      {/* Claimable Banner */}
      {stats.claimablePositions > 0 && (
        <div className="mt-4 rounded-xl bg-green-50 border border-green-200 p-4 flex items-center justify-between animate-fade-in stagger-5">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-green-100">
              <DollarCircle size={5} color="green" />
            </div>
            <div>
              <p className="font-medium text-green-800">
                {stats.claimablePositions} winning{" "}
                {stats.claimablePositions === 1 ? "position" : "positions"} to
                claim
              </p>
              <p className="text-sm text-green-700">
                Switch to the "Claimable" tab to collect your winnings
              </p>
            </div>
          </div>
          <div className="hidden sm:flex h-8 w-8 items-center justify-center rounded-full bg-green-200 text-green-700">
            <RightArrow size={4} />
          </div>
        </div>
      )}
    </div>
  );
}
