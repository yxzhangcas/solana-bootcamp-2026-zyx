"use client";

import { ClusterSelect } from "./cluster-select";
import { ThemeToggle } from "./standalone/theme-toggle";
import { WalletButton } from "./wallet-button";

export function HeaderPanel() {
  return (
    <header className="mx-auto flex max-w-6xl items-center justify-between px-6 py-4">
      <span className="text-sm font-semibold tracking-tight">
        Solana Starter Kit
      </span>
      <div className="flex items-center gap-3">
        <ThemeToggle />
        <ClusterSelect />
        <WalletButton />
      </div>
    </header>
  );
}
