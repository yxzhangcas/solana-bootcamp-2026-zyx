import Link from "next/link";
import WalletButton from "./wallet-button";

export function MarketHeader({ path }: { path: string }) {
  return (
    <header className="sticky top-0 border-b border-border-low bg-bg1/80 backdrop-blur-sm">
      <div className="mx-auto flex max-w-5xl items-center justify-between px-4 py-3">
        <div className="flex items-center gap-3">
          <Link
            href="/"
            className="flex items-center gap-3 hover:opacity-80 transition"
          >
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-foreground text-background font-bold text-sm">
              PM
            </div>
            <div>
              <h1 className="text-sm font-semibold">Prediction Markets</h1>
              <p className="text-xs text-muted">Solana Localnet</p>
            </div>
          </Link>
        </div>
        <div className="flex items-center gap-4">
          {path === "/" ? (
            <>
              <span className="text-sm font-medium">Markets</span>
              <Link
                href="/activity"
                className="text-sm text-muted hover:text-foreground transition"
              >
                Activity
              </Link>
            </>
          ) : (
            <>
              <Link
                href="/"
                className="text-sm text-muted hover:text-foreground transition"
              >
                Markets
              </Link>
              <span className="text-sm font-medium">Activity</span>
            </>
          )}
          <WalletButton />
        </div>
      </div>
    </header>
  );
}
