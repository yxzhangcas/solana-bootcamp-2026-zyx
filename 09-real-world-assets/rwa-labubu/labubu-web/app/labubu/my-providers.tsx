"use client";

import { SolanaProvider } from "@solana/react-hooks";
import { PropsWithChildren } from "react";

import { autoDiscover, createClient } from "@solana/client";

const client = createClient({
  endpoint: "http://127.0.0.1:8899",
  walletConnectors: autoDiscover(),
});

export function MyProviders({ children }: PropsWithChildren) {
  return <SolanaProvider client={client}>{children}</SolanaProvider>;
}