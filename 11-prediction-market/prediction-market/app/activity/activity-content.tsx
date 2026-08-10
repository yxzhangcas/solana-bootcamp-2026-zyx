import { Address } from "@solana/kit";
import Link from "next/link";
import { PositionsList } from "./position-list";

interface ActivityContentProps {
  walletAddress: Address;
}

export function ActivityContent({ walletAddress }: ActivityContentProps) {
  return (
    <div>
      <div className="mb-8 flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">
            Your Activity
          </h2>
          <p className="text-sm text-muted">
            Track your positions and performance
          </p>
        </div>
        <Link
          href="/"
          className="flex items-center gap-2 text-sm text-muted hover:text-foreground transition"
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
              d="M10 19l-7-7m0 0l7-7m-7 7h18"
            />
          </svg>
          Back to Markets
        </Link>
      </div>
      <PositionsList walletAddress={walletAddress} />
    </div>
  );
}
