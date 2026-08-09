"use client";

import { GridBackground } from "./components/grid-background";
import { HeaderPanel } from "./components/header-panel";
import { MainPanel } from "./components/main-panel";
import { VaultCard } from "./components/vault-card";
import { WalletBalance } from "./components/wallet-balance";

export default function Home() {
  return (
    <div className="relative min-h-screen bg-background text-foreground">
      <GridBackground />
      <div className="relative z-10">
        <HeaderPanel />
        <main className="mx-auto max-w-6xl px-6">
          <section className="pt-6 pb-20 md:pt-8 md:pb-32">
            <MainPanel />
          </section>
          <div className="space-y-10 pb-20">
            <WalletBalance />
            <VaultCard />
          </div>
        </main>
      </div>
    </div>
  );
}
