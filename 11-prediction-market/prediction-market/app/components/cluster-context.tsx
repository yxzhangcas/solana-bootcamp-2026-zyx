"use client";

import {
  createContext,
  useContext,
  useState,
  useCallback,
  type ReactNode,
} from "react";
import type { ClusterMoniker } from "../lib/solana-client";
import { CLUSTERS } from "../lib/solana-client";
import { getExplorerUrl } from "../lib/explorer";

type ClusterContextValue = {
  cluster: ClusterMoniker;
  setCluster: (cluster: ClusterMoniker) => void;
  getExplorerUrl: (path: string) => string;
};

const ClusterContext = createContext<ClusterContextValue | null>(null);

const STORAGE_KEY = "solana-cluster";
function getInitialCluster(): ClusterMoniker {
  // 页面打开时会触发此处的逻辑，直接将网络类型设置为devnet。
  // 但可能由于页面记录了上次使用的网络类型，如果不匹配就会报错。
  // 打开页面后，手动将网络类型点选成devnet，再刷新页面就正常了。
  if (typeof window === "undefined") return "devnet"; 
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored && CLUSTERS.includes(stored as ClusterMoniker)) {
    return stored as ClusterMoniker;
  }
  return "devnet";
}

export { CLUSTERS };

export function ClusterProvider({ children }: { children: ReactNode }) {
  const [cluster, setClusterState] =
    useState<ClusterMoniker>(getInitialCluster);

  const setCluster = useCallback((c: ClusterMoniker) => {
    setClusterState(c);
    localStorage.setItem(STORAGE_KEY, c);
  }, []);

  const explorerUrl = useCallback(
    (path: string) => getExplorerUrl(path, cluster),
    [cluster]
  );

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
