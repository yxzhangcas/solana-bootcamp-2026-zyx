"use client";

import { Market, UserPosition } from "../generated/vault";
import { formatSol } from "./utils";

interface ClaimSectionProps {
  status: string;
  isResolved: boolean;
  userPosition: UserPosition | null;
  market: Market;
  isSending: boolean;
  onClaim: () => void;
}

export default function ClaimSection({
  status,
  isResolved,
  userPosition,
  market,
  isSending,
  onClaim,
}: ClaimSectionProps) {
  if (
    status !== "connected" ||
    !isResolved ||
    !userPosition ||
    userPosition.claimed
  ) {
    return null;
  }
  const outcome = market.outcome;
  if (outcome === null || outcome === undefined) {
    return null;
  }
  const userWinningBet = outcome
    ? userPosition.yesAmount
    : userPosition.noAmount;
  if (userWinningBet === 0n) {
    return null;
  }
  const winningPool = outcome ? market.yesPoolLamports : market.noPoolLamports;
  const losingPool = outcome ? market.noPoolLamports : market.yesPoolLamports;
  const winnings =
    winningPool > 0n ? (userWinningBet * losingPool) / winningPool : 0n;
  const totalPayout = userWinningBet + winnings;
  return (
    <div className="border-t border-border-low p-3 bg-green-50">
      <button
        onClick={onClaim}
        disabled={isSending}
        className="w-full rounded-md bg-green-600 px-3 py-2 text-sm font-medium text-white transition hover:bg-green-700 disabled:opacity-40"
      >
        {isSending ? "Claiming..." : `Claim ${formatSol(totalPayout)} SOL`}
      </button>
    </div>
  );
}
