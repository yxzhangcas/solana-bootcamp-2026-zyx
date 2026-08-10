"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import { ClusterMoniker, CLUSTERS } from "../lib/solana-client";

const STORAGE_KEY = "solana-cluster";

function getInitialCluster(): ClusterMoniker {
  if (typeof window === "undefined") return "devnet";
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored && CLUSTERS.includes(stored as ClusterMoniker)) {
    return stored as ClusterMoniker;
  }
  return "devnet";
}

function getExplorerUrl(path: string, cluster: ClusterMoniker): string {
  const base = "https://explorer.solana.com";
  const url = new URL(path, base);
  if (cluster !== "mainnet") {
    if (cluster === "localnet") {
      url.searchParams.set("cluster", "custom");
      url.searchParams.set("customUrl", "http://localhost:8899");
    } else {
      url.searchParams.set("cluster", cluster);
    }
  }
  return url.toString();
}

type ClusterContextValue = {
  cluster: ClusterMoniker;
  setCluster: (cluster: ClusterMoniker) => void;
  getExplorerUrl: (path: string) => string;
};

const ClusterContext = createContext<ClusterContextValue | null>(null);

export function ClusterProvider({ children }: { children: ReactNode }) {
  const [cluster, setClusterState] = useState<ClusterMoniker>("devnet");

  const setCluster = useCallback((c: ClusterMoniker) => {
    setClusterState(c);
    localStorage.setItem(STORAGE_KEY, c);
  }, []);
  const explorerUrl = useCallback(
    (path: string) => getExplorerUrl(path, cluster),
    [cluster]
  );

  useEffect(() => {
    let c = getInitialCluster();
    setClusterState(c);
    localStorage.setItem(STORAGE_KEY, c);
  }, []);

  return (
    <ClusterContext.Provider
      value={{ cluster, setCluster, getExplorerUrl: explorerUrl }}
    >
      {children}
    </ClusterContext.Provider>
  );
}

export function useCluster() {
  const ctx = useContext(ClusterContext);
  if (!ctx) throw new Error("useCluster must be used within ClusterProvider");
  return ctx;
}
