"use client";

import { lamports as sol } from "@solana/kit";
import { useState } from "react";
import { toast } from "sonner";
import { ellipsify } from "../lib/explorer";
import { useBalance } from "../lib/hooks/use-balance";
import { lamportsToSolString } from "../lib/lamports";
import { useSolanaClient } from "../lib/solana-client-context";
import { useWallet } from "../lib/wallet/context";
import { useCluster } from "./cluster-context";

export function WalletBalance() {
  const { wallet, status } = useWallet();
  const { cluster, getExplorerUrl } = useCluster();
  const client = useSolanaClient();

  const address = wallet?.account.address;
  const balance = useBalance(address);
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    if (!address) return;
    await navigator.clipboard.writeText(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleAirdrop = async () => {
    if (!address) return;
    try {
      toast.info("Requesting airdrop...");
      const sig = await client.airdrop(address, sol(1_000_000_000n));
      toast.success("Airdrop received!", {
        description: sig ? (
          <a
            href={getExplorerUrl(`/tx/${sig}`)}
            target="_blank"
            rel="noopener noreferrer"
            className="underline"
          >
            View transaction
          </a>
        ) : undefined,
      });
    } catch (err) {
      console.error("Airdrop failed:", err);
      const msg = err instanceof Error ? err.message : String(err);
      const isRateLimited =
        msg.includes("429") || msg.includes("Internal JSON-RPC error");
      toast.error(
        isRateLimited
          ? "Devnet faucet rate-limited. Use the web faucet instead."
          : "Airdrop failed. Try again later.",
        isRateLimited
          ? {
              description: (
                <a
                  href="https://faucet.solana.com/"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="underline"
                >
                  Open faucet.solana.com
                </a>
              ),
            }
          : undefined
      );
    }
  };

  return (
    <>
      {status === "connected" && address && (
        <section className="relative w-full overflow-hidden rounded-2xl border border-border-low bg-card px-5 py-5">
          <div
            className="pointer-events-none absolute inset-0 opacity-100 dark:opacity-0"
            aria-hidden="true"
            style={{
              backgroundImage: `
              linear-gradient(to right, rgba(0,0,0,0.06) 1px, transparent 1px),
              linear-gradient(to bottom, rgba(0,0,0,0.06) 1px, transparent 1px)
            `,
              backgroundSize: "24px 24px",
              mask: "radial-gradient(ellipse 80% 80% at 50% 0%, black, transparent)",
              WebkitMask:
                "radial-gradient(ellipse 80% 80% at 50% 0%, black, transparent)",
            }}
          />
          <div
            className="pointer-events-none absolute inset-0 opacity-0 dark:opacity-100"
            aria-hidden="true"
            style={{
              backgroundImage: `
              linear-gradient(to right, rgba(255,255,255,0.06) 1px, transparent 1px),
              linear-gradient(to bottom, rgba(255,255,255,0.06) 1px, transparent 1px)
            `,
              backgroundSize: "24px 24px",
              mask: "radial-gradient(ellipse 80% 80% at 50% 0%, black, transparent)",
              WebkitMask:
                "radial-gradient(ellipse 80% 80% at 50% 0%, black, transparent)",
            }}
          />
          <div className="relative flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-cream">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  className="h-4 w-4 text-foreground/70"
                >
                  <path d="M21 12V7H5a2 2 0 0 1 0-4h14v4" />
                  <path d="M3 5v14a2 2 0 0 0 2 2h16v-5" />
                  <path d="M18 12a2 2 0 0 0 0 4h4v-4Z" />
                </svg>
              </div>
              <span className="text-sm font-medium">Wallet Balance</span>
              <button
                onClick={handleCopy}
                className="flex cursor-pointer items-center gap-1.5 font-mono text-xs text-muted transition hover:text-foreground"
              >
                {ellipsify(address, 4)}
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  className="h-3 w-3"
                >
                  {copied ? (
                    <path d="M20 6 9 17l-5-5" />
                  ) : (
                    <>
                      <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
                      <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
                    </>
                  )}
                </svg>
              </button>
            </div>
            {cluster !== "mainnet" && (
              <button
                onClick={handleAirdrop}
                className="cursor-pointer rounded-lg border border-border-low px-3 py-1.5 text-xs font-medium transition hover:bg-cream"
              >
                Airdrop
              </button>
            )}
          </div>
          <p className="relative mt-4 font-mono text-4xl font-bold tabular-nums tracking-tight">
            {balance.lamports != null
              ? lamportsToSolString(balance.lamports)
              : "\u2014"}
            <span className="ml-1.5 text-lg font-normal text-muted">SOL</span>
          </p>
        </section>
      )}
    </>
  );
}
