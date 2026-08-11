import { Address } from "@solana/kit";
import Link from "next/link";
import { BackArrow } from "../utils/icon";
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
          <BackArrow size={4} />
          Back to Markets
        </Link>
      </div>
      <PositionsList walletAddress={walletAddress} />
    </div>
  );
}
